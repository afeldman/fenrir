//! Browser state management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

/// State of a single browser tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    /// Unique tab ID
    pub id: String,
    /// Current URL
    pub url: Url,
    /// Page title
    pub title: String,
    /// Favicon URL
    pub favicon: Option<Url>,
    /// Whether the tab is loading
    pub loading: bool,
    /// Load progress (0.0 to 1.0)
    pub progress: f32,
    /// Associated Servo instance ID
    pub instance_id: String,
    /// Whether the tab is pinned
    pub pinned: bool,
    /// Timestamp when tab was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl TabState {
    /// Create a new tab state
    pub fn new(id: String, url: Url, instance_id: String, title: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            url,
            title,
            favicon: None,
            loading: true,
            progress: 0.0,
            instance_id,
            pinned: false,
            created_at: now,
            last_activity: now,
        }
    }
    
    /// Update the URL
    pub fn update_url(&mut self, url: Url) {
        self.url = url;
        self.last_activity = chrono::Utc::now();
    }
    
    /// Update the title
    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.last_activity = chrono::Utc::now();
    }
    
    /// Update loading state
    pub fn update_loading(&mut self, loading: bool, progress: f32) {
        self.loading = loading;
        self.progress = progress;
        self.last_activity = chrono::Utc::now();
    }
    
    /// Update favicon
    pub fn update_favicon(&mut self, favicon: Option<Url>) {
        self.favicon = favicon;
        self.last_activity = chrono::Utc::now();
    }
    
    /// Toggle pinned state
    pub fn toggle_pinned(&mut self) {
        self.pinned = !self.pinned;
        self.last_activity = chrono::Utc::now();
    }
}

/// Main browser state
pub struct BrowserState {
    /// All tabs
    tabs: RwLock<HashMap<String, TabState>>,
    /// Active tab ID
    active_tab: RwLock<Option<String>>,
    /// Browser window state
    window_state: WindowState,
    /// User preferences
    preferences: Preferences,
}

impl BrowserState {
    /// Create new browser state
    pub fn new() -> Self {
        Self {
            tabs: RwLock::new(HashMap::new()),
            active_tab: RwLock::new(None),
            window_state: WindowState::default(),
            preferences: Preferences::default(),
        }
    }
    
    /// Initialize browser state
    pub async fn initialize(&self) -> Result<(), crate::UiError> {
        // Create initial "about:blank" tab
        let initial_url = Url::parse("about:blank")
            .map_err(|e| crate::UiError::InvalidUrl("about:blank".to_string(), e.to_string()))?;
        
        let tab_id = uuid::Uuid::new_v4().to_string();
        let instance_id = format!("initial-{}", tab_id);
        
        let tab = TabState::new(
            tab_id.clone(),
            initial_url,
            instance_id,
            "New Tab".to_string(),
        );
        
        self.tabs.write().await.insert(tab_id.clone(), tab);
        *self.active_tab.write().await = Some(tab_id);
        
        Ok(())
    }
    
    /// Add a new tab
    pub fn add_tab(&self, url: Url, instance_id: String, title: String) -> String {
        let tab_id = uuid::Uuid::new_v4().to_string();
        let tab = TabState::new(tab_id.clone(), url, instance_id, title);
        
        self.tabs.blocking_write().insert(tab_id.clone(), tab);
        
        tab_id
    }
    
    /// Get a tab by ID
    pub fn get_tab(&self, tab_id: &str) -> Option<TabState> {
        self.tabs.blocking_read().get(tab_id).cloned()
    }
    
    /// Get all tabs
    pub fn tabs(&self) -> HashMap<String, TabState> {
        self.tabs.blocking_read().clone()
    }
    
    /// Remove a tab
    pub fn remove_tab(&self, tab_id: &str) -> Option<TabState> {
        self.tabs.blocking_write().remove(tab_id)
    }
    
    /// Get active tab ID
    pub fn get_active_tab(&self) -> Option<String> {
        self.active_tab.blocking_read().clone()
    }
    
    /// Set active tab
    pub fn set_active_tab(&self, tab_id: String) {
        *self.active_tab.blocking_write() = Some(tab_id);
    }
    
    /// Update tab URL
    pub fn update_tab_url(&self, tab_id: &str, url: Url) -> Result<(), crate::UiError> {
        if let Some(tab) = self.tabs.blocking_write().get_mut(tab_id) {
            tab.update_url(url);
            Ok(())
        } else {
            Err(crate::UiError::TabNotFound(tab_id.to_string()))
        }
    }
    
    /// Update tab title
    pub fn update_tab_title(&self, tab_id: &str, title: String) -> Result<(), crate::UiError> {
        if let Some(tab) = self.tabs.blocking_write().get_mut(tab_id) {
            tab.update_title(title);
            Ok(())
        } else {
            Err(crate::UiError::TabNotFound(tab_id.to_string()))
        }
    }
    
    /// Update tab loading state
    pub fn update_tab_loading(&self, tab_id: &str, loading: bool, progress: f32) -> Result<(), crate::UiError> {
        if let Some(tab) = self.tabs.blocking_write().get_mut(tab_id) {
            tab.update_loading(loading, progress);
            Ok(())
        } else {
            Err(crate::UiError::TabNotFound(tab_id.to_string()))
        }
    }
    
    /// Get window state
    pub fn window_state(&self) -> WindowState {
        self.window_state.clone()
    }
    
    /// Update window state
    pub fn update_window_state(&mut self, state: WindowState) {
        self.window_state = state;
    }
    
    /// Get preferences
    pub fn preferences(&self) -> Preferences {
        self.preferences.clone()
    }
    
    /// Update preferences
    pub fn update_preferences(&mut self, preferences: Preferences) {
        self.preferences = preferences;
    }
}

/// Browser window state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// Window position (x, y)
    pub position: Option<(i32, i32)>,
    /// Window size (width, height)
    pub size: (u32, u32),
    /// Whether window is maximized
    pub maximized: bool,
    /// Whether window is fullscreen
    pub fullscreen: bool,
    /// Zoom level
    pub zoom_level: f32,
    /// Theme (light/dark/system)
    pub theme: String,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            position: None,
            size: (1024, 768),
            maximized: false,
            fullscreen: false,
            zoom_level: 1.0,
            theme: "system".to_string(),
        }
    }
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    /// Home page URL
    pub home_page: String,
    /// Search engine
    pub search_engine: String,
    /// Whether to show bookmarks bar
    pub show_bookmarks_bar: bool,
    /// Whether to enable JavaScript
    pub enable_javascript: bool,
    /// Whether to block popups
    pub block_popups: bool,
    /// Whether to enable cookies
    pub enable_cookies: bool,
    /// Cookie policy (all, none, third-party)
    pub cookie_policy: String,
    /// Whether to send Do Not Track header
    pub do_not_track: bool,
    /// Default download location
    pub download_location: String,
    /// Whether to ask for download location
    pub ask_download_location: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            home_page: "about:blank".to_string(),
            search_engine: "https://duckduckgo.com/?q=".to_string(),
            show_bookmarks_bar: true,
            enable_javascript: true,
            block_popups: true,
            enable_cookies: true,
            cookie_policy: "all".to_string(),
            do_not_track: true,
            download_location: "".to_string(),
            ask_download_location: true,
        }
    }
}

/// Application state shared across components
pub struct AppState {
    browser_state: Arc<BrowserState>,
}

impl AppState {
    /// Create new application state
    pub fn new(browser_state: Arc<BrowserState>) -> Self {
        Self { browser_state }
    }
    
    /// Get browser state
    pub fn browser_state(&self) -> Arc<BrowserState> {
        Arc::clone(&self.browser_state)
    }
}
