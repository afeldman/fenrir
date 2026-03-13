//! fenrir-network — Fenrir's gesamter HTTP Stack.
//!
//! # Architektur
//!
//! Jeder Request (von Servo, vom Browser selbst, von Extensions) läuft durch
//! eine `InterceptorPipeline` bevor er das Netz erreicht:
//!
//! ```text
//! Servo load_web_resource()
//!     ↓
//! InterceptorPipeline
//!     ├── DnsInterceptor    (DoH/DoT via hickory-dns)
//!     ├── TrackerInterceptor (Phase 2: fenrir-privacy)
//!     ├── VpnInterceptor    (Phase 4: fenrir-vpn)
//!     └── (beliebig mehr)
//!     ↓
//! FenrirHttpClient          (reqwest + rustls, HTTP/2)
//!     ↓
//! Internet
//! ```
//!
//! # Servo-Integration
//!
//! `FenrirNetworkHandler` implementiert das Interface das `fenrir-servo`'s
//! `FenrirServoDelegate::load_web_resource()` braucht.

pub mod bandwidth;
pub mod client;
pub mod dns;
pub mod interceptor;
pub mod request;

pub use client::FenrirHttpClient;
pub use dns::FenrirDnsResolver;
pub use interceptor::{
    BlockReason, InterceptorPipeline, InterceptorResult, NetworkInterceptor,
};
pub use request::{NetworkRequest, NetworkResponse};

use fenrir_core::error::FenrirError;
use std::sync::Arc;
use tracing::info;

/// Vollständiger Netzwerk-Handler — kombiniert DNS, Pipeline und HTTP Client.
///
/// Wird von `fenrir-servo`'s `FenrirServoDelegate` genutzt um jeden
/// HTTP/HTTPS Request abzufangen und durch die Pipeline zu schicken.
pub struct FenrirNetworkHandler {
    pipeline: InterceptorPipeline,
    client: FenrirHttpClient,
}

impl FenrirNetworkHandler {
    pub fn new(pipeline: InterceptorPipeline, client: FenrirHttpClient) -> Self {
        Self { pipeline, client }
    }

    /// Standard-Konfiguration: DoH + kein Tracker (Phase 2 fügt mehr hinzu).
    pub async fn with_defaults() -> Result<Self, FenrirError> {
        let dns = FenrirDnsResolver::new_doh().await?;
        let client = FenrirHttpClient::new(Arc::new(dns))?;
        let pipeline = InterceptorPipeline::empty();
        Ok(Self { pipeline, client })
    }

    /// Nur die Pipeline laufen lassen — ohne HTTP-Aufruf.
    /// Genutzt von fenrir-servo's ServoDelegate: Servo übernimmt den eigentlichen Fetch,
    /// wir prüfen nur ob der Request erlaubt ist.
    pub async fn pipeline_check(&self, request: &NetworkRequest) -> InterceptorResult {
        self.pipeline.run(request).await
    }

    /// Verarbeitet einen Request durch Pipeline → HTTP Client.
    ///
    /// Rückgabe: `None` wenn blockiert, `Some(response)` wenn erlaubt.
    pub async fn handle(
        &self,
        request: NetworkRequest,
    ) -> Result<Option<NetworkResponse>, FenrirError> {
        match self.pipeline.run(&request).await {
            InterceptorResult::Allow => {
                let response = self.client.execute(request).await?;
                Ok(Some(response))
            }
            InterceptorResult::Block(reason) => {
                info!(url = %request.url, reason = ?reason, "Request blockiert");
                Ok(None)
            }
            InterceptorResult::Redirect(url) => {
                let redirected = NetworkRequest { url, ..request };
                let response = self.client.execute(redirected).await?;
                Ok(Some(response))
            }
        }
    }
}
