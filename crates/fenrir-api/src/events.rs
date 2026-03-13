//! Event types for communication between components
//!
//! This module defines events that are exchanged between UI components,
//! tabs, and system services. All events must be serializable for IPC.

use serde::{Deserialize, Serialize};

/// Events related to tab lifecycle and state changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabEvent {
    /// A new tab was created
    ///
    /// Contains the tab ID and initial URL.
    Created {
        /// ID of the new tab
        tab_id: crate::tabs::TabId,
        /// Initial URL of the tab
        url: String,
    },
    
    /// A tab was closed
    ///
    /// Contains the tab ID that was closed.
    Closed {
        /// ID of the closed tab
        tab_id: crate::tabs::TabId,
    },
    
    /// Tab title changed
    ///
    /// Emitted when a page sets its title.
    TitleChanged {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
        /// New title
        title: String,
    },
    
    /// Tab URL changed
    ///
    /// Emitted when navigation occurs.
    UrlChanged {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
        /// New URL
        url: String,
        /// Origin of the new URL
        origin: crate::permissions::Origin,
    },
    
    /// Tab loading state changed
    ///
    /// Emitted when a tab starts or finishes loading.
    LoadingStateChanged {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
        /// Whether the tab is loading
        loading: bool,
        /// Progress (0.0 to 1.0) if loading
        progress: Option<f32>,
    },
    
    /// Tab became active (focused)
    ///
    /// Emitted when a tab receives focus.
    Activated {
        /// ID of the activated tab
        tab_id: crate::tabs::TabId,
    },
    
    /// Tab became inactive (lost focus)
    ///
    /// Emitted when a tab loses focus.
    Deactivated {
        /// ID of the deactivated tab
        tab_id: crate::tabs::TabId,
    },
    
    /// Tab crashed or encountered a fatal error
    ///
    /// Emitted when a tab becomes unusable.
    Crashed {
        /// ID of the crashed tab
        tab_id: crate::tabs::TabId,
        /// Error message
        error: String,
    },
}

/// Events from the user interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiEvent {
    /// User requested to create a new tab
    ///
    /// The UI should provide an optional URL to load.
    NewTabRequested {
        /// URL to load in the new tab (empty for blank tab)
        url: Option<String>,
        /// Whether to make the new tab active
        make_active: bool,
    },
    
    /// User requested to close a tab
    CloseTabRequested {
        /// ID of the tab to close
        tab_id: crate::tabs::TabId,
    },
    
    /// User requested to activate a tab
    ActivateTabRequested {
        /// ID of the tab to activate
        tab_id: crate::tabs::TabId,
    },
    
    /// User entered a URL to navigate to
    NavigateRequested {
        /// ID of the tab to navigate
        tab_id: crate::tabs::TabId,
        /// URL to navigate to
        url: String,
    },
    
    /// User requested to reload a tab
    ReloadRequested {
        /// ID of the tab to reload
        tab_id: crate::tabs::TabId,
    },
    
    /// User requested to go back in history
    GoBackRequested {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
    },
    
    /// User requested to go forward in history
    GoForwardRequested {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
    },
    
    /// User requested to stop loading
    StopRequested {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
    },
    
    /// User interacted with the browser UI
    ///
    /// This is a generic event for UI interactions that don't have
    /// specific event types.
    Interaction {
        /// Type of interaction
        interaction_type: String,
        /// Additional data
        data: serde_json::Value,
    },
}

/// Events related to permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionEvent {
    /// A permission was requested by a tab
    ///
    /// The UI should prompt the user to grant or deny the permission.
    Requested {
        /// ID of the tab requesting permission
        tab_id: crate::tabs::TabId,
        /// Type of permission being requested
        permission_type: String,
        /// Resource the permission is for (e.g., path, device)
        resource: String,
        /// Additional context
        context: serde_json::Value,
    },
    
    /// A permission was granted
    Granted {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
        /// Type of permission
        permission_type: String,
        /// Resource the permission is for
        resource: String,
        /// When the permission was granted
        granted_at: std::time::SystemTime,
    },
    
    /// A permission was denied
    Denied {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
        /// Type of permission
        permission_type: String,
        /// Resource the permission is for
        resource: String,
        /// When the permission was denied
        denied_at: std::time::SystemTime,
    },
    
    /// A permission was revoked
    Revoked {
        /// ID of the tab
        tab_id: crate::tabs::TabId,
        /// Type of permission
        permission_type: String,
        /// Resource the permission is for
        resource: String,
        /// When the permission was revoked
        revoked_at: std::time::SystemTime,
    },
}

/// System-level events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    /// Browser is starting up
    Startup,
    
    /// Browser is shutting down
    Shutdown,
    
    /// Memory usage is high
    ///
    /// Emitted when memory usage exceeds thresholds.
    HighMemoryUsage {
        /// Current memory usage in bytes
        usage: u64,
        /// Threshold that was exceeded
        threshold: u64,
    },
    
    /// Disk space is low
    ///
    /// Emitted when disk space falls below thresholds.
    LowDiskSpace {
        /// Available space in bytes
        available: u64,
        /// Threshold that was crossed
        threshold: u64,
    },
    
    /// Network connectivity changed
    ConnectivityChanged {
        /// Whether the system is now online
        online: bool,
    },
    
    /// System is going to sleep
    SystemSleep,
    
    /// System woke from sleep
    SystemWake,
}

/// Trait for event bus systems
///
/// An event bus allows components to publish and subscribe to events
/// without direct coupling.
pub trait EventBus: Send + Sync {
    /// Publish an event to the bus
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(EventError)` on failure.
    fn publish(&self, event: Event) -> Result<(), EventError>;
    
    /// Subscribe to events of a specific type
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call when an event is received
    ///
    /// # Returns
    ///
    /// A subscription ID that can be used to unsubscribe.
    fn subscribe<F>(&self, callback: F) -> SubscriptionId
    where
        F: Fn(Event) + Send + Sync + 'static;
    
    /// Unsubscribe from events
    ///
    /// # Arguments
    ///
    /// * `subscription_id` - The ID returned by `subscribe`
    fn unsubscribe(&self, subscription_id: SubscriptionId);
    
    /// Wait for the next event of a specific type
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait
    ///
    /// # Returns
    ///
    /// `Some(Event)` if an event was received, `None` on timeout.
    fn wait_for_event(&self, timeout: std::time::Duration) -> Option<Event>;
}

/// Type alias for subscription IDs
pub type SubscriptionId = u64;

/// Union type for all possible events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Tab-related event
    Tab(TabEvent),
    /// UI-related event
    Ui(UiEvent),
    /// Permission-related event
    Permission(PermissionEvent),
    /// System-related event
    System(SystemEvent),
}

impl Event {
    /// Get the type of this event as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Tab(_) => "tab",
            Self::Ui(_) => "ui",
            Self::Permission(_) => "permission",
            Self::System(_) => "system",
        }
    }
    
    /// Check if this is a tab event
    pub fn is_tab_event(&self) -> bool {
        matches!(self, Self::Tab(_))
    }
    
    /// Check if this is a UI event
    pub fn is_ui_event(&self) -> bool {
        matches!(self, Self::Ui(_))
    }
    
    /// Check if this is a permission event
    pub fn is_permission_event(&self) -> bool {
        matches!(self, Self::Permission(_))
    }
    
    /// Check if this is a system event
    pub fn is_system_event(&self) -> bool {
        matches!(self, Self::System(_))
    }
}

/// Error type for event bus operations
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum EventError {
    /// Event bus is full
    #[error("Event bus is full")]
    BusFull,
    
    /// Subscription not found
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(SubscriptionId),
    
    /// Invalid event
    #[error("Invalid event: {0}")]
    InvalidEvent(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl EventError {
    /// Create a new event error with a message
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
