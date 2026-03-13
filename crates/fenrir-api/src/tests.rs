//! Tests for fenrir-api types and traits
//!
//! These tests verify that:
//! 1. All types can be serialized and deserialized
//! 2. Type relationships are correct
//! 3. Traits have the expected signatures

use crate::prelude::*;
use serde_json;

#[test]
fn test_access_level_serialization() {
    let levels = [
        AccessLevel::Denied,
        AccessLevel::Read,
        AccessLevel::Write,
        AccessLevel::Execute,
    ];
    
    for level in levels {
        let serialized = serde_json::to_string(&level).unwrap();
        let deserialized: AccessLevel = serde_json::from_str(&serialized).unwrap();
        assert_eq!(level, deserialized);
    }
}

#[test]
fn test_access_level_permissions() {
    assert!(!AccessLevel::Denied.can_read());
    assert!(AccessLevel::Read.can_read());
    assert!(AccessLevel::Write.can_read());
    assert!(AccessLevel::Execute.can_read());
    
    assert!(!AccessLevel::Denied.can_write());
    assert!(!AccessLevel::Read.can_write());
    assert!(AccessLevel::Write.can_write());
    assert!(AccessLevel::Execute.can_write());
    
    assert!(!AccessLevel::Denied.can_execute());
    assert!(!AccessLevel::Read.can_execute());
    assert!(!AccessLevel::Write.can_execute());
    assert!(AccessLevel::Execute.can_execute());
}

#[test]
fn test_origin_creation() {
    let origin = Origin::new(
        "https".to_string(),
        "example.com".to_string(),
        Some(443),
    ).unwrap();
    
    assert_eq!(origin.scheme, "https");
    assert_eq!(origin.host, "example.com");
    assert_eq!(origin.port, Some(443));
    assert!(!origin.is_file_origin());
    assert_eq!(origin.as_string(), "https://example.com:443");
}

#[test]
fn test_origin_validation() {
    // Empty scheme should fail
    assert!(Origin::new("".to_string(), "example.com".to_string(), None).is_err());
    
    // Empty host should fail
    assert!(Origin::new("https".to_string(), "".to_string(), None).is_err());
    
    // Valid origin should succeed
    assert!(Origin::new("http".to_string(), "localhost".to_string(), Some(8080)).is_ok());
}

#[test]
fn test_tab_id_serialization() {
    let tab_id = TabId::from_raw(12345);
    let serialized = serde_json::to_string(&tab_id).unwrap();
    let deserialized: TabId = serde_json::from_str(&serialized).unwrap();
    assert_eq!(tab_id.as_raw(), deserialized.as_raw());
}

#[test]
fn test_tab_info_creation() {
    let origin = Origin::new(
        "https".to_string(),
        "example.com".to_string(),
        Some(443),
    ).unwrap();
    
    let tab_info = TabInfo::new(
        "https://example.com".to_string(),
        origin.clone(),
    );
    
    assert_eq!(tab_info.url, "https://example.com");
    assert_eq!(tab_info.origin, origin);
    assert_eq!(tab_info.state, TabState::Loading);
    assert!(!tab_info.is_active);
    assert!(!tab_info.is_pinned);
}

#[test]
fn test_canonical_path_validation() {
    // Valid paths
    assert!(CanonicalPath::new("/home/user/file.txt").is_ok());
    assert!(CanonicalPath::new("C:\\Users\\file.txt").is_ok());
    
    // Invalid paths
    assert!(CanonicalPath::new("").is_err());
    assert!(CanonicalPath::new("/home/../file.txt").is_err());
    assert!(CanonicalPath::new("/home//file.txt").is_err());
}

#[test]
fn test_canonical_path_operations() {
    let path = CanonicalPath::new("/home/user").unwrap();
    
    // Join
    let joined = path.join("file.txt").unwrap();
    assert_eq!(joined.as_str(), "/home/user/file.txt");
    
    // Parent
    let parent = path.parent().unwrap();
    assert_eq!(parent.as_str(), "/home");
    
    // Display
    assert_eq!(path.to_string(), "/home/user");
}

#[test]
fn test_event_serialization() {
    let tab_event = Event::Tab(TabEvent::Created {
        tab_id: TabId::from_raw(1),
        url: "https://example.com".to_string(),
    });
    
    let serialized = serde_json::to_string(&tab_event).unwrap();
    let deserialized: Event = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        Event::Tab(TabEvent::Created { tab_id, url }) => {
            assert_eq!(tab_id.as_raw(), 1);
            assert_eq!(url, "https://example.com");
        }
        _ => panic!("Wrong event type deserialized"),
    }
}

#[test]
fn test_error_serialization() {
    let error = FenrirError::validation("Test validation error");
    let serialized = serde_json::to_string(&error).unwrap();
    let deserialized: FenrirError = serde_json::from_str(&serialized).unwrap();
    
    match deserialized {
        FenrirError::Validation(e) => {
            assert_eq!(e.message, "Test validation error");
        }
        _ => panic!("Wrong error type deserialized"),
    }
}

#[test]
fn test_prelude_imports() {
    // This test just verifies that the prelude module compiles
    // and exports the expected types
    use crate::prelude::*;
    
    // These should all compile
    let _: AccessLevel = AccessLevel::Read;
    let _: Origin = Origin::new("https".to_string(), "example.com".to_string(), None).unwrap();
    let _: TabId = TabId::from_raw(1);
    let _: TabInfo = TabInfo::new(
        "https://example.com".to_string(),
        Origin::new("https".to_string(), "example.com".to_string(), None).unwrap(),
    );
    let _: CanonicalPath = CanonicalPath::new("/test").unwrap();
    let _: FenrirError = FenrirError::internal("test");
}

#[test]
fn test_trait_signatures() {
    // This test verifies that trait methods have the expected signatures
    // by creating dummy implementations
    
    struct DummyPermissionManager;
    
    impl PermissionManager for DummyPermissionManager {
        fn check_access(&self, _tab: &TabId, _path: &CanonicalPath, _access: AccessLevel) -> bool {
            false
        }
        
        fn grant_access(&self, _tab: &TabId, _path: CanonicalPath, _access: AccessLevel) -> Result<(), AccessError> {
            Ok(())
        }
        
        fn revoke_access(&self, _tab: &TabId, _path: &CanonicalPath) -> Result<(), AccessError> {
            Ok(())
        }
        
        fn get_tab_permissions(&self, _tab: &TabId) -> Vec<PathPermission> {
            Vec::new()
        }
        
        fn get_path_permissions(&self, _path: &CanonicalPath) -> Vec<PathPermission> {
            Vec::new()
        }
    }
    
    struct DummyTabRegistry;
    
    impl TabRegistry for DummyTabRegistry {
        fn register(&self, _info: TabInfo) -> TabId {
            TabId::from_raw(1)
        }
        
        fn unregister(&self, _id: &TabId) {}
        
        fn get_info(&self, _id: &TabId) -> Option<TabInfo> {
            None
        }
        
        fn get_origin(&self, _id: &TabId) -> Option<Origin> {
            None
        }
        
        fn update_info(&self, _id: &TabId, _info: TabInfo) -> Result<(), TabError> {
            Ok(())
        }
        
        fn get_all_ids(&self) -> Vec<TabId> {
            Vec::new()
        }
        
        fn get_all_infos(&self) -> Vec<TabInfo> {
            Vec::new()
        }
    }
    
    // Verify the types implement Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    
    assert_send_sync::<DummyPermissionManager>();
    assert_send_sync::<DummyTabRegistry>();
}
