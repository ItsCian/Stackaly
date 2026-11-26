//! Integration tests for /companies, against a real Postgres (db-test).
//! Run with `just test`: it starts db-test and applies migrations first.

use api::{
    auth::password::hash_password,
    build_app,
    entities::{companies, memberships, users},
    state::AppState,
};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use http_body_util::BodyExt;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, NotSet, Set};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

const PASSWORD: &str = "test-password";

// ── Helpers ─────────────────────────────────────────────────────────

/// The app plus a direct database connection, to set up data the API
/// can't create yet (users and memberships, until sign-up exists).
struct TestApp {
    router: Router,
    db: DatabaseConnection,
}

/// Builds the app like main.rs, connected to the test database.
async fn app() -> TestApp {
    dotenvy::dotenv().ok();
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set (run the tests with `just test`)");
    let db =
        sea_orm::Database::connect(&url).await.expect("failed to connect to the test database");

    TestApp { router: build_app(AppState { db: db.clone() }), db }
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

/// Sends one request, with an optional JSON body and an optional session cookie.
async fn send(
    app: &TestApp,
    method: &str,
    uri: &str,
    body: Option<Value>,
    cookie: Option<&str>,
) -> Response {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    let request = match body {
        Some(json) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap();

    app.router.clone().oneshot(request).await.unwrap()
}

/// Reads a response body as JSON.
async fn body_json(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Creates a company through the public endpoint and returns it as JSON.
async fn create_company(app: &TestApp) -> Value {
    let response =
        send(app, "POST", "/api/companies", Some(company_body(&unique_vat())), None).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    body_json(response).await
}

/// Creates a user with the given role in a company, logs them in,
/// and returns the session cookie ("session=...").
async fn session_for(app: &TestApp, company_id: &str, role: &str) -> String {
    let user_id = Uuid::now_v7();
    let email = format!("test-{user_id}@example.com");

    users::ActiveModel {
        id: Set(user_id),
        email: Set(email.clone()),
        password_hash: Set(hash_password(PASSWORD).unwrap()),
        first_name: Set("Test".into()),
        last_name: Set("User".into()),
        disabled_at: NotSet,
        email_verified_at: NotSet,
        must_change_password: NotSet,
        password_changed_at: NotSet,
        failed_login_count: NotSet,
        locked_until: NotSet,
        last_login_at: NotSet,
        created_at: NotSet,
        updated_at: NotSet,
    }
    .insert(&app.db)
    .await
    .unwrap();

    memberships::ActiveModel {
        user_id: Set(user_id),
        company_id: Set(company_id.parse().unwrap()),
        role: Set(role.into()),
        deactivated_at: NotSet,
        added_by: NotSet,
        created_at: NotSet,
    }
    .insert(&app.db)
    .await
    .unwrap();

    // One membership -> login selects this company automatically
    let response = send(
        app,
        "POST",
        "/api/auth/login",
        Some(json!({ "email": email, "password": PASSWORD })),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let set_cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    // Keep only "session=<token>", drop the attributes after the first ';'
    set_cookie.split(';').next().unwrap().to_string()
}

/// Creates a company and a logged-in owner. Returns (company JSON, cookie).
async fn owner_session(app: &TestApp) -> (Value, String) {
    let company = create_company(app).await;
    let cookie = session_for(app, company["id"].as_str().unwrap(), "owner").await;
    (company, cookie)
}

// ── Create (public) ─────────────────────────────────────────────────

#[tokio::test]
async fn create_then_get_company() {
    let app = app().await;
    let (created, cookie) = owner_session(&app).await;
    assert!(created["suspended_at"].is_null());
    assert!(created["closed_at"].is_null());

    let id = created["id"].as_str().unwrap();
    let response = send(&app, "GET", &format!("/api/companies/{id}"), None, Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await, created);
}

#[tokio::test]
async fn create_with_duplicate_vat_returns_conflict() {
    let app = app().await;
    let vat = unique_vat();

    let first = send(&app, "POST", "/api/companies", Some(company_body(&vat)), None).await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let second = send(&app, "POST", "/api/companies", Some(company_body(&vat)), None).await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
    assert_eq!(body_json(second).await["code"], "conflict");
}

#[tokio::test]
async fn create_with_missing_field_returns_invalid_request() {
    let app = app().await;
    let mut body = company_body(&unique_vat());
    body.as_object_mut().unwrap().remove("city");

    let response = send(&app, "POST", "/api/companies", Some(body), None).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_json(response).await["code"], "invalid_request");
}

#[tokio::test]
async fn create_with_broken_json_returns_invalid_request() {
    let app = app().await;
    let request = Request::builder()
        .method("POST")
        .uri("/api/companies")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"legal_name": "Broken""#)) // no closing brace
        .unwrap();

    let response = app.router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["code"], "invalid_request");
}

// ── Access control ──────────────────────────────────────────────────

#[tokio::test]
async fn list_route_is_disabled() {
    let app = app().await;
    let (_, cookie) = owner_session(&app).await;

    // Only POST is left on "/" until the super-admin (US-09)
    let response = send(&app, "GET", "/api/companies", None, Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn get_without_session_returns_unauthorized() {
    let app = app().await;
    let company = create_company(&app).await;
    let id = company["id"].as_str().unwrap();

    let response = send(&app, "GET", &format!("/api/companies/{id}"), None, None).await;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_json(response).await["code"], "unauthorized");
}

#[tokio::test]
async fn get_other_company_returns_not_found() {
    let app = app().await;
    let (_, my_cookie) = owner_session(&app).await;
    let (other, _) = owner_session(&app).await;
    let other_id = other["id"].as_str().unwrap();

    // Tenant isolation: another company is "not found", not "forbidden"
    let response =
        send(&app, "GET", &format!("/api/companies/{other_id}"), None, Some(&my_cookie)).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_unknown_id_returns_not_found() {
    let app = app().await;
    let (_, cookie) = owner_session(&app).await;

    let response =
        send(&app, "GET", &format!("/api/companies/{}", Uuid::now_v7()), None, Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(response).await["code"], "not_found");
}

#[tokio::test]
async fn get_invalid_id_returns_invalid_request() {
    let app = app().await;
    let (_, cookie) = owner_session(&app).await;

    let response = send(&app, "GET", "/api/companies/abc", None, Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["code"], "invalid_request");
}

#[tokio::test]
async fn employee_can_read_but_not_update() {
    let app = app().await;
    let company = create_company(&app).await;
    let id = company["id"].as_str().unwrap();
    let cookie = session_for(&app, id, "employee").await;
    let uri = format!("/api/companies/{id}");

    let response = send(&app, "GET", &uri, None, Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::OK);

    let response = send(&app, "PATCH", &uri, Some(json!({ "city": "Namur" })), Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(body_json(response).await["code"], "forbidden");
}

// ── Update (owner) ──────────────────────────────────────────────────

#[tokio::test]
async fn update_changes_only_sent_fields() {
    let app = app().await;
    let (created, cookie) = owner_session(&app).await;
    let id = created["id"].as_str().unwrap();

    let response = send(
        &app,
        "PATCH",
        &format!("/api/companies/{id}"),
        Some(json!({ "city": "Namur", "postal_code": "5000" })),
        Some(&cookie),
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
    let (second, cookie) = owner_session(&app).await;
    let id = second["id"].as_str().unwrap();

    let response = send(
        &app,
        "PATCH",
        &format!("/api/companies/{id}"),
        Some(json!({ "vat_number": first["vat_number"] })),
        Some(&cookie),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(body_json(response).await["code"], "conflict");
}

#[tokio::test]
async fn update_unknown_id_returns_not_found() {
    let app = app().await;
    let (_, cookie) = owner_session(&app).await;

    let response = send(
        &app,
        "PATCH",
        &format!("/api/companies/{}", Uuid::now_v7()),
        Some(json!({ "city": "Namur" })),
        Some(&cookie),
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── Delete (owner, closes the company) ──────────────────────────────

#[tokio::test]
async fn delete_closes_company_without_erasing_it() {
    let app = app().await;
    let (created, cookie) = owner_session(&app).await;
    let id = created["id"].as_str().unwrap();
    let uri = format!("/api/companies/{id}");

    let response = send(&app, "DELETE", &uri, None, Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // From the API: gone, reading it again is a 404
    let response = send(&app, "GET", &uri, None, Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // In the database: still there, with closed_at set
    let row = companies::Entity::find_by_id(id.parse::<Uuid>().unwrap())
        .one(&app.db)
        .await
        .unwrap()
        .expect("a closed company must not be erased");
    assert!(row.closed_at.is_some());
}

#[tokio::test]
async fn delete_twice_returns_not_found() {
    let app = app().await;
    let (created, cookie) = owner_session(&app).await;
    let uri = format!("/api/companies/{}", created["id"].as_str().unwrap());

    send(&app, "DELETE", &uri, None, Some(&cookie)).await;
    let response = send(&app, "DELETE", &uri, None, Some(&cookie)).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ── CSRF ────────────────────────────────────────────────────────────

#[tokio::test]
async fn cross_site_patch_is_blocked() {
    let app = app().await;
    let (created, cookie) = owner_session(&app).await;
    let id = created["id"].as_str().unwrap();

    // A valid owner session, but the browser says the request comes from another site
    let request = Request::builder()
        .method("PATCH")
        .uri(format!("/api/companies/{id}"))
        .header(header::COOKIE, &cookie)
        .header("sec-fetch-site", "cross-site")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({ "city": "Namur" }).to_string()))
        .unwrap();

    let response = app.router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(body_json(response).await["code"], "forbidden");
}

// ── Other ───────────────────────────────────────────────────────────

#[tokio::test]
async fn unknown_route_returns_not_found_json() {
    let app = app().await;

    let response = send(&app, "GET", "/api/nope", None, None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(body_json(response).await["code"], "not_found");
}
