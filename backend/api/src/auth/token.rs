//! Session token generation and hashing.
//!
//! The raw token only goes in the browser cookie; the database only stores
//! its SHA-256 hash (`sessions.token_hash`).

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

/// Generates a new session token: 32 random bytes from the operating system,
/// encoded as URL-safe base64 without padding (43 characters).
/// This is the value to put in the cookie.
pub fn generate_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

/// Returns the SHA-256 hash of a token, to store in `sessions.token_hash`
/// and to look up the session on every request.
pub fn hash_token(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_tokens_are_different() {
        let a = generate_token().unwrap();
        let b = generate_token().unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn token_is_43_url_safe_chars() {
        let token = generate_token().unwrap();
        assert_eq!(token.len(), 43);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn same_token_gives_same_hash() {
        let token = generate_token().unwrap();
        assert_eq!(hash_token(&token), hash_token(&token));
    }

    #[test]
    fn different_tokens_give_different_hashes() {
        let a = generate_token().unwrap();
        let b = generate_token().unwrap();
        assert_ne!(hash_token(&a), hash_token(&b));
    }
}
