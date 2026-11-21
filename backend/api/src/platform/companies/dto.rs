//! Request bodies for the companies endpoints.

use serde::Deserialize;

/// Body of POST /companies. All fields are required.
#[derive(Debug, Deserialize)]
pub struct CreateCompany {
    pub legal_name: String,
    pub vat_number: String,
    pub street: String,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}

/// Body of PATCH /companies/{id}. Only the fields sent are changed.
#[derive(Debug, Deserialize)]
pub struct UpdateCompany {
    pub legal_name: Option<String>,
    pub vat_number: Option<String>,
    pub street: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
}
