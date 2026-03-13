//! Request Interceptor Pipeline.
//!
//! Jeder Interceptor kann einen Request erlauben, blockieren oder umleiten.
//! Die Pipeline läuft alle Interceptors in Reihenfolge durch bis einer
//! Block/Redirect zurückgibt oder alle Allow zurückgeben.

use crate::request::NetworkRequest;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};
use url::Url;

/// Ergebnis eines Interceptors.
#[derive(Debug)]
pub enum InterceptorResult {
    /// Request durchlassen.
    Allow,
    /// Request blockieren (Tracker, Phishing, VPN Kill-Switch, etc.).
    Block(BlockReason),
    /// Auf andere URL umleiten.
    Redirect(Url),
}

#[derive(Debug, Clone)]
pub enum BlockReason {
    Tracker,
    Phishing,
    VpnKillSwitch,
    UserRule(String),
    MaliciousDomain,
}

/// Trait für jeden Request-Interceptor.
#[async_trait]
pub trait NetworkInterceptor: Send + Sync {
    fn name(&self) -> &'static str;
    async fn intercept(&self, request: &NetworkRequest) -> InterceptorResult;
}

/// Pipeline: läuft Interceptors in Reihenfolge durch.
pub struct InterceptorPipeline {
    interceptors: Vec<Arc<dyn NetworkInterceptor>>,
}

impl InterceptorPipeline {
    pub fn empty() -> Self {
        Self { interceptors: vec![] }
    }

    pub fn new(interceptors: Vec<Arc<dyn NetworkInterceptor>>) -> Self {
        Self { interceptors }
    }

    pub fn add(&mut self, interceptor: Arc<dyn NetworkInterceptor>) {
        self.interceptors.push(interceptor);
    }

    /// Request durch alle Interceptors schicken.
    /// Erster Block/Redirect gewinnt. Alle Allow → Allow.
    pub async fn run(&self, request: &NetworkRequest) -> InterceptorResult {
        for interceptor in &self.interceptors {
            match interceptor.intercept(request).await {
                InterceptorResult::Allow => continue,
                other => return other,
            }
        }
        InterceptorResult::Allow
    }
}

/// Passthrough-Interceptor — erlaubt alles (für Tests und Phase 1).
pub struct AllowAllInterceptor;

#[async_trait]
impl NetworkInterceptor for AllowAllInterceptor {
    fn name(&self) -> &'static str {
        "allow-all"
    }

    async fn intercept(&self, request: &NetworkRequest) -> InterceptorResult {
        debug!(
            method = %request.method,
            url = %request.url,
            "network: request intercepted (allow-all)"
        );
        InterceptorResult::Allow
    }
}

/// HTTP-only Blocker — blockiert unsichere HTTP-Requests (außer localhost).
/// Einfachste Sicherheitsmassnahme: kein unverschlüsselter Traffic.
pub struct HttpsEnforcer;

#[async_trait]
impl NetworkInterceptor for HttpsEnforcer {
    fn name(&self) -> &'static str {
        "https-enforcer"
    }

    async fn intercept(&self, request: &NetworkRequest) -> InterceptorResult {
        debug!(
            method = %request.method,
            url = %request.url,
            "network: request intercepted"
        );

        if request.scheme() == "http" {
            // localhost / 127.0.0.1 erlauben (dev)
            let host = request.host().unwrap_or("");
            if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                return InterceptorResult::Allow;
            }
            // HTTP → HTTPS upgrade versuchen
            if let Ok(_https_url) = request.url.to_string().parse::<Url>() {
                // Einfacher Weg: URL-String ersetzen
                let https_str = request.url.as_str().replacen("http://", "https://", 1);
                if let Ok(url) = Url::parse(&https_str) {
                    info!(
                        original = %request.url,
                        upgraded = %url,
                        "network: HTTP→HTTPS upgrade"
                    );
                    return InterceptorResult::Redirect(url);
                }
            }
            
            error!(
                url = %request.url,
                "network: HTTP request ohne HTTPS-Upgrade-Möglichkeit"
            );
        }
        InterceptorResult::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{HttpMethod, NetworkRequest};
    use std::collections::HashMap;

    fn make_request(url: &str) -> NetworkRequest {
        NetworkRequest {
            url: Url::parse(url).unwrap(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
            is_main_frame: true,
        }
    }

    #[tokio::test]
    async fn allow_all_passes_everything() {
        let pipeline = InterceptorPipeline::new(vec![Arc::new(AllowAllInterceptor)]);
        let req = make_request("https://example.com");
        assert!(matches!(pipeline.run(&req).await, InterceptorResult::Allow));
    }

    #[tokio::test]
    async fn empty_pipeline_allows() {
        let pipeline = InterceptorPipeline::empty();
        let req = make_request("https://example.com");
        assert!(matches!(pipeline.run(&req).await, InterceptorResult::Allow));
    }

    #[tokio::test]
    async fn https_enforcer_upgrades_http() {
        let pipeline = InterceptorPipeline::new(vec![Arc::new(HttpsEnforcer)]);
        let req = make_request("http://example.com/page");
        match pipeline.run(&req).await {
            InterceptorResult::Redirect(url) => {
                assert_eq!(url.scheme(), "https");
            }
            other => panic!("Expected Redirect, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn https_enforcer_allows_localhost_http() {
        let pipeline = InterceptorPipeline::new(vec![Arc::new(HttpsEnforcer)]);
        let req = make_request("http://localhost:3000/api");
        assert!(matches!(pipeline.run(&req).await, InterceptorResult::Allow));
    }

    #[tokio::test]
    async fn https_enforcer_allows_https() {
        let pipeline = InterceptorPipeline::new(vec![Arc::new(HttpsEnforcer)]);
        let req = make_request("https://example.com");
        assert!(matches!(pipeline.run(&req).await, InterceptorResult::Allow));
    }
}
