//! Model Downloader — lädt noeum-1-nano von HuggingFace.
//!
//! Nutzt hf-hub (HuggingFace's offizieller Rust Client).
//! Cacht Modelle in ~/.cache/huggingface/ (Standard HF Cache).

use fenrir_core::error::FenrirError;
use hf_hub::{Repo, RepoType, api::tokio::Api};
use std::path::PathBuf;
use tracing::info;

pub const MODEL_REPO: &str = "noeum/noeum-1-nano";
pub const MODEL_FILES: &[&str] = &[
    "config.json",
    "generation_config.json",
    "tokenizer.model",
    "tokenizer_config.json",
    "special_tokens_map.json",
    "added_tokens.json",
    "pytorch_model.bin",
];

pub struct ModelDownloader {
    api: Api,
}

impl ModelDownloader {
    pub fn new() -> Self {
        Self {
            api: Api::new().expect("HF Hub API konnte nicht initialisiert werden"),
        }
    }

    /// Stellt sicher dass alle Modell-Dateien lokal vorhanden sind.
    /// Download nur wenn nötig (HF Hub cacht automatisch).
    pub async fn ensure_downloaded(&self) -> Result<PathBuf, FenrirError> {
        info!("Prüfe noeum-1-nano Modell-Cache");

        let repo = self.api.repo(Repo::new(
            MODEL_REPO.to_string(),
            RepoType::Model,
        ));

        let mut model_dir = None;

        for filename in MODEL_FILES {
            info!(file = filename, "Sicherstellen dass Datei vorhanden");
            let path = repo
                .get(filename)
                .await
                .map_err(|e| FenrirError::Network(
                    format!("Download fehlgeschlagen für '{filename}': {e}")
                ))?;

            // Alle Dateien liegen im gleichen Verzeichnis
            if model_dir.is_none() {
                model_dir = path.parent().map(|p| p.to_path_buf());
            }
        }

        let dir = model_dir
            .ok_or_else(|| FenrirError::Config("Modell-Verzeichnis nicht gefunden".into()))?;

        info!(dir = %dir.display(), "Noeum-1-Nano vollständig verfügbar");
        Ok(dir)
    }

    /// Gibt das Cache-Verzeichnis zurück ohne Download.
    pub fn cache_dir(&self) -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("~/.cache"))
            .join("huggingface")
            .join("hub")
    }

    /// Ist das Modell bereits vollständig gecacht?
    pub async fn is_cached(&self) -> bool {
        let repo = self.api.repo(Repo::new(
            MODEL_REPO.to_string(),
            RepoType::Model,
        ));
        // Prüfe nur config.json als schnellen Check
        repo.get("config.json").await.is_ok()
    }
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new()
    }
}
