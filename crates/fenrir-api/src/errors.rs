//! Error types for the Fenrir browser
//!
//! This module defines unified error types that can be used across
//! all Fenrir components. All errors must be serializable for IPC.

use serde::{Deserialize, Serialize};

/// Main error type for Fenrir operations
///
/// This is a unified error type that can represent any error that
/// occurs in the Fenrir system. It's designed to be serializable
/// and provide useful error information for debugging and user display.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum FenrirError {
    /// Validation error (invalid input)
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    /// Security error (permission denied, etc.)
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    
    /// Tab-related error
    #[error("Tab error: {0}")]
    Tab(#[from] crate::tabs::TabError),
    
    /// Filesystem error
    #[error("Filesystem error: {0}")]
    Filesystem(#[from] crate::fs::FsError),
    
    /// Permission/access error
    #[error("Access error: {0}")]
    Access(#[from] crate::permissions::AccessError),
    
    /// Event bus error
    #[error("Event error: {0}")]
    Event(#[from] crate::events::EventError),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Network error
    #[error("Network error: {0}")]
    Network(String),
    
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
    
    /// Operation timed out
    #[error("Operation timed out: {0}")]
    Timeout(String),
    
    /// Operation was cancelled
    #[error("Operation cancelled: {0}")]
    Cancelled(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Internal error (bugs, unexpected conditions)
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// External error (from dependencies)
    #[error("External error: {0}")]
    External(String),
}

impl FenrirError {
    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(ValidationError::new(msg))
    }
    
    /// Create a new security error
    pub fn security(msg: impl Into<String>) -> Self {
        Self::Security(SecurityError::new(msg))
    }
    
    /// Create a new internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    
    /// Create a new not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
    
    /// Check if this error is a security error
    pub fn is_security_error(&self) -> bool {
        matches!(self, Self::Security(_) | Self::Access(_))
    }
    
    /// Check if this error is a validation error
    pub fn is_validation_error(&self) -> bool {
        matches!(self, Self::Validation(_))
    }
    
    /// Check if this error is recoverable (the operation could be retried)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Timeout(_) | Self::Cancelled(_) | Self::Network(_)
        )
    }
    
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::Validation(e) => e.to_string(),
            Self::Security(e) => e.to_string(),
            Self::Tab(e) => e.to_string(),
            Self::Filesystem(e) => e.to_string(),
            Self::Access(e) => e.to_string(),
            Self::Event(e) => e.to_string(),
            Self::Serialization(msg) => format!("Data error: {}", msg),
            Self::Network(msg) => format!("Network error: {}", msg),
            Self::NotFound(msg) => format!("Not found: {}", msg),
            Self::AlreadyExists(msg) => format!("Already exists: {}", msg),
            Self::Timeout(msg) => format!("Operation timed out: {}", msg),
            Self::Cancelled(msg) => format!("Operation cancelled: {}", msg),
            Self::Config(msg) => format!("Configuration error: {}", msg),
            Self::Internal(msg) => format!("Internal error: {}", msg),
            Self::External(msg) => format!("External error: {}", msg),
        }
    }
}

/// Result type for Fenrir operations
pub type FenrirResult<T> = Result<T, FenrirError>;

/// Validation error (invalid input data)
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("Validation error: {message}")]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Field that failed validation, if applicable
    pub field: Option<String>,
    /// Expected value or format, if applicable
    pub expected: Option<String>,
    /// Actual value received, if applicable
    pub actual: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
            expected: None,
            actual: None,
        }
    }
    
    /// Create a new validation error with field information
    pub fn with_field(message: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: Some(field.into()),
            expected: None,
            actual: None,
        }
    }
    
    /// Create a new validation error with expected/actual values
    pub fn with_values(
        message: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            field: None,
            expected: Some(expected.into()),
            actual: Some(actual.into()),
        }
    }
    
    /// Add field information to an existing error
    pub fn with_field_mut(&mut self, field: impl Into<String>) -> &mut Self {
        self.field = Some(field.into());
        self
    }
    
    /// Add expected/actual values to an existing error
    pub fn with_values_mut(
        &mut self,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> &mut Self {
        self.expected = Some(expected.into());
        self.actual = Some(actual.into());
        self
    }
}

/// Security error (permission denied, policy violation, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("Security error: {message}")]
pub struct SecurityError {
    /// Error message
    pub message: String,
    /// Security policy that was violated, if applicable
    pub policy: Option<String>,
    /// Resource that was accessed, if applicable
    pub resource: Option<String>,
    /// Suggested action to resolve the error, if applicable
    pub suggestion: Option<String>,
}

impl SecurityError {
    /// Create a new security error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            policy: None,
            resource: None,
            suggestion: None,
        }
    }
    
    /// Create a new security error with policy information
    pub fn with_policy(message: impl Into<String>, policy: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            policy: Some(policy.into()),
            resource: None,
            suggestion: None,
        }
    }
    
    /// Create a new security error with resource information
    pub fn with_resource(message: impl Into<String>, resource: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            policy: None,
            resource: Some(resource.into()),
            suggestion: None,
        }
    }
    
    /// Add policy information to an existing error
    pub fn with_policy_mut(&mut self, policy: impl Into<String>) -> &mut Self {
        self.policy = Some(policy.into());
        self
    }
    
    /// Add resource information to an existing error
    pub fn with_resource_mut(&mut self, resource: impl Into<String>) -> &mut Self {
        self.resource = Some(resource.into());
        self
    }
    
    /// Add a suggestion to an existing error
    pub fn with_suggestion_mut(&mut self, suggestion: impl Into<String>) -> &mut Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Convenience trait for converting errors to FenrirError
pub trait IntoFenrirError {
    /// Convert to FenrirError
    fn into_fenrir_error(self) -> FenrirError;
}

impl IntoFenrirError for std::io::Error {
    fn into_fenrir_error(self) -> FenrirError {
        FenrirError::Filesystem(crate::fs::FsError::from_io_error(self, ""))
    }
}

impl IntoFenrirError for serde_json::Error {
    fn into_fenrir_error(self) -> FenrirError {
        FenrirError::Serialization(self.to_string())
    }
}

impl IntoFenrirError for url::ParseError {
    fn into_fenrir_error(self) -> FenrirError {
        FenrirError::Validation(ValidationError::new(format!("URL parse error: {}", self)))
    }
}
