//! Belgian VAT number: normalize + validate (format and mod-97 checksum).
//!
//! Belgium only for the MVP. A Belgian VAT number is "BE" followed by
//! 10 digits (the company's enterprise number). The first digit is 0 or 1,
//! and the last 2 digits are a checksum of the first 8:
//!
//! ```text
//! check = 97 - (first_8_digits % 97)
//! ```
//!
//! Example: BE 0403.219.991 -> first 8 = 04032199, 4032199 % 97 = 6,
//! 97 - 6 = 91 -> last 2 digits must be "91". Valid.

use crate::errors::AppError;

/// Cleans up and validates a Belgian VAT number.
///
/// "be 0403.219.991" -> Ok("BE0403219991")
///
/// The returned value is the ONLY form that should be stored in
/// `companies.vat_number`, so the UNIQUE constraint catches duplicates
/// whatever way the user typed the number.
pub fn normalize_vat(input: &str) -> Result<String, AppError> {
    // 1. Clean: keep ASCII letters/digits only (drops spaces, dots, dashes
    //    and any non-ASCII character), and uppercase ("be" -> "BE").
    let vat: String = input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    // 2. Prefix: strip_prefix returns Some(rest) only if "BE" is there,
    //    so we get the digit part without slicing by hand (no panic risk).
    let Some(digits) = vat.strip_prefix("BE") else {
        return Err(invalid());
    };

    // 3. Exactly 10 characters, all digits. Checked BEFORE any slicing below,
    //    so `digits[..8]` can never go out of bounds.
    //    `.len()` counts bytes, which is fine here: we only kept ASCII.
    if digits.len() != 10 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(invalid());
    }

    // 4. First digit is 0 or 1 (Belgian enterprise numbers).
    if !digits.starts_with(['0', '1']) {
        return Err(invalid());
    }

    // 5. Checksum. u64 because 8 digits (up to 19_999_999) don't fit
    //    comfortably in smaller types and we never want an overflow.
    //    unwrap() is safe: step 3 guaranteed only ASCII digits.
    let base: u64 = digits[..8].parse().unwrap();
    let check: u64 = digits[8..].parse().unwrap();

    if 97 - (base % 97) != check {
        return Err(invalid());
    }

    Ok(vat)
}

/// One message for every failure: the user just needs to know the number
/// is wrong, and the front end can show it next to the VAT field.
fn invalid() -> AppError {
    AppError::Validation("Invalid Belgian VAT number (expected format: BE0123456789)".into())
}

#[cfg(test)] // compiled only for `cargo test`
mod tests {
    use super::*; // brings normalize_vat into scope

    #[test]
    fn accepts_clean_number() {
        assert_eq!(normalize_vat("BE0403219991").unwrap(), "BE0403219991");
    }

    #[test]
    fn accepts_formatted_lowercase_number() {
        // spaces, dots, lowercase prefix: all normalized away
        assert_eq!(normalize_vat("be 0403.219.991").unwrap(), "BE0403219991");
    }

    #[test]
    fn accepts_checksum_of_97() {
        // base % 97 == 0 -> check is 97, not 0 (edge case of the formula)
        assert_eq!(normalize_vat("BE0000000097").unwrap(), "BE0000000097");
    }

    #[test]
    fn rejects_wrong_checksum() {
        assert!(normalize_vat("BE0403219992").is_err());
    }

    #[test]
    fn rejects_missing_prefix() {
        assert!(normalize_vat("0403219991").is_err());
    }

    #[test]
    fn rejects_other_country() {
        assert!(normalize_vat("FR0403219991").is_err());
    }

    #[test]
    fn rejects_too_short_and_too_long() {
        assert!(normalize_vat("BE040321999").is_err()); // 9 digits
        assert!(normalize_vat("BE04032199910").is_err()); // 11 digits
    }

    #[test]
    fn rejects_bad_first_digit() {
        assert!(normalize_vat("BE2403219991").is_err());
    }

    #[test]
    fn rejects_non_ascii_digits() {
        // Arabic-Indic digit ٣: would pass is_alphanumeric(), not is_ascii_alphanumeric()
        assert!(normalize_vat("BE040321999٣").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(normalize_vat("").is_err());
    }
}
