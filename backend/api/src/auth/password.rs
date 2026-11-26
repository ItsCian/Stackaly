//! Password hashing and verification with Argon2id.
//!
//! Passwords are never stored or logged in plain text: only the PHC string
//! returned by `hash_password` is saved in `users.password_hash`.

use std::sync::LazyLock;

use argon2::{
    Argon2,
    password_hash::{Error, PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

/// Hashes a plain password with Argon2id and returns the PHC string
/// to store in `users.password_hash`. A new random salt is used on every call.
pub fn hash_password(password: &str) -> Result<String, Error> {
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes())?;
    Ok(hash.to_string())
}

/// Checks a plain password against a stored PHC string.
/// Returns `Ok(true)` if it matches, `Ok(false)` if the password is wrong,
/// and `Err` if the stored hash is corrupt.
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, Error> {
    let parsed = PasswordHash::new(stored_hash)?;
    let verifier = Argon2::default();

    match verifier.verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(Error::PasswordInvalid) => Ok(false),
        Err(other) => Err(other),
    }
}

pub static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    hash_password("dummy-password-for-timing").expect("failed to compute the dummy password hash")
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_password_verifies() {
        let hash = hash_password("trattoria123").unwrap();
        assert!(verify_password("trattoria123", &hash).unwrap());
    }

    #[test]
    fn wrong_password_is_false_not_error() {
        let hash = hash_password("trattoria123").unwrap();
        assert!(!verify_password("trattoria124", &hash).unwrap());
    }

    #[test]
    fn same_password_gives_different_hashes() {
        let a = hash_password("trattoria123").unwrap();
        let b = hash_password("trattoria123").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn hash_is_argon2id_phc_string() {
        let hash = hash_password("trattoria123").unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn corrupt_hash_is_an_error() {
        assert!(verify_password("trattoria123", "not-a-phc-string").is_err());
    }
}
