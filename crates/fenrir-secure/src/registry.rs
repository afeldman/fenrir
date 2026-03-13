//! Registry of active Servo instances

use crate::origin::Origin;
use parking_lot::RwLock;
use servo::Servo;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// Information about a registered Servo instance
#[derive(Clone)]
pub struct InstanceInfo {
    /// Unique identifier for the instance
    pub id: String,
    /// Origin of the instance
    pub origin: Origin,
    /// Reference to the Servo instance
    pub servo: Arc<Servo>,
    /// Timestamp when the instance was created
    pub created_at: std::time::Instant,
}

impl std::fmt::Debug for InstanceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InstanceInfo")
            .field("id", &self.id)
            .field("origin", &self.origin)
            .field("created_at", &self.created_at)
            .finish_non_exhaustive()
    }
}

/// Registry of all active Servo instances
pub struct ServoInstanceRegistry {
    instances: RwLock<HashMap<String, InstanceInfo>>,
}

impl ServoInstanceRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new Servo instance
    pub fn register_instance(&self, id: String, origin: Origin, servo: Arc<Servo>) {
        let info = InstanceInfo {
            id: id.clone(),
            origin,
            servo,
            created_at: std::time::Instant::now(),
        };
        
        self.instances.write().insert(id.clone(), info);
        info!("Registered Servo instance {} with origin", id);
    }
    
    /// Unregister a Servo instance
    pub fn unregister_instance(&self, id: &str) {
        if self.instances.write().remove(id).is_some() {
            info!("Unregistered Servo instance {}", id);
        }
    }
    
    /// Get information about a specific instance
    pub fn get_instance_info(&self, id: &str) -> Option<InstanceInfo> {
        self.instances.read().get(id).cloned()
    }
    
    /// Get all instances
    pub fn get_all_instances(&self) -> Vec<InstanceInfo> {
        self.instances.read().values().cloned().collect()
    }
    
    /// Find instances by origin
    pub fn find_instances_by_origin(&self, origin: &Origin) -> Vec<InstanceInfo> {
        self.instances
            .read()
            .values()
            .filter(|info| &info.origin == origin)
            .cloned()
            .collect()
    }
    
    /// Check if an instance exists
    pub fn has_instance(&self, id: &str) -> bool {
        self.instances.read().contains_key(id)
    }
    
    /// Get the number of active instances
    pub fn instance_count(&self) -> usize {
        self.instances.read().len()
    }
}
