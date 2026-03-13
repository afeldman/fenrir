use crate::download::ModelDownloader;
use crate::{InferenceRequest, InferenceResponse, InferenceEngine};

use fenrir_core::error::FenrirError;
use std::path::PathBuf;
use tracing::info;

/// Noeum-1-Nano Engine — Haupt-Einstiegspunkt.
pub struct NoeumEngine {
    inner: InferenceEngine,
}

impl NoeumEngine {
    /// Modell von HuggingFace laden (Download wenn nicht vorhanden).
    pub async fn load() -> Result<Self, FenrirError> {
        let downloader = ModelDownloader::new();
        let model_dir = downloader.ensure_downloaded().await?;
        Self::from_dir(model_dir).await
    }

    /// Aus lokalem Verzeichnis laden (kein Download).
    pub async fn from_dir(dir: PathBuf) -> Result<Self, FenrirError> {
        info!(dir = %dir.display(), "Lade Noeum-1-Nano Modell");
        let inner = InferenceEngine::load(dir).await?;
        info!("Noeum-1-Nano geladen ✓");
        Ok(Self { inner })
    }

    /// Text-Completion — mit optionalem Thinking Mode.
    pub async fn complete(&mut self, request: InferenceRequest) -> Result<InferenceResponse, FenrirError> {
        self.inner.complete(request).await
    }

    /// Ist das Modell im Speicher geladen?
    pub fn is_loaded(&self) -> bool {
        true
    }
}
