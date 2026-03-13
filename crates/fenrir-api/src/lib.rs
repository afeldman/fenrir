//! Fenrir Public API
//!
//! # Overview
//!
//! This crate defines ALL public types and traits used by other Fenrir crates.
//! It contains NO implementations - only type definitions and trait declarations.
//!
//! ## Design Principles
//!
//! 1. **Implementation-free**: This crate must not contain any actual implementations
//! 2. **Minimal dependencies**: Only serde and thiserror are allowed
//! 3. **IPC-ready**: All types must be serializable for Tauri IPC
//! 4. **Thread-safe**: All traits require `Send + Sync` for cross-thread usage
//!
//! ## Usage
//!
//! Other crates should import from this crate:
//!
//! ```rust
//! use fenrir_api::prelude::*;
//! use fenrir_api::{TabId, PermissionManager, TabRegistry};
//! ```
//!
//! Implementations go in other crates:
//!
//! ```rust
//! // In fenrir-secure
//! use fenrir_api::PermissionManager;
//!
//! pub struct SecurePermissionManager { /* ... */ }
//!
//! impl PermissionManager for SecurePermissionManager {
//!     // Implementation here
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod permissions;
pub mod tabs;
pub mod fs;
pub mod events;
pub mod errors;
pub mod prelude;

// Re-exports of the most commonly used types
pub use permissions::*;
pub use tabs::*;
pub use fs::*;
pub use events::*;
pub use errors::*;

/// Convenience module for easy imports
pub mod api {
    pub use super::permissions::*;
    pub use super::tabs::*;
    pub use super::fs::*;
    pub use super::events::*;
    pub use super::errors::*;
}

#[cfg(test)]
mod tests;
