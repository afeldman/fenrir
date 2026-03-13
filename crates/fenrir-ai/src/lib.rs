//! fenrir-ai — Lokale LLM Inference für Fenrir Browser.
//!
//! Implementiert das Noeum-1-Nano Modell (Ramo-Architektur) nativ in Rust/Candle.
//!
//! # Modell: noeum/noeum-1-nano
//! - Herkunft: Österreich (EU-native)
//! - Architektur: Ramo (MoE + GQA + QK-Norm + YaRN-RoPE)
//! - Größe: 0.6B total / ~0.2B aktiv
//! - Lizenz: Apache 2.0
//! - Thinking Mode: `/think` → System-2 Reasoning
//!
//! # Nutzung
//! ```ignore
//! use fenrir_ai::{NoeumEngine, InferenceRequest, ThinkingConfig};
//!
//! let engine = NoeumEngine::load().await?;
//! let response = engine.complete(InferenceRequest {
//!     prompt: "Was ist Privacy?".into(),
//!     thinking: Some(ThinkingConfig { budget_tokens: 128 }),
//!     ..Default::default()
//! }).await?;
//! ```

pub mod download;
pub mod inference;
pub mod model;
pub mod tokenizer;

pub use download::ModelDownloader;
pub use inference::{InferenceRequest, InferenceResponse, ThinkingConfig};

use fenrir_core::error::FenrirError;
use std::path::PathBuf;
use tracing::info;

/// Noeum-1-Nano Engine — Haupt-Einstiegspunkt.
///
/// Hält Modell + Tokenizer geladen. Thread-sicher (Arc<Mutex<>> intern).
pub struct NoeumEngine {
    inner: inference::InferenceEngine,
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
        let inner = inference::InferenceEngine::load(dir).await?;
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
