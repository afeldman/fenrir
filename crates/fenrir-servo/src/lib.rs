//! fenrir-servo — Servo rendering engine integration for Fenrir Browser.
//!
//! Wraps Servo's embedding API und integriert es mit tokio und fenrir-network.
//! Jeder HTTP-Request läuft durch fenrir-network's InterceptorPipeline.

mod delegate;
mod waker;
mod webview_delegate;

pub use delegate::FenrirServoDelegate;
pub use waker::FenrirEventLoopWaker;
pub use webview_delegate::FenrirWebViewDelegate;

use fenrir_core::error::FenrirError;
use fenrir_network::{FenrirHttpClient, FenrirNetworkHandler, interceptor::InterceptorPipeline};
use servo::{Servo, ServoBuilder, WebView, WebViewBuilder};
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;
use url::Url;

/// Haupteinstiegspunkt: verwaltet Servo-Instanz + Netzwerk-Stack.
pub struct FenrirHost {
    servo: Servo,
    waker_notify: Arc<Notify>,
}

impl FenrirHost {
    /// Servo + fenrir-network initialisieren.
    pub async fn new() -> Result<Self, FenrirError> {
        let waker_notify = Arc::new(Notify::new());
        let waker = FenrirEventLoopWaker::new(Arc::clone(&waker_notify));

        // Netzwerk-Stack: DoH + HTTPS-Enforcer
        let network = Arc::new(FenrirNetworkHandler::with_defaults().await?);

        let delegate = FenrirServoDelegate::new(Arc::clone(&network));

        info!("Initializing Servo engine");

        let servo = ServoBuilder::default()
            .event_loop_waker(Box::new(waker))
            .build();

        servo.set_delegate(Rc::new(delegate));

        info!("Servo + fenrir-network initialisiert");

        Ok(Self {
            servo,
            waker_notify,
        })
    }

    /// Neue WebView (Tab) öffnen.
    pub fn open_tab(
        &self,
        url: Url,
        rendering_context: Rc<dyn servo::RenderingContext>,
    ) -> Result<WebView, FenrirError> {
        let webview = WebViewBuilder::new(&self.servo, rendering_context)
            .url(url)
            .delegate(Rc::new(FenrirWebViewDelegate::new()))
            .build();

        Ok(webview)
    }

    /// Event-Loop-Tick: muss aus dem Plattform-Event-Loop aufgerufen werden.
    pub fn spin(&self) {
        self.servo.spin_event_loop();
    }

    /// Warte asynchron bis Servo Arbeit signalisiert.
    pub async fn wait_for_work(&self) {
        self.waker_notify.notified().await;
    }
}
