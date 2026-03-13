//! Tab management types and traits
//!
//! This module defines types and traits for managing browser tabs,
//! including tab identification, information, and registry operations.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a browser tab
///
/// # Security
///
/// Tab IDs should be generated using a cryptographically secure random
/// number generator to prevent prediction or collision attacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(u64);

impl TabId {
    /// Create a new TabId from a raw u64 value
    ///
    /// # Security Warning
    ///
    /// This should only be used when reconstructing a TabId from a
    /// previously generated value. For creating new tabs, use a
    /// proper ID generator.
    pub fn from_raw(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw u64 value of the TabId
    pub fn as_raw(&self) -> u64 {
        self.0
    }

    /// Generate a new random TabId
    ///
    /// # Implementation Note
    ///
    /// Actual implementations should use a cryptographically secure
    /// random number generator.
    pub fn new_random() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self((nanos % u64::MAX as u128) as u64)
    }
}

impl fmt::Display for TabId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TabId({})", self.0)
    }
}

/// State of a browser tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TabState {
    /// Tab is loading content
    Loading,
    /// Tab has finished loading
    Loaded,
    /// Tab is in an error state
    Error,
    /// Tab is being closed
    Closing,
    /// Tab is closed
    Closed,
}

/// Information about a browser tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    /// Unique identifier for this tab
    pub id: TabId,
    /// Current title of the tab (from page title)
    pub title: String,
    /// Current URL being displayed
    pub url: String,
    /// Origin of the current page
    pub origin: crate::permissions::Origin,
    /// Current state of the tab
    pub state: TabState,
    /// Whether the tab is active (focused)
    pub is_active: bool,
    /// Whether the tab is pinned
    pub is_pinned: bool,
    /// When this tab was created
    pub created_at: std::time::SystemTime,
    /// Last time this tab was active
    pub last_active: std::time::SystemTime,
}

impl TabInfo {
    /// Create a new TabInfo with default values
    ///
    /// # Arguments
    ///
    /// * `url` - The initial URL for the tab
    /// * `origin` - The origin of the URL
    ///
    /// # Returns
    ///
    /// Returns a new TabInfo with a generated ID and default values.
    pub fn new(url: String, origin: crate::permissions::Origin) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            id: TabId::new_random(),
            title: String::new(),
            url,
            origin,
            state: TabState::Loading,
            is_active: false,
            is_pinned: false,
            created_at: now,
            last_active: now,
        }
    }

    /// Update the tab's title
    pub fn update_title(&mut self, title: String) {
        self.title = title;
    }

    /// Update the tab's URL and origin
    pub fn update_url(&mut self, url: String, origin: crate::permissions::Origin) {
        self.url = url;
        self.origin = origin;
        self.last_active = std::time::SystemTime::now();
    }

    /// Update the tab's state
    pub fn update_state(&mut self, state: TabState) {
        self.state = state;
        if state == TabState::Loaded {
            self.last_active = std::time::SystemTime::now();
        }
    }

    /// Mark the tab as active or inactive
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        if active {
            self.last_active = std::time::SystemTime::now();
        }
    }
}

/// Trait for tab registry operations
///
/// A tab registry tracks all active tabs and their information.
/// Implementations must be thread-safe.
pub trait TabRegistry: Send + Sync {
    /// Register a new tab and return its ID
    ///
    /// # Arguments
    ///
    /// * `info` - Initial tab information
    ///
    /// # Returns
    ///
    /// The TabId assigned to the new tab.
    fn register(&self, info: TabInfo) -> TabId;
    
    /// Unregister a tab (mark it as closed)
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to unregister
    fn unregister(&self, id: &TabId);
    
    /// Get information about a specific tab
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to get information for
    ///
    /// # Returns
    ///
    /// `Some(TabInfo)` if the tab exists, `None` otherwise.
    fn get_info(&self, id: &TabId) -> Option<TabInfo>;
    
    /// Get the origin of a specific tab
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to get the origin for
    ///
    /// # Returns
    ///
    /// `Some(Origin)` if the tab exists, `None` otherwise.
    fn get_origin(&self, id: &TabId) -> Option<crate::permissions::Origin>;
    
    /// Update tab information
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to update
    /// * `info` - New tab information
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(TabError)` on failure.
    fn update_info(&self, id: &TabId, info: TabInfo) -> Result<(), TabError>;
    
    /// Get all registered tab IDs
    ///
    /// # Returns
    ///
    /// A vector of all registered tab IDs.
    fn get_all_ids(&self) -> Vec<TabId>;
    
    /// Get information for all tabs
    ///
    /// # Returns
    ///
    /// A vector of information for all registered tabs.
    fn get_all_infos(&self) -> Vec<TabInfo>;
}

/// Trait for tab management operations
///
/// This trait extends TabRegistry with operations for managing
/// tab lifecycle and interactions.
pub trait TabManager: TabRegistry {
    /// Create a new tab with the given URL
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to load in the new tab
    /// * `make_active` - Whether to make the new tab active
    ///
    /// # Returns
    ///
    /// The TabId of the newly created tab.
    fn create_tab(&self, url: String, make_active: bool) -> Result<TabId, TabError>;
    
    /// Close a tab
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to close
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(TabError)` on failure.
    fn close_tab(&self, id: &TabId) -> Result<(), TabError>;
    
    /// Activate a tab (bring it to front)
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to activate
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(TabError)` on failure.
    fn activate_tab(&self, id: &TabId) -> Result<(), TabError>;
    
    /// Navigate a tab to a new URL
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to navigate
    /// * `url` - The new URL to navigate to
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(TabError)` on failure.
    fn navigate_tab(&self, id: &TabId, url: String) -> Result<(), TabError>;
    
    /// Reload a tab
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the tab to reload
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(TabError)` on failure.
    fn reload_tab(&self, id: &TabId) -> Result<(), TabError>;
}

/// Error type for tab-related operations
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum TabError {
    /// Tab not found
    #[error("Tab not found: {0}")]
    NotFound(TabId),
    
    /// Tab already exists
    #[error("Tab already exists: {0}")]
    AlreadyExists(TabId),
    
    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    /// Invalid origin
    #[error("Invalid origin: {0}")]
    InvalidOrigin(String),
    
    /// Operation not allowed in current tab state
    #[error("Operation not allowed in state: {0:?}")]
    InvalidState(TabState),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl TabError {
    /// Create a new tab error with a message
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
