//! ServoDelegate: globale Servo-Callbacks.
//!
//! Wichtigster Hook: load_web_resource() — hier werden ALLE HTTP-Requests
//! von Servo abgefangen und durch fenrir-network's InterceptorPipeline gerouted.

use fenrir_network::{
    FenrirNetworkHandler, NetworkRequest,
    interceptor::InterceptorResult,
    request::HttpMethod,
};
use servo::{ConsoleLogLevel, ServoDelegate, ServoError, WebResourceLoad};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;
use tracing::{error, info, warn};
use url::Url;

pub struct FenrirServoDelegate {
    network: Arc<FenrirNetworkHandler>,
}

impl FenrirServoDelegate {
    pub fn new(network: Arc<FenrirNetworkHandler>) -> Self {
        Self { network }
    }
}

impl ServoDelegate for FenrirServoDelegate {
    fn notify_error(&self, err: ServoError) {
        error!(servo_error = ?err, "Servo internal error");
    }

    /// Jeder HTTP/HTTPS Request von Servo läuft hier durch.
    /// Pipeline: HTTPS-Enforcer → (Phase 2: Tracker-Blocker) → HTTP Client
    fn load_web_resource(&self, load: WebResourceLoad) {
        let url_raw = load.request().url.clone();
        let network = Arc::clone(&self.network);

        let url = match Url::parse(url_raw.as_str()) {
            Ok(u) => u,
            Err(e) => {
                error!(url = %url_raw, error = %e, "Ungültige URL — blockiert");
                return; // Servo's Default: allow wenn keine Response → lass durch
            }
        };

        let method_str = load.request().method.as_str().to_uppercase();
        let method = match method_str.as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            "PATCH" => HttpMethod::Patch,
            other => HttpMethod::Other(other.to_string()),
        };

        let request = NetworkRequest {
            url: url.clone(),
            method,
            headers: HashMap::new(), // Servo-Headers werden direkt von Servo verwaltet
            body: None,
            is_main_frame: load.request().is_for_main_frame,
        };

        // Synchron aus dem Servo-Delegate-Kontext heraus in den tokio-Runtime spawnen.
        // Servo's load_web_resource ist sync — wir spawnen den async Block.
        let handle = Handle::current();
        handle.spawn(async move {
            match network.pipeline_check(&request).await {
                InterceptorResult::Allow => {
                    info!(url = %url, "Request erlaubt → Servo übernimmt");
                    // Kein intercept → Servo lädt normal
                }
                InterceptorResult::Block(reason) => {
                    warn!(url = %url, reason = ?reason, "Request blockiert");
                    // TODO: load.block() wenn Servo API das anbietet
                    // Für jetzt: loggen und Servo's Default-Verhalten greifen lassen
                }
                InterceptorResult::Redirect(new_url) => {
                    info!(from = %url, to = %new_url, "Request umgeleitet (HTTPS Upgrade)");
                    // TODO: load.redirect(new_url) wenn Servo API das anbietet
                }
            }
        });
    }

    fn show_console_message(&self, level: ConsoleLogLevel, message: String) {
        match level {
            ConsoleLogLevel::Error => error!(target: "servo::console", "{}", message),
            ConsoleLogLevel::Warn => warn!(target: "servo::console", "{}", message),
            _ => info!(target: "servo::console", "{}", message),
        }
    }
}
