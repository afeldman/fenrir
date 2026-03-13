//! fenrir-config — Browser-Einstellungen, Profile und Feature Flags.
//!
//! Konfiguration liegt in `~/.config/fenrir/config.toml`.
//! Beim ersten Start wird ein Default-Config erstellt.

pub mod features;
pub mod profile;
pub mod settings;

pub use features::FeatureFlags;
pub use settings::{FenrirConfig, SearchEngine};

use fenrir_core::error::FenrirError;
use std::path::PathBuf;
use tracing::{debug, info};

/// Lädt Config aus Disk oder erstellt Default.
pub fn load() -> Result<FenrirConfig, FenrirError> {
    let path = config_path();
    if path.exists() {
        debug!(path = %path.display(), "Lade Config");
        settings::load_from_file(&path)
    } else {
        info!(path = %path.display(), "Erstelle Default-Config");
        let config = FenrirConfig::default();
        save(&config)?;
        Ok(config)
    }
}

/// Speichert Config auf Disk.
pub fn save(config: &FenrirConfig) -> Result<(), FenrirError> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    settings::save_to_file(config, &path)
}

pub fn config_path() -> PathBuf {
    dirs::config_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fenrir")
        .join("config.toml")
}
