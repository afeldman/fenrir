//! Permission management

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Types of permissions that can be granted
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Permission {
    /// Read files from local filesystem
    FileRead,
    /// Write files to local filesystem
    FileWrite,
    /// Access camera
    Camera,
    /// Access microphone
    Microphone,
    /// Access geolocation
    Geolocation,
    /// Send notifications
    Notifications,
    /// Access clipboard
    Clipboard,
    /// Access persistent storage
    PersistentStorage,
    /// Access to specific APIs
    ApiAccess(String),
}

/// A set of permissions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set
    pub fn new() -> Self {
        Self {
            permissions: HashSet::new(),
        }
    }
    
    /// Add a permission
    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }
    
    /// Remove a permission
    pub fn remove(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }
    
    /// Check if a permission is granted
    pub fn has(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }
    
    /// Check if all permissions in a set are granted
    pub fn has_all(&self, permissions: &PermissionSet) -> bool {
        permissions.permissions.iter().all(|p| self.has(p))
    }
    
    /// Check if any permission in a set is granted
    pub fn has_any(&self, permissions: &PermissionSet) -> bool {
        permissions.permissions.iter().any(|p| self.has(p))
    }
    
    /// Get all permissions
    pub fn iter(&self) -> impl Iterator<Item = &Permission> {
        self.permissions.iter()
    }
}

/// Result of a permission request
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PermissionGrant {
    /// Permission granted
    Granted,
    /// Permission denied
    Denied,
    /// Permission granted temporarily (for this session only)
    Temporary,
}

impl PermissionGrant {
    /// Check if permission is granted (including temporary)
    pub fn is_granted(&self) -> bool {
        matches!(self, PermissionGrant::Granted | PermissionGrant::Temporary)
    }
}
