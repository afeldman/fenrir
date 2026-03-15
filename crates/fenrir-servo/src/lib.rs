//! fenrir-servo — Servo rendering engine integration for Fenrir Browser.
//!
//! This crate acts as a bridge between Servo's embedding API and Fenrir's
//! internal architecture. It integrates:
//! - fenrir-rendering: Pure rendering logic
//! - fenrir-metrics: Performance monitoring
//! - fenrir-network: Network stack
//!
//! Each HTTP request goes through fenrir-network's InterceptorPipeline.

mod delegate;
mod waker;
// mod servo_renderer; // Disabled due to Send/Sync issues with Servo API

pub use delegate::FenrirServoDelegate;
pub use waker::FenrirEventLoopWaker;
// pub use servo_renderer::ServoRenderer;

use fenrir_core::error::FenrirError;
use fenrir_network::FenrirNetworkHandler;
use fenrir_rendering::webview::FenrirWebViewDelegate;
use fenrir_metrics::render::{RenderMetrics, RenderMetricsConfig, RenderStats, PerformanceAnomaly};
use servo::{Servo, ServoBuilder, WebView, WebViewBuilder, LoadStatus, NavigationRequest, PermissionRequest, WebViewDelegate};
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;
use url::Url;

/// Main entry point: manages Servo instance + network stack + metrics.
pub struct FenrirHost {
    servo: Servo,
    waker_notify: Arc<Notify>,
    render_metrics: Arc<RenderMetrics>,
}

impl FenrirHost {
    /// Initialize Servo + fenrir-network + metrics.
    pub async fn new() -> Result<Self, FenrirError> {
        let waker_notify = Arc::new(Notify::new());
        let waker = FenrirEventLoopWaker::new(Arc::clone(&waker_notify));

        // Network stack: DoH + HTTPS enforcer
        let network = Arc::new(FenrirNetworkHandler::with_defaults().await?);

        let delegate = FenrirServoDelegate::new(Arc::clone(&network));

        info!("Initializing Servo engine");

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(waker))
            .build();

        servo.set_delegate(Rc::new(delegate));

        // Initialize render metrics for performance monitoring
        let render_metrics_config = RenderMetricsConfig {
            window_size: 100,
            auto_log: true,
            log_interval: 10,
            ..Default::default()
        };
        
        let render_metrics = Arc::new(RenderMetrics::new(render_metrics_config));

        info!("Servo + fenrir-network + metrics initialized");

        Ok(Self {
            servo,
            waker_notify,
            render_metrics,
        })
    }
    
    /// Create with custom metrics configuration.
    pub async fn with_metrics_config(config: RenderMetricsConfig) -> Result<Self, FenrirError> {
        let waker_notify = Arc::new(Notify::new());
        let waker = FenrirEventLoopWaker::new(Arc::clone(&waker_notify));

        let network = Arc::new(FenrirNetworkHandler::with_defaults().await?);
        let delegate = FenrirServoDelegate::new(Arc::clone(&network));

        info!("Initializing Servo engine with custom metrics");

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(waker))
            .build();

        servo.set_delegate(Rc::new(delegate));
        
        let render_metrics = Arc::new(RenderMetrics::new(config));

        info!("Servo initialized with custom metrics configuration");

        Ok(Self {
            servo,
            waker_notify,
            render_metrics,
        })
    }

    /// Open new WebView (tab) with integrated metrics.
    pub fn open_tab(
        &self,
        url: Url,
        rendering_context: Rc<dyn servo::RenderingContext>,
    ) -> Result<WebView, FenrirError> {
        // Create WebView with metrics-aware delegate
        let delegate = MetricsAwareWebViewDelegate::new(
            FenrirWebViewDelegate::new(),
            Arc::clone(&self.render_metrics),
        );

        let webview = WebViewBuilder::new(&self.servo, rendering_context)
            .url(url)
            .delegate(Rc::new(delegate))
            .build();

        Ok(webview)
    }

    /// Get reference to render metrics for external monitoring.
    pub fn render_metrics(&self) -> Arc<RenderMetrics> {
        Arc::clone(&self.render_metrics)
    }

    /// Get current render statistics.
    pub fn render_stats(&self) -> RenderStats {
        self.render_metrics.calculate_stats()
    }

    /// Check for performance anomalies.
    pub fn check_anomalies(&self) -> Vec<PerformanceAnomaly> {
        self.render_metrics.check_anomalies()
    }

    /// Event loop tick: must be called from platform event loop.
    pub fn spin(&self) {
        self.servo.spin_event_loop();
    }

    /// Wait asynchronously until Servo signals work.
    pub async fn wait_for_work(&self) {
        self.waker_notify.notified().await;
    }
}

/// WebView delegate that integrates metrics collection.
struct MetricsAwareWebViewDelegate {
    inner: FenrirWebViewDelegate,
    render_metrics: Arc<RenderMetrics>,
}

impl MetricsAwareWebViewDelegate {
    fn new(inner: FenrirWebViewDelegate, render_metrics: Arc<RenderMetrics>) -> Self {
        Self { inner, render_metrics }
    }
}

impl WebViewDelegate for MetricsAwareWebViewDelegate {
    fn notify_new_frame_ready(&self, webview: WebView) {
        // Start timing before calling inner delegate
        let timer = self.render_metrics.start_timing();
        
        // Call inner delegate (pure rendering logic)
        self.inner.notify_new_frame_ready(webview);
        
        // Finish timing (automatically records when timer drops)
        timer.finish();
    }

    fn request_navigation(&self, webview: WebView, request: NavigationRequest) {
        self.inner.request_navigation(webview, request);
    }

    fn request_permission(&self, webview: WebView, request: PermissionRequest) {
        self.inner.request_permission(webview, request);
    }

    fn notify_page_title_changed(&self, webview: WebView, title: Option<String>) {
        self.inner.notify_page_title_changed(webview, title);
    }

    fn notify_url_changed(&self, webview: WebView, url: Url) {
        self.inner.notify_url_changed(webview, url);
    }

    fn notify_load_status_changed(&self, webview: WebView, status: LoadStatus) {
        self.inner.notify_load_status_changed(webview, status);
    }
}
