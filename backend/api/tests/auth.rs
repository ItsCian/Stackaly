//! Integration tests for POST /auth/register (US-01), against a real Postgres (db-test).
//! Run with `just test`: it starts db-test and applies migrations first.
//!
//! Every test uses a fresh email and VAT number, so tests can run in
//! parallel and the test database never needs to be emptied.

use api::{
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
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

const PASSWORD: &str = "a-long-password";

// ── Helpers ─────────────────────────────────────────────────────────
// Same helpers as tests/companies.rs. Each file in tests/ is compiled as its
// own crate, so they can't share code directly (see tests/common/ later).

/// The app plus a direct database connection, to check what was stored.
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

/// A random VAT number that passes `normalize_vat`.
fn unique_vat() -> String {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).unwrap();
    let base = u64::from_le_bytes(bytes) % 20_000_000; // 0..=19_999_999
    let check = 97 - (base % 97);
    format!("BE{base:08}{check:02}")
}

/// An email no other test uses.
fn unique_email() -> String {
    format!("owner-{}@example.com", Uuid::now_v7())
}

/// A valid register body with the given email and VAT number.
fn register_body(email: &str, vat: &str) -> Value {
    json!({
        "email": email,
        "password": PASSWORD,
        "first_name": "Test",
        "last_name": "Owner",
        "company": {
            "legal_name": "Register Test",
            "vat_number": vat,
            "street": "Rue Test 1",
            "postal_code": "1348",
            "city": "Louvain-la-Neuve",
            "country": "BE"
        }
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

/// Keeps only "session=<token>" from a Set-Cookie header,
/// dropping the attributes after the first ';'.
fn session_cookie(response: &Response) -> String {
    let set_cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    set_cookie.split(';').next().unwrap().to_string()
}

/// Looks up a company by its (normalized) VAT number, straight in the DB.
async fn company_by_vat(app: &TestApp, vat: &str) -> Option<companies::Model> {
    companies::Entity::find()
        .filter(companies::Column::VatNumber.eq(vat))
        .one(&app.db)
        .await
        .unwrap()
}

/// Looks up a user by email, straight in the DB.
async fn user_by_email(app: &TestApp, email: &str) -> Option<users::Model> {
    users::Entity::find().filter(users::Column::Email.eq(email)).one(&app.db).await.unwrap()
}

// ── Happy path ──────────────────────────────────────────────────────

/// AC 3: after signing up, the owner is logged in, inside their new company.
#[tokio::test]
async fn register_logs_in_as_owner() {
    let app = app().await;
    let email = unique_email();

    let response =
        send(&app, "POST", "/api/auth/register", Some(register_body(&email, &unique_vat())), None)
            .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let cookie = session_cookie(&response);

    // The cookie alone is enough to be recognized
    let me = send(&app, "GET", "/api/auth/me", None, Some(&cookie)).await;
    assert_eq!(me.status(), StatusCode::OK);
    let me = body_json(me).await;
    assert_eq!(me["role"], "owner");
    assert!(me["company_id"].is_string(), "the new company must be the active one");

    // And the DB has exactly what we expect: one owner membership, added by nobody
    let user = user_by_email(&app, &email).await.expect("user was not created");
    let membership = memberships::Entity::find()
        .filter(memberships::Column::UserId.eq(user.id))
        .one(&app.db)
        .await
        .unwrap()
        .expect("membership was not created");
    assert_eq!(membership.role, "owner");
    assert_eq!(membership.added_by, None);
    assert_eq!(membership.company_id.to_string(), me["company_id"].as_str().unwrap());
}

/// The session cookie has the same security flags as the login one.
#[tokio::test]
async fn register_sets_a_secure_cookie() {
    let app = app().await;

    let response = send(
        &app,
        "POST",
        "/api/auth/register",
        Some(register_body(&unique_email(), &unique_vat())),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let set_cookie = response.headers()[header::SET_COOKIE].to_str().unwrap();
    assert!(set_cookie.starts_with("session="));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("Secure"));
    assert!(set_cookie.contains("SameSite=Strict"));
}

/// Email and VAT number are stored in their normalized form,
/// whatever way the user typed them.
#[tokio::test]
async fn register_normalizes_email_and_vat() {
    let app = app().await;
    let email = unique_email();
    let vat = unique_vat(); // e.g. "BE0812345603"

    // "  OWNER-...@EXAMPLE.COM " and "be 0812.345.603"
    let messy_email = format!("  {}  ", email.to_uppercase());
    let messy_vat = format!("be {}.{}.{}", &vat[2..6], &vat[6..9], &vat[9..]);

    let response = send(
        &app,
        "POST",
        "/api/auth/register",
        Some(register_body(&messy_email, &messy_vat)),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    assert!(user_by_email(&app, &email).await.is_some(), "email was not normalized");
    assert!(company_by_vat(&app, &vat).await.is_some(), "VAT number was not normalized");
}

/// The password chosen at sign-up works for logging in later.
#[tokio::test]
async fn register_then_login_with_same_credentials() {
    let app = app().await;
    let email = unique_email();

    let response =
        send(&app, "POST", "/api/auth/register", Some(register_body(&email, &unique_vat())), None)
            .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let login = send(
        &app,
        "POST",
        "/api/auth/login",
        Some(json!({ "email": email, "password": PASSWORD })),
        None,
    )
    .await;
    assert_eq!(login.status(), StatusCode::NO_CONTENT);
}

// ── Conflicts (AC 2: email already in use) ──────────────────────────

/// The important one: proves the transaction is all-or-nothing.
/// The company is inserted BEFORE the user, so when the user insert fails
/// on the duplicate email, the company must be rolled back with it.
#[tokio::test]
async fn duplicate_email_is_rejected_and_rolled_back() {
    let app = app().await;
    let email = unique_email();

    let first =
        send(&app, "POST", "/api/auth/register", Some(register_body(&email, &unique_vat())), None)
            .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    // Same email, but a NEW valid VAT number: the company insert succeeds,
    // then the user insert fails.
    let second_vat = unique_vat();
    let second =
        send(&app, "POST", "/api/auth/register", Some(register_body(&email, &second_vat)), None)
            .await;
    assert_eq!(second.status(), StatusCode::CONFLICT);
    let body = body_json(second).await;
    assert_eq!(body["message"], "This email address is already in use");

    // No orphan company left behind
    assert!(
        company_by_vat(&app, &second_vat).await.is_none(),
        "the company of the failed sign-up was not rolled back"
    );
}

/// A VAT number already used by another company is refused,
/// and no user is created for the failed sign-up.
#[tokio::test]
async fn duplicate_vat_is_rejected() {
    let app = app().await;
    let vat = unique_vat();

    let first =
        send(&app, "POST", "/api/auth/register", Some(register_body(&unique_email(), &vat)), None)
            .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let second_email = unique_email();
    let second =
        send(&app, "POST", "/api/auth/register", Some(register_body(&second_email, &vat)), None)
            .await;
    assert_eq!(second.status(), StatusCode::CONFLICT);

    assert!(user_by_email(&app, &second_email).await.is_none());
}

// ── Validation (AC 2: invalid VAT number format) ────────────────────

#[tokio::test]
async fn invalid_vat_is_rejected() {
    let app = app().await;
    let email = unique_email();

    // Valid format, wrong checksum (BE0403219991 is the valid one)
    let response =
        send(&app, "POST", "/api/auth/register", Some(register_body(&email, "BE0403219990")), None)
            .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["code"], "validation_error");

    assert!(user_by_email(&app, &email).await.is_none());
}

#[tokio::test]
async fn short_password_is_rejected() {
    let app = app().await;
    let email = unique_email();
    let mut body = register_body(&email, &unique_vat());
    body["password"] = json!("short");

    let response = send(&app, "POST", "/api/auth/register", Some(body), None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["code"], "validation_error");

    assert!(user_by_email(&app, &email).await.is_none());
}

/// Characters, not bytes: 5 accented letters are 10 bytes but only 5 characters.
#[tokio::test]
async fn password_length_counts_characters_not_bytes() {
    let app = app().await;
    let mut body = register_body(&unique_email(), &unique_vat());
    body["password"] = json!("ééééé");

    let response = send(&app, "POST", "/api/auth/register", Some(body), None).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn invalid_email_is_rejected() {
    let app = app().await;

    let response = send(
        &app,
        "POST",
        "/api/auth/register",
        Some(register_body("not-an-email", &unique_vat())),
        None,
    )
    .await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(body_json(response).await["code"], "validation_error");
}

/// A missing field is rejected by the JSON extractor, before the service runs.
#[tokio::test]
async fn missing_company_is_rejected() {
    let app = app().await;
    let mut body = register_body(&unique_email(), &unique_vat());
    body.as_object_mut().unwrap().remove("company");

    let response = send(&app, "POST", "/api/auth/register", Some(body), None).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_json(response).await["code"], "invalid_request");
}
