//! Integration tests for /companies, against a real Postgres (db-test).
//! Run with `just test`: it starts db-test and applies migrations first.

use api::{build_app, state::AppState};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

// ── Helpers ─────────────────────────────────────────────────────────

/// Builds the app like main.rs, connected to the test database.
async fn app() -> Router {
    dotenvy::dotenv().ok();
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set (run the tests with `just test`)");
    let db =
        sea_orm::Database::connect(&url).await.expect("failed to connect to the test database");

    build_app(AppState { db })
}

/// A VAT number no other test uses, so tests can run in parallel.
fn unique_vat() -> String {
    format!("TEST-{}", Uuid::now_v7())
}

/// A valid create body with the given VAT number.
fn company_body(vat: &str) -> Value {
    json!({
        "legal_name": "Integration Test",
        "vat_number": vat,
        "street": "Rue Test 1",
        "postal_code": "1000",
        "city": "Brussels",
        "country": "BE"
    })
}

/// Sends one request, with an optional JSON body.
async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> Response {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => {
            builder.header("content-type", "application/json").body(Body::from(json.to_string()))
        }
        None => builder.body(Body::empty()),
    }
    .unwrap();

    app.clone().oneshot(request).await.unwrap()
}

/// Reads a response body as JSON.
async fn body_json(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Creates a company and returns it as JSON.
async fn create_company(app: &Router) -> Value {
    let response = send(app, "POST", "/api/companies", Some(company_body(&unique_vat()))).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    body_json(response).await
}

// ── Create ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_then_get_company() {
    let app = app().await;
    let vat = unique_vat();

    let response = send(&app, "POST", "/api/companies", Some(company_body(&vat))).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let created = body_json(response).await;
    assert_eq!(created["vat_number"], vat);
    assert!(created["suspended_at"].is_null());

    let id = created["id"].as_str().unwrap();
    let response = send(&app, "GET", &format!("/api/companies/{id}"), None).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, created);
}

#[tokio::test]
async fn create_with_duplicate_vat_returns_conflict() {
    let app = app().await;
    let vat = unique_vat();

    let first = send(&app, "POST", "/api/companies", Some(company_body(&vat))).await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = send(&app, "POST", "/api/companies", Some(company_body(&vat))).await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
    assert_eq!(body_json(second).await["code"], "conflict");
}

#[tokio::test]
async fn create_with_missing_field_returns_invalid_request() {
    let app = app().await;
    let mut body = company_body(&unique_vat());
    body.as_object_mut().unwrap().remove("city");

    let response = send(&app, "POST", "/api/companies", Some(body)).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_json(response).await["code"], "invalid_request");
}

#[tokio::test]
async fn create_with_broken_json_returns_invalid_request() {
    let app = app().await;
    let request = Request::builder()
        .method("POST")
        .uri("/api/companies")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"legal_name": "Broken""#)) // no closing brace
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["code"], "invalid_request");
}

// ── Read ────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_contains_created_company() {
    let app = app().await;
    let created = create_company(&app).await;

    let response = send(&app, "GET", "/api/companies", None).await;
    assert_eq!(response.status(), StatusCode::OK);

    // Other tests create companies in parallel: check ours is there, not the count.
    let list = body_json(response).await;
    let ids: Vec<&Value> = list.as_array().unwrap().iter().map(|c| &c["id"]).collect();
    assert!(ids.contains(&&created["id"]));
}

#[tokio::test]
async fn get_unknown_id_returns_not_found() {
    let app = app().await;

    let response = send(&app, "GET", &format!("/api/companies/{}", Uuid::now_v7()), None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(response).await["code"], "not_found");
}

#[tokio::test]
async fn get_invalid_id_returns_invalid_request() {
    let app = app().await;

    let response = send(&app, "GET", "/api/companies/abc", None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["code"], "invalid_request");
}

// ── Update ──────────────────────────────────────────────────────────

#[tokio::test]
async fn update_changes_only_sent_fields() {
    let app = app().await;
    let created = create_company(&app).await;
    let id = created["id"].as_str().unwrap();

    let response = send(
        &app,
        "PATCH",
        &format!("/api/companies/{id}"),
        Some(json!({ "city": "Namur", "postal_code": "5000" })),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let updated = body_json(response).await;
    // Changed
    assert_eq!(updated["city"], "Namur");
    assert_eq!(updated["postal_code"], "5000");
    assert_ne!(updated["updated_at"], created["updated_at"]);
    // Unchanged
    assert_eq!(updated["legal_name"], created["legal_name"]);
    assert_eq!(updated["vat_number"], created["vat_number"]);
    assert_eq!(updated["street"], created["street"]);
    assert_eq!(updated["created_at"], created["created_at"]);
}

#[tokio::test]
async fn update_with_duplicate_vat_returns_conflict() {
    let app = app().await;
    let first = create_company(&app).await;
    let second = create_company(&app).await;
    let id = second["id"].as_str().unwrap();

    let response = send(
        &app,
        "PATCH",
        &format!("/api/companies/{id}"),
        Some(json!({ "vat_number": first["vat_number"] })),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(body_json(response).await["code"], "conflict");
}

#[tokio::test]
async fn update_unknown_id_returns_not_found() {
    let app = app().await;

    let response = send(
        &app,
        "PATCH",
        &format!("/api/companies/{}", Uuid::now_v7()),
        Some(json!({ "city": "Namur" })),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── Delete ──────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_removes_company() {
    let app = app().await;
    let created = create_company(&app).await;
    let uri = format!("/api/companies/{}", created["id"].as_str().unwrap());

    let response = send(&app, "DELETE", &uri, None).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Gone: reading it again is a 404
    let response = send(&app, "GET", &uri, None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_twice_returns_not_found() {
    let app = app().await;
    let created = create_company(&app).await;
    let uri = format!("/api/companies/{}", created["id"].as_str().unwrap());

    send(&app, "DELETE", &uri, None).await;
    let response = send(&app, "DELETE", &uri, None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── Other ───────────────────────────────────────────────────────────

#[tokio::test]
async fn unknown_route_returns_not_found_json() {
    let app = app().await;

    let response = send(&app, "GET", "/api/nope", None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(response).await["code"], "not_found");
}
