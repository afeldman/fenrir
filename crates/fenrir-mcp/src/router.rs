//! LlmRouter — wählt automatisch den besten verfügbaren Provider.
//!
//! Priorität: Lokal > Ollama > Mammoth > Custom
//! User kann Priorität in fenrir-config überschreiben.

use crate::provider::{LlmProvider, LlmRequest, LlmResponse, ProviderKind};
use fenrir_core::error::FenrirError;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub struct LlmRouter {
    /// Provider in Prioritätsreihenfolge (Index 0 = höchste Priorität).
    providers: Vec<Arc<dyn LlmProvider>>,
}

impl LlmRouter {
    pub fn new(providers: Vec<Arc<dyn LlmProvider>>) -> Self {
        Self { providers }
    }

    /// Builder-Methode: Provider hinzufügen.
    pub fn with_provider(mut self, provider: Arc<dyn LlmProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    /// Wählt den ersten verfügbaren Provider und führt den Request durch.
    ///
    /// Versucht Provider in Prioritätsreihenfolge. Schlägt keiner an → Fehler.
    pub async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, FenrirError> {
        if self.providers.is_empty() {
            return Err(FenrirError::Config(
                "Kein LLM-Provider konfiguriert".into(),
            ));
        }

        for provider in &self.providers {
            let info = provider.info();
            debug!(provider = info.name, "Prüfe Provider-Verfügbarkeit");

            if provider.is_available().await {
                info!(provider = info.name, "Sende Request an Provider");
                return provider.complete(request).await;
            } else {
                warn!(provider = info.name, "Provider nicht verfügbar, versuche nächsten");
            }
        }

        Err(FenrirError::Network(
            "Kein LLM-Provider verfügbar (weder lokal noch Cloud)".into(),
        ))
    }

    /// Gibt alle konfigurierten Provider-Infos zurück (für UI/Settings).
    pub async fn available_providers(&self) -> Vec<ProviderKind> {
        let mut available = Vec::new();
        for provider in &self.providers {
            if provider.is_available().await {
                available.push(provider.info().kind);
            }
        }
        available
    }
}
