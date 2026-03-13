//! LocalLlmProvider — Stub für lokales LLM (Phi-3 Mini via candle).
//! Phase 3: echte candle-Integration.

use crate::provider::{LlmProvider, LlmRequest, LlmResponse, ProviderInfo, ProviderKind};
use async_trait::async_trait;
use fenrir_core::error::FenrirError;

pub struct LocalLlmProvider {
    /// Modellname (z.B. "phi-3-mini", "mistral-7b-q4")
    model_name: String,
    /// Ist das Modell geladen?
    loaded: bool,
}

impl LocalLlmProvider {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
            loaded: false,
        }
    }
}

#[async_trait]
impl LlmProvider for LocalLlmProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            kind: ProviderKind::Local,
            name: "local-llm",
            supports_streaming: false, // Phase 3: true
        }
    }

    async fn is_available(&self) -> bool {
        // Phase 3: prüfen ob Modell-Datei existiert und geladen werden kann
        // Jetzt: immer false damit Fallback auf mammoth.ai greift
        self.loaded
    }

    async fn complete(&self, _request: LlmRequest) -> Result<LlmResponse, FenrirError> {
        // Phase 3: candle inference hier
        Err(FenrirError::Config(
            format!("Lokales Modell '{}' noch nicht implementiert (Phase 3)", self.model_name)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn local_provider_not_available_without_model() {
        let provider = LocalLlmProvider::new("phi-3-mini");
        assert!(!provider.is_available().await);
    }

    #[tokio::test]
    async fn local_provider_info() {
        let provider = LocalLlmProvider::new("phi-3-mini");
        assert_eq!(provider.info().kind, ProviderKind::Local);
        assert_eq!(provider.info().name, "local-llm");
    }
}
