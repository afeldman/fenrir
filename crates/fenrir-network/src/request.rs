//! NetworkRequest / NetworkResponse — Fenrir's interne Request-Typen.
//!
//! Unabhängig von reqwest/Servo-Typen, damit die Pipeline tauschbar bleibt.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
    Other(String),
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Patch => write!(f, "PATCH"),
            Self::Other(m) => write!(f, "{}", m),
        }
    }
}

/// Fenrir-interner HTTP Request.
#[derive(Debug, Clone)]
pub struct NetworkRequest {
    pub url: Url,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    /// Kommt der Request von einem Tab (true) oder intern (false)?
    pub is_main_frame: bool,
}

impl NetworkRequest {
    pub fn get(url: Url) -> Self {
        Self {
            url,
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
            is_main_frame: false,
        }
    }

    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    pub fn host(&self) -> Option<&str> {
        self.url.host_str()
    }
}

/// Fenrir-interne HTTP Response.
#[derive(Debug)]
pub struct NetworkResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub url: Url,
}

impl NetworkResponse {
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_get_sets_method() {
        let url = Url::parse("https://example.com").unwrap();
        let req = NetworkRequest::get(url.clone());
        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.url, url);
        assert!(req.body.is_none());
    }

    #[test]
    fn request_scheme_and_host() {
        let url = Url::parse("https://example.com/path").unwrap();
        let req = NetworkRequest::get(url);
        assert_eq!(req.scheme(), "https");
        assert_eq!(req.host(), Some("example.com"));
    }

    #[test]
    fn response_success_range() {
        let url = Url::parse("https://example.com").unwrap();
        let ok = NetworkResponse { status: 200, headers: Default::default(), body: vec![], url: url.clone() };
        let err = NetworkResponse { status: 404, headers: Default::default(), body: vec![], url };
        assert!(ok.is_success());
        assert!(!err.is_success());
    }
}
