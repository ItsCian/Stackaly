//! Request bodies for the auth endpoints.

use secrecy::SecretString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]

/// Body of POST /auth/login.
pub struct LoginRequest {
    pub email: String,
    pub password: SecretString,
}
