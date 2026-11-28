//! Request bodies for the auth endpoints.

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::platform::companies::dto::CreateCompany;

/// Body of POST /auth/login.
#[derive(Debug, Deserialize)]
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

/// Body of POST /auth/register (US-01): the owner + their company.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: SecretString, // never Debug-printed or logged, same as LoginRequest
    pub first_name: String,
    pub last_name: String,
    pub company: CreateCompany, // nested object: { "company": { "legal_name": ..., ... } }
}
