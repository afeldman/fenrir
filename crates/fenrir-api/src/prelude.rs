//! Prelude module for convenient imports
//!
//! This module re-exports the most commonly used types and traits
//! from the Fenrir API. Import it with:
//!
//! ```rust
//! use fenrir_api::prelude::*;
//! ```

pub use crate::permissions::{
    AccessLevel, AccessError, Origin, PermissionManager, 
    PermissionRegistry, MountedPath, PathPermission
};

pub use crate::tabs::{
    TabId, TabInfo, TabState, TabRegistry, TabManager
};

pub use crate::fs::{
    CanonicalPath, FileAccess, FileSystem, ReadResult, WriteResult
};

pub use crate::events::{
    TabEvent, UiEvent, PermissionEvent, SystemEvent, EventBus
};

pub use crate::errors::{
    FenrirError, FenrirResult, ValidationError, SecurityError
};
