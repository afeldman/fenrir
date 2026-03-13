//! Origin handling and policies

use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;

/// Represents a web origin (scheme + host + port)
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Origin {
    scheme: String,
    host: String,
    port: Option<u16>,
}

impl Origin {
    /// Create a new origin from a URL
    pub fn from_url(url: &Url) -> Option<Self> {
        let scheme = url.scheme().to_string();
        let host = url.host_str()?.to_string();
        let port = url.port();
        
        Some(Self {
            scheme,
            host,
            port,
        })
    }
    
    /// Create a new origin from components
    pub fn new(scheme: String, host: String, port: Option<u16>) -> Self {
        Self { scheme, host, port }
    }
    
    /// Check if this origin matches a URI
    pub fn matches_uri(&self, uri: &Url) -> bool {
        uri.scheme() == self.scheme &&
        uri.host_str() == Some(&self.host) &&
        uri.port_or_known_default() == self.port
    }
    
    /// Get the scheme
    pub fn scheme(&self) -> &str {
        &self.scheme
    }
    
    /// Get the host
    pub fn host(&self) -> &str {
        &self.host
    }
    
    /// Get the port
    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", self.scheme, self.host)?;
        if let Some(port) = self.port {
            write!(f, ":{}", port)?;
        }
        Ok(())
    }
}

/// Policy for what an origin is allowed to do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginPolicy {
    /// The origin this policy applies to
    pub origin: Origin,
    /// Whether the origin can access file:// URLs
    pub allow_file_access: bool,
    /// Whether the origin can make cross-origin requests
    pub allow_cross_origin: bool,
    /// Specific domains this origin can access
    pub allowed_domains: Vec<String>,
    /// Maximum storage quota in bytes
    pub storage_quota: Option<u64>,
}

impl OriginPolicy {
    /// Create a default restrictive policy
    pub fn default_for(origin: Origin) -> Self {
        Self {
            origin,
            allow_file_access: false,
            allow_cross_origin: false,
            allowed_domains: Vec::new(),
            storage_quota: None,
        }
    }
    
    /// Check if this policy allows access to a URI
    pub fn allows_uri(&self, uri: &Url) -> bool {
        // Check if it's a file:// URL
        if uri.scheme() == "file" {
            return self.allow_file_access;
        }
        
        // Check same-origin
        if self.origin.matches_uri(uri) {
            return true;
        }
        
        // Check cross-origin if allowed
        if self.allow_cross_origin {
            // Check specific domains if specified
            if !self.allowed_domains.is_empty() {
                if let Some(host) = uri.host_str() {
                    return self.allowed_domains.contains(&host.to_string());
                }
            } else {
                // Allow all domains if no specific list
                return true;
            }
        }
        
        false
    }
}
