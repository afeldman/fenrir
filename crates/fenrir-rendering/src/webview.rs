//! WebView management and delegation
//!
//! Provides WebViewDelegate implementation and WebView management utilities.

use servo::{LoadStatus, NavigationRequest, PermissionRequest, WebView};
use tracing::{debug, info, warn};
use url::Url;

/// WebViewDelegate implementation for Fenrir Browser.
/// 
/// This delegate handles WebView-specific callbacks from Servo.
/// Each WebView (tab) gets its own delegate instance.
pub struct FenrirWebViewDelegate {
    // Note: Metrics tracking has been moved to fenrir-metrics crate
    // This delegate only handles pure rendering concerns
}

impl FenrirWebViewDelegate {
    /// Create a new WebViewDelegate.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for FenrirWebViewDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl servo::WebViewDelegate for FenrirWebViewDelegate {
    // --- Rendering ---

    /// Servo signals: new frame ready → call WebView::paint().
    fn notify_new_frame_ready(&self, _webview: WebView) {
        // Pure rendering concern: signal that paint should be called
        // Metrics tracking is handled separately by fenrir-metrics
        debug!("New frame ready for painting");
    }

    // --- Navigation ---

    /// Navigation request: check URL, allow or deny.
    fn request_navigation(&self, _webview: WebView, request: NavigationRequest) {
        let url = &request.url;
        info!(url = %url, "Navigation request");
        // No explicit allow/deny → Servo default (allow)
    }

    // --- Permissions ---

    /// Browser permissions (camera, microphone, geolocation, etc.)
    fn request_permission(&self, _webview: WebView, request: PermissionRequest) {
        warn!(
            permission = ?request.feature(),
            "Permission request denied by default"
        );
        request.deny();
    }

    // --- Page State ---

    fn notify_page_title_changed(&self, _webview: WebView, title: Option<String>) {
        info!(title = ?title, "Page title changed");
    }

    fn notify_url_changed(&self, _webview: WebView, url: Url) {
        info!(url = %url, "URL changed");
    }

    fn notify_load_status_changed(&self, _webview: WebView, status: LoadStatus) {
        debug!(status = ?status, "Load status changed");
    }
}

/// Manages multiple WebViews (tabs).
pub struct WebViewManager {
    // In a real implementation, this would track multiple WebViews
    // For now, it's a placeholder for future expansion
}

impl WebViewManager {
    /// Create a new WebViewManager.
    pub fn new() -> Self {
        Self {}
    }
    
    /// Create a new WebView with default delegate.
    pub fn create_webview(&self) -> FenrirWebViewDelegate {
        FenrirWebViewDelegate::new()
    }
}

impl Default for WebViewManager {
    fn default() -> Self {
        Self::new()
    }
}
