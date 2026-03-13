//! Permission types and traits
//!
//! This module defines the permission system for Fenrir, including:
//! - Access levels for resources
//! - Origin validation and representation
//! - Permission management traits
//! - Path mounting and permission tracking

use crate::errors::ValidationError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Access level for resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessLevel {
    /// No access allowed
    Denied,
    /// Read-only access
    Read,
    /// Write access (implies read)
    Write,
    /// Full access including execution
    Execute,
}

impl AccessLevel {
    /// Check if this access level includes read permission
    pub fn can_read(&self) -> bool {
        matches!(self, AccessLevel::Read | AccessLevel::Write | AccessLevel::Execute)
    }

    /// Check if this access level includes write permission
    pub fn can_write(&self) -> bool {
        matches!(self, AccessLevel::Write | AccessLevel::Execute)
    }

    /// Check if this access level includes execute permission
    pub fn can_execute(&self) -> bool {
        matches!(self, AccessLevel::Execute)
    }
}

impl Default for AccessLevel {
    fn default() -> Self {
        Self::Denied
    }
}

/// Represents a web origin (scheme://host:port)
///
/// # Security
///
/// This type validates that the origin is properly formatted and safe to use.
/// It should be constructed from trusted URLs only.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Origin {
    /// The scheme (http, https, file, etc.)
    pub scheme: String,
    /// The host (domain or IP address)
    pub host: String,
    /// The port, if specified
    pub port: Option<u16>,
}

impl Origin {
    /// Create a new origin from its components
    ///
    /// # Arguments
    ///
    /// * `scheme` - The URL scheme (must be non-empty)
    /// * `host` - The host (must be non-empty)
    /// * `port` - Optional port number
    ///
    /// # Returns
    ///
    /// Returns `Ok(Origin)` if the components are valid, `Err(ValidationError)` otherwise.
    pub fn new(scheme: String, host: String, port: Option<u16>) -> Result<Self, ValidationError> {
        if scheme.is_empty() {
            return Err(ValidationError::new("Scheme cannot be empty"));
        }
        if host.is_empty() {
            return Err(ValidationError::new("Host cannot be empty"));
        }

        Ok(Self { scheme, host, port })
    }

    /// Create an origin from a URL string
    ///
    /// # Security
    ///
    /// This method validates the URL and extracts the origin components.
    /// Only call this with trusted URLs.
    pub fn from_url(url: &str) -> Result<Self, ValidationError> {
        // Parse the URL and extract origin components
        // This is a simplified version - real implementation would use a URL parser
        if url.starts_with("http://") {
            let rest = &url[7..];
            let (host, port) = Self::parse_host_port(rest);
            Self::new("http".to_string(), host.to_string(), port)
        } else if url.starts_with("https://") {
            let rest = &url[8..];
            let (host, port) = Self::parse_host_port(rest);
            Self::new("https".to_string(), host.to_string(), port)
        } else if url.starts_with("file://") {
            // File URLs have special handling
            Self::new("file".to_string(), "".to_string(), None)
        } else {
            Err(ValidationError::new("Unsupported URL scheme"))
        }
    }

    fn parse_host_port(s: &str) -> (&str, Option<u16>) {
        // Simplified parsing - real implementation would be more robust
        if let Some(colon_pos) = s.find(':') {
            let host = &s[..colon_pos];
            let port_str = &s[colon_pos + 1..];
            if let Ok(port) = port_str.parse::<u16>() {
                (host, Some(port))
            } else {
                (s, None)
            }
        } else if let Some(slash_pos) = s.find('/') {
            (&s[..slash_pos], None)
        } else {
            (s, None)
        }
    }

    /// Check if this origin is a file origin
    pub fn is_file_origin(&self) -> bool {
        self.scheme == "file"
    }

    /// Get the origin as a string (scheme://host:port)
    pub fn as_string(&self) -> String {
        if let Some(port) = self.port {
            format!("{}://{}:{}", self.scheme, self.host, port)
        } else {
            format!("{}://{}", self.scheme, self.host)
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Represents a mounted path with its permissions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MountedPath {
    /// The canonical path that is mounted
    pub path: String,
    /// The virtual path in the sandbox
    pub virtual_path: String,
    /// Default access level for this mount
    pub default_access: AccessLevel,
    /// Whether this mount is read-only in the sandbox
    pub read_only: bool,
}

/// Represents a specific permission for a path
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathPermission {
    /// The path this permission applies to
    pub path: String,
    /// The access level granted
    pub access: AccessLevel,
    /// The origin that has this permission
    pub origin: Origin,
    /// When this permission was granted
    pub granted_at: std::time::SystemTime,
}

/// Trait for managing permissions
///
/// # Implementation Requirements
///
/// Implementations must be thread-safe (`Send + Sync`) and should handle
/// concurrent access properly.
pub trait PermissionManager: Send + Sync {
    /// Check if a tab has access to a path with the requested access level
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab ID requesting access
    /// * `path` - The canonical path to check
    /// * `access` - The requested access level
    ///
    /// # Returns
    ///
    /// `true` if access is granted, `false` otherwise.
    fn check_access(&self, tab: &crate::tabs::TabId, path: &crate::fs::CanonicalPath, access: AccessLevel) -> bool;
    
    /// Grant access to a tab for a specific path
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab ID to grant access to
    /// * `path` - The canonical path to grant access to
    /// * `access` - The access level to grant
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(AccessError)` on failure.
    fn grant_access(&self, tab: &crate::tabs::TabId, path: crate::fs::CanonicalPath, access: AccessLevel) -> Result<(), AccessError>;
    
    /// Revoke access from a tab for a specific path
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab ID to revoke access from
    /// * `path` - The canonical path to revoke access from
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(AccessError)` on failure.
    fn revoke_access(&self, tab: &crate::tabs::TabId, path: &crate::fs::CanonicalPath) -> Result<(), AccessError>;
    
    /// Get all permissions for a specific tab
    ///
    /// # Arguments
    ///
    /// * `tab` - The tab ID to get permissions for
    ///
    /// # Returns
    ///
    /// A vector of path permissions for the tab.
    fn get_tab_permissions(&self, tab: &crate::tabs::TabId) -> Vec<PathPermission>;
    
    /// Get all permissions for a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to get permissions for
    ///
    /// # Returns
    ///
    /// A vector of path permissions for the path.
    fn get_path_permissions(&self, path: &crate::fs::CanonicalPath) -> Vec<PathPermission>;
}

/// Trait for managing permission registries
///
/// This trait is for systems that need to track and query permissions
/// across multiple tabs and paths.
pub trait PermissionRegistry: Send + Sync {
    /// Register a new permission
    fn register_permission(&self, permission: PathPermission) -> Result<(), AccessError>;
    
    /// Unregister a permission
    fn unregister_permission(&self, permission: &PathPermission) -> Result<(), AccessError>;
    
    /// Find permissions by origin
    fn find_by_origin(&self, origin: &Origin) -> Vec<PathPermission>;
    
    /// Find permissions by path prefix
    fn find_by_path_prefix(&self, prefix: &str) -> Vec<PathPermission>;
    
    /// Clear all permissions for a tab
    fn clear_tab_permissions(&self, tab: &crate::tabs::TabId) -> Result<(), AccessError>;
}

/// Error type for access-related operations
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum AccessError {
    /// The path is not accessible
    #[error("Path not accessible: {0}")]
    PathNotAccessible(String),
    
    /// Insufficient permissions
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    /// Permission already exists
    #[error("Permission already exists")]
    PermissionExists,
    
    /// Permission not found
    #[error("Permission not found")]
    PermissionNotFound,
    
    /// Invalid access level
    #[error("Invalid access level")]
    InvalidAccessLevel,
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AccessError {
    /// Create a new access error with a message
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
