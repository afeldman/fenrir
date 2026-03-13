# Fenrir API Usage Examples

This document shows how to use the `fenrir-api` crate in other Fenrir components.

## Basic Import

```rust
// Import commonly used types
use fenrir_api::prelude::*;

// Or import specific modules
use fenrir_api::{TabId, PermissionManager, TabRegistry, CanonicalPath};
```

## Example 1: Implementing PermissionManager in fenrir-secure

```rust
// In crates/fenrir-secure/src/lib.rs or similar

use fenrir_api::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct SecurePermissionManager {
    permissions: Arc<RwLock<HashMap<TabId, Vec<PathPermission>>>>,
}

impl SecurePermissionManager {
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl PermissionManager for SecurePermissionManager {
    fn check_access(&self, tab: &TabId, path: &CanonicalPath, access: AccessLevel) -> bool {
        let permissions = self.permissions.read().unwrap();
        if let Some(tab_perms) = permissions.get(tab) {
            for perm in tab_perms {
                if perm.path == path.as_str() && perm.access >= access {
                    return true;
                }
            }
        }
        false
    }
    
    fn grant_access(&self, tab: &TabId, path: CanonicalPath, access: AccessLevel) -> Result<(), AccessError> {
        let mut permissions = self.permissions.write().unwrap();
        let tab_perms = permissions.entry(*tab).or_insert_with(Vec::new);
        
        // Check if permission already exists
        if tab_perms.iter().any(|p| p.path == path.as_str()) {
            return Err(AccessError::PermissionExists);
        }
        
        let permission = PathPermission {
            path: path.as_str().to_string(),
            access,
            origin: Origin::new("file".to_string(), "".to_string(), None).unwrap(), // Example
            granted_at: std::time::SystemTime::now(),
        };
        
        tab_perms.push(permission);
        Ok(())
    }
    
    // ... implement other methods
}
```

## Example 2: Using TabRegistry in fenrir-sandbox

```rust
// In crates/fenrir-sandbox/src/lib.rs

use fenrir_api::prelude::*;
use std::sync::{Arc, RwLock};

pub struct SandboxTabRegistry {
    tabs: Arc<RwLock<HashMap<TabId, TabInfo>>>,
}

impl TabRegistry for SandboxTabRegistry {
    fn register(&self, info: TabInfo) -> TabId {
        let mut tabs = self.tabs.write().unwrap();
        let id = info.id;
        tabs.insert(id, info);
        id
    }
    
    fn unregister(&self, id: &TabId) {
        let mut tabs = self.tabs.write().unwrap();
        tabs.remove(id);
    }
    
    fn get_info(&self, id: &TabId) -> Option<TabInfo> {
        let tabs = self.tabs.read().unwrap();
        tabs.get(id).cloned()
    }
    
    // ... implement other methods
}

// Function to create a new tab
pub fn create_tab(registry: &impl TabRegistry, url: String) -> Result<TabId, TabError> {
    let origin = Origin::from_url(&url)
        .map_err(|e| TabError::InvalidUrl(e.to_string()))?;
    
    let info = TabInfo::new(url, origin);
    let id = registry.register(info);
    
    // ... create Tauri window, Servo instance, etc.
    
    Ok(id)
}
```

## Example 3: Using Events in fenrir-ui

```rust
// In crates/fenrir-ui/src/lib.rs

use fenrir_api::prelude::*;

pub struct UiEventHandler {
    event_bus: Arc<dyn EventBus>,
}

impl UiEventHandler {
    pub fn new(event_bus: Arc<dyn EventBus>) -> Self {
        Self { event_bus }
    }
    
    pub fn handle_tab_created(&self, tab_id: TabId, url: String) {
        let event = Event::Tab(TabEvent::Created { tab_id, url });
        let _ = self.event_bus.publish(event);
    }
    
    pub fn handle_user_navigation(&self, tab_id: TabId, url: String) {
        let event = Event::Ui(UiEvent::NavigateRequested { tab_id, url });
        let _ = self.event_bus.publish(event);
    }
}

// Subscribe to events
pub fn setup_event_listeners(event_bus: Arc<dyn EventBus>) -> SubscriptionId {
    event_bus.subscribe(|event| {
        match event {
            Event::Tab(TabEvent::TitleChanged { tab_id, title }) => {
                println!("Tab {} title changed to: {}", tab_id, title);
                // Update UI title
            }
            Event::Tab(TabEvent::UrlChanged { tab_id, url, .. }) => {
                println!("Tab {} navigated to: {}", tab_id, url);
                // Update URL bar
            }
            Event::Permission(PermissionEvent::Requested { tab_id, permission_type, resource, .. }) => {
                println!("Permission requested by tab {}: {} for {}", 
                         tab_id, permission_type, resource);
                // Show permission prompt
            }
            _ => {}
        }
    })
}
```

## Example 4: Error Handling

```rust
use fenrir_api::prelude::*;

fn process_file_access(
    permission_manager: &impl PermissionManager,
    tab_id: TabId,
    path: CanonicalPath,
) -> FenrirResult<()> {
    // Check permissions
    if !permission_manager.check_access(&tab_id, &path, AccessLevel::Read) {
        return Err(FenrirError::security(
            format!("Tab {} cannot read {}", tab_id, path)
        ));
    }
    
    // Perform file operation
    // ...
    
    Ok(())
}

fn handle_api_error(error: FenrirError) {
    match error {
        FenrirError::Security(e) => {
            eprintln!("Security violation: {}", e);
            // Log security event
        }
        FenrirError::Validation(e) => {
            eprintln!("Invalid input: {}", e);
            // Show user error message
        }
        FenrirError::Tab(e) => {
            eprintln!("Tab error: {}", e);
            // Handle tab-specific error
        }
        _ => {
            eprintln!("Unexpected error: {}", error);
            // Generic error handling
        }
    }
}
```

## Example 5: Filesystem Operations

```rust
use fenrir_api::prelude::*;

struct SecureFileSystem {
    permission_manager: Arc<dyn PermissionManager>,
    // ... other fields
}

impl FileSystem for SecureFileSystem {
    fn read_file(
        &self,
        tab: &TabId,
        path: &CanonicalPath,
    ) -> Result<ReadResult, FsError> {
        // Check permissions first
        if !self.permission_manager.check_access(tab, path, AccessLevel::Read) {
            return Err(FsError::PermissionDenied(
                format!("Tab {} cannot read {}", tab, path)
            ));
        }
        
        // Actually read the file
        // ...
        
        Ok(ReadResult {
            data: vec![],
            mime_type: None,
            size: 0,
            modified: None,
        })
    }
    
    // ... implement other methods
}
```

## Testing Your Implementations

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use fenrir_api::prelude::*;
    
    #[test]
    fn test_permission_manager_implementation() {
        let manager = SecurePermissionManager::new();
        let tab_id = TabId::from_raw(1);
        let path = CanonicalPath::new("/test/file.txt").unwrap();
        
        // Initially no access
        assert!(!manager.check_access(&tab_id, &path, AccessLevel::Read));
        
        // Grant access
        manager.grant_access(&tab_id, path.clone(), AccessLevel::Read)
            .expect("Should grant access");
        
        // Now should have access
        assert!(manager.check_access(&tab_id, &path, AccessLevel::Read));
        assert!(!manager.check_access(&tab_id, &path, AccessLevel::Write));
    }
}
```

## Key Points

1. **Always use `fenrir-api` types** for public interfaces between crates
2. **Implement traits** in your crates, don't add methods to API types
3. **Use `Send + Sync`** for all trait implementations (required by the API)
4. **Serialize everything** - all types support `serde` for IPC
5. **Handle errors properly** - use `FenrirError` and related error types
