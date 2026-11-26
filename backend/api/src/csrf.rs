//! CSRF protection: blocks state-changing requests sent by another site.
//!
//! Second layer on top of the `SameSite=Strict` session cookie (defense in depth).
//! Applied to the whole API router, so new routes are protected by default.

use axum::{
    extract::Request,
    http::{HeaderMap, Method, header},
    middleware::Next,
    response::Response,
};

use crate::errors::AppError;

/// Middleware: lets safe methods through, and refuses POST/PATCH/DELETE...
/// coming from another site.
pub async fn csrf(request: Request, next: Next) -> Result<Response, AppError> {
    // GET, HEAD and OPTIONS change nothing, so they can't be abused
    if matches!(*request.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return Ok(next.run(request).await);
    }

    if !is_same_site_request(request.headers()) {
        return Err(AppError::Forbidden("Cross-site request blocked".into()));
    }

    Ok(next.run(request).await)
}

/// Decides from the browser's headers whether a request comes from our own site.
fn is_same_site_request(headers: &HeaderMap) -> bool {
    // Modern browsers: Sec-Fetch-Site is set by the browser and can't be forged
    // by a web page. "none" = typed in the address bar or a bookmark.
    if let Some(site) = headers.get("sec-fetch-site").and_then(|v| v.to_str().ok()) {
        return site == "same-origin" || site == "none";
    }

    // Older browsers: compare the Origin's host with the Host header
    match headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) {
        Some(origin) => {
            let origin_host = origin.split_once("://").map(|(_, host)| host);
            let host = headers.get(header::HOST).and_then(|v| v.to_str().ok());
            origin_host.is_some() && origin_host == host
        }
        // No Origin at all: not a browser (curl, tests). CSRF needs a victim's
        // browser, which always sends Origin on a cross-site POST.
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_static(value));
        }
        map
    }

    #[test]
    fn same_origin_is_allowed() {
        assert!(is_same_site_request(&headers(&[("sec-fetch-site", "same-origin")])));
    }

    #[test]
    fn cross_site_is_blocked() {
        assert!(!is_same_site_request(&headers(&[("sec-fetch-site", "cross-site")])));
    }

    #[test]
    fn same_site_subdomain_is_blocked() {
        assert!(!is_same_site_request(&headers(&[("sec-fetch-site", "same-site")])));
    }

    #[test]
    fn matching_origin_is_allowed_without_fetch_metadata() {
        assert!(is_same_site_request(&headers(&[
            ("origin", "http://localhost:5173"),
            ("host", "localhost:5173"),
        ])));
    }

    #[test]
    fn foreign_origin_is_blocked_without_fetch_metadata() {
        assert!(!is_same_site_request(&headers(&[
            ("origin", "https://evil.example"),
            ("host", "localhost:5173"),
        ])));
    }

    #[test]
    fn no_browser_headers_is_allowed() {
        assert!(is_same_site_request(&HeaderMap::new()));
    }
}
