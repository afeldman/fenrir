//! Model Downloader — lädt noeum-1-nano von HuggingFace.
//!
//! Nutzt hf-hub (HuggingFace's offizieller Rust Client).
//! Speichert Modelle in ~/.local/share/fenrir/models/noeum-1-nano/

use fenrir_core::error::FenrirError;
use hf_hub::{Repo, RepoType, api::tokio::Api};
use std::path::PathBuf;
use tracing::info;

pub struct ModelDownloader {
    model_id: String,
    local_dir: PathBuf,
}

impl ModelDownloader {
    pub fn new() -> Self {
        Self {
            model_id: "noeum/noeum-1-nano".to_string(),
            local_dir: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("fenrir/models/noeum-1-nano"),
        }
    }

    /// Lädt Modell herunter wenn nicht vorhanden. Gibt Verzeichnis zurück.
    pub async fn ensure_downloaded(&self) -> Result<PathBuf, FenrirError> {
        // Prüfen ob bereits vorhanden
        let model_path = self.local_dir.join("pytorch_model.bin");
        if model_path.exists() {
            info!(path = %self.local_dir.display(), "Modell bereits vorhanden");
            return Ok(self.local_dir.clone());
        }

        info!(model = %self.model_id, "Lade noeum-1-nano von HuggingFace...");
        std::fs::create_dir_all(&self.local_dir)
            .map_err(|e| FenrirError::Io(e))?;

        let api = Api::new().map_err(|e| FenrirError::Network(e.to_string()))?;
        let repo = api.repo(Repo::new(self.model_id.clone(), RepoType::Model));

        for filename in &[
            "config.json",
            "tokenizer_config.json",
            "tokenizer.model",
            "pytorch_model.bin",
            "generation_config.json",
        ] {
            info!(file = filename, "Download...");
            let path = repo.get(filename).await
                .map_err(|e| FenrirError::Network(format!("Download fehlgeschlagen für '{filename}': {e}")))?;
            let dest = self.local_dir.join(filename);
            std::fs::copy(&path, &dest)
                .map_err(|e| FenrirError::Io(e))?;
        }

        info!(path = %self.local_dir.display(), "Download abgeschlossen");
        Ok(self.local_dir.clone())
    }
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new()
    }
}
