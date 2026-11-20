//! Request bodies for the companies endpoints.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateCompany {
    pub legal_name: String,
    pub vat_number: String,
    pub street: String,
    pub postal_code: String,
    pub city: String,
    pub country: String,
}
