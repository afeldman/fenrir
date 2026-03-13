//! fenrir-secure — Core security and permission system for Fenrir Browser, SERVO-AWARE.
//!
//! This crate provides:
//! - Origin-based permission management compatible with Servo's existing permission system
//! - Path canonicalization and validation
//! - Central registry of active Servo instances with their origins and granted permissions
//! - Integration with Servo's embedder APIs for permission delegation

mod origin;
mod path;
mod permissions;
mod registry;

pub use origin::{Origin, OriginPolicy};
pub use path::{canonicalize_path, validate_file, PathError};
pub use permissions::{Permission, PermissionSet, PermissionGrant};
pub use registry::{ServoInstanceRegistry, InstanceInfo};

use servo::PermissionRequest;
use servo::Servo;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;
use url::Url;

/// Main security manager that coordinates with Servo's permission system
pub struct SecurityManager {
    registry: Arc<ServoInstanceRegistry>,
    origin_policies: HashMap<Origin, OriginPolicy>,
    mounted_paths: HashMap<Origin, Vec<PathBuf>>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            registry: Arc::new(ServoInstanceRegistry::new()),
            origin_policies: HashMap::new(),
            mounted_paths: HashMap::new(),
        }
    }

    /// Register a new Servo instance with its origin
    pub fn register_servo_instance(&self, instance_id: String, origin: Origin, servo: Arc<Servo>) {
        self.registry.register_instance(instance_id, origin, servo);
    }

    /// Unregister a Servo instance
    pub fn unregister_servo_instance(&self, instance_id: &str) {
        self.registry.unregister_instance(instance_id);
    }

    /// Mount a local directory for a specific origin
    pub fn mount_directory(&mut self, origin: Origin, path: PathBuf) -> Result<(), SecurityError> {
        // Validate and canonicalize the path
        let canonical_path = canonicalize_path(&path)?;
        
        // Check if the origin already has this path mounted
        let paths = self.mounted_paths.entry(origin.clone()).or_insert_with(Vec::new);
        if !paths.contains(&canonical_path) {
            paths.push(canonical_path);
            info!("Mounted directory {:?} for origin {}", path, origin);
        }
        
        Ok(())
    }

    /// Unmount a directory from an origin
    pub fn unmount_directory(&mut self, origin: &Origin, path: &PathBuf) -> Result<(), SecurityError> {
        if let Some(paths) = self.mounted_paths.get_mut(origin) {
            if let Some(pos) = paths.iter().position(|p| p == path) {
                paths.remove(pos);
                info!("Unmounted directory {:?} from origin {}", path, origin);
                if paths.is_empty() {
                    self.mounted_paths.remove(origin);
                }
                return Ok(());
            }
        }
        
        Err(SecurityError::PathNotMounted(path.clone()))
    }

    /// Check if an origin has permission to access a path
    pub fn check_path_permission(&self, origin: &Origin, requested_path: &PathBuf) -> Result<PathBuf, SecurityError> {
        // First canonicalize the requested path
        let canonical_requested = canonicalize_path(requested_path)?;
        
        // Check if the origin has any mounted paths
        if let Some(mounted_paths) = self.mounted_paths.get(origin) {
            // Check if the requested path is within any mounted directory
            for mounted_path in mounted_paths {
                if canonical_requested.starts_with(mounted_path) {
                    return Ok(canonical_requested);
                }
            }
        }
        
        Err(SecurityError::PermissionDenied {
            origin: origin.clone(),
            path: canonical_requested,
        })
    }

    /// Handle a permission request from Servo
    pub fn handle_servo_permission_request(
        &self,
        request: PermissionRequest,
        instance_id: &str,
    ) -> Result<PermissionGrant, SecurityError> {
        // Get the origin for this instance
        let origin = self.registry
            .get_instance_info(instance_id)
            .ok_or(SecurityError::InstanceNotFound(instance_id.to_string()))?
            .origin
            .clone();
        
        // Check if the origin has the required permission
        // This integrates with Servo's permission delegation system
        // For now, we'll implement a simple permission check based on the feature
        match request.feature() {
            _ => {
                // Default implementation: deny by default for security
                // Can be extended based on origin policies
                Ok(PermissionGrant::Denied)
            }
        }
    }

    /// Set an origin policy
    pub fn set_origin_policy(&mut self, origin: Origin, policy: OriginPolicy) {
        self.origin_policies.insert(origin, policy);
    }

    /// Get an origin policy
    pub fn get_origin_policy(&self, origin: &Origin) -> Option<&OriginPolicy> {
        self.origin_policies.get(origin)
    }

    /// Check if an origin is allowed to access a URI based on its policy
    fn check_origin_policy(&self, origin: &Origin, uri: &Url) -> bool {
        if let Some(policy) = self.origin_policies.get(origin) {
            policy.allows_uri(uri)
        } else {
            // Default policy: only allow same-origin
            origin.matches_uri(uri)
        }
    }

    /// Check default permissions for other request types
    fn check_default_permission(&self, _origin: &Origin, _request: &PermissionRequest) -> bool {
        // Default implementation: deny by default for security
        // Can be extended based on origin policies
        false
    }
}

/// Security errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Path error: {0}")]
    Path(#[from] PathError),
    
    #[error("Permission denied for origin {origin} to access path {path:?}")]
    PermissionDenied {
        origin: Origin,
        path: PathBuf,
    },
    
    #[error("Path not mounted: {0:?}")]
    PathNotMounted(PathBuf),
    
    #[error("Servo instance not found: {0}")]
    InstanceNotFound(String),
    
    #[error("Invalid origin: {0}")]
    InvalidOrigin(String),
}

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;
