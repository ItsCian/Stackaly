//! Request bodies for the auth endpoints.

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]

/// Body of POST /auth/login.
pub struct LoginRequest {
    pub email: String,
    pub password: SecretString,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: Uuid,
    pub company_id: Option<Uuid>,
    pub role: Option<String>,
}
