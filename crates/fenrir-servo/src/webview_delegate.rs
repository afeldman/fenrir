//! WebViewDelegate: Tab-spezifische Callbacks von Servo.
//!
//! Jede WebView (Tab) bekommt eine eigene Delegate-Instanz.

use servo::{LoadStatus, NavigationRequest, PermissionRequest, WebView, WebViewDelegate};
use tracing::{debug, info, warn};
use url::Url;

pub struct FenrirWebViewDelegate;

impl FenrirWebViewDelegate {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FenrirWebViewDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl WebViewDelegate for FenrirWebViewDelegate {
    // --- Rendering ---

    /// Servo signalisiert: neuer Frame bereit → WebView::paint() aufrufen.
    fn notify_new_frame_ready(&self, _webview: WebView) {
        // TODO: Paint-Signal an Tauri-Render-Loop schicken.
        debug!("New frame ready");
    }

    // --- Navigation ---

    /// Navigation-Request: URL prüfen, erlauben oder ablehnen.
    ///
    /// TODO Phase 2: durch fenrir-security (URL-Filter, Phishing) routen.
    fn request_navigation(&self, _webview: WebView, request: NavigationRequest) {
        let url = &request.url;
        info!(url = %url, "Navigation request (passthrough — fenrir-security not yet connected)");
        // Kein explizites allow/deny → Servo-Default (allow)
    }

    // --- Permissions ---

    /// Browser-Permissions (Kamera, Mikrofon, Geolocation, etc.)
    ///
    /// TODO Phase 2: durch fenrir-security Permission Engine routen.
    fn request_permission(&self, _webview: WebView, request: PermissionRequest) {
        warn!(
            permission = ?request.feature(),
            "Permission request denied by default (fenrir-security not yet connected)"
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
