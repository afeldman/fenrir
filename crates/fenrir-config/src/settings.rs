//! Haupt-Konfigurationsstruktur.

use crate::features::FeatureFlags;
use fenrir_core::error::FenrirError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FenrirConfig {
    /// Browser-UI Einstellungen
    pub ui: UiSettings,
    /// Netzwerk-Einstellungen
    pub network: NetworkSettings,
    /// Privacy-Einstellungen
    pub privacy: PrivacySettings,
    /// AI/LLM Einstellungen
    pub ai: AiSettings,
    /// Feature Flags (experimentell)
    pub features: FeatureFlags,
    /// Aktives Nutzerprofil
    pub active_profile: String,
}

impl Default for FenrirConfig {
    fn default() -> Self {
        Self {
            ui: UiSettings::default(),
            network: NetworkSettings::default(),
            privacy: PrivacySettings::default(),
            ai: AiSettings::default(),
            features: FeatureFlags::default(),
            active_profile: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    pub start_url: String,
    pub window_width: u32,
    pub window_height: u32,
    pub theme: Theme,
    pub toolbar_height: f32,
    pub language: String, // Language code like "en-US", "de-DE"
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            // lite.duckduckgo.com: reines HTML, kein JS-Framework, Servo-kompatibel
            start_url: "https://duckduckgo.com".to_string(),
            window_width: 1280,
            window_height: 800,
            theme: Theme::System,
            toolbar_height: 40.0,
            language: "en-US".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkSettings {
    pub doh_server: String,
    pub enforce_https: bool,
    pub proxy: Option<String>,
    pub user_agent: String,
}

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            doh_server: "https://cloudflare-dns.com/dns-query".to_string(),
            enforce_https: true,
            proxy: None,
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:140.0) Servo/0.0.5 Fenrir/0.1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivacySettings {
    pub block_trackers: bool,
    pub block_ads: bool,
    pub resist_fingerprinting: bool,
    pub clear_cookies_on_exit: bool,
    pub search_engine: SearchEngine,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            block_trackers: true,
            block_ads: false, // Phase 2
            resist_fingerprinting: true,
            clear_cookies_on_exit: false,
            search_engine: SearchEngine::DuckDuckGo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchEngine {
    #[default]
    DuckDuckGo,
    Brave,
    Kagi,
    Custom(String),
}

impl SearchEngine {
    pub fn search_url(&self, query: &str) -> String {
        let q = urlencoding::encode(query);
        match self {
            Self::DuckDuckGo => format!("https://duckduckgo.com/?q={q}"),
            Self::Brave      => format!("https://search.brave.com/search?q={q}"),
            Self::Kagi       => format!("https://kagi.com/search?q={q}"),
            Self::Custom(u)  => format!("{u}{q}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiSettings {
    pub enabled: bool,
    pub model_id: String,
    pub model_dir: Option<String>, // None → auto (~/.local/share/fenrir/models/)
    pub auto_download: bool,
    pub max_tokens: usize,
    pub bookmark_categorization: bool,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            model_id: "noeum/noeum-1-nano".to_string(),
            model_dir: None,
            auto_download: false, // expliziter User-Consent nötig
            max_tokens: 256,
            bookmark_categorization: true,
        }
    }
}

pub fn load_from_file(path: &Path) -> Result<FenrirConfig, FenrirError> {
    let content = std::fs::read_to_string(path)?;
    toml::from_str(&content).map_err(|e| FenrirError::Config(e.to_string()))
}

pub fn save_to_file(config: &FenrirConfig, path: &Path) -> Result<(), FenrirError> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| FenrirError::Config(e.to_string()))?;
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = FenrirConfig::default();
        assert!(!config.ui.start_url.is_empty());
        assert!(config.network.enforce_https);
        assert!(config.privacy.block_trackers);
        assert!(!config.ai.auto_download);
    }

    #[test]
    fn config_roundtrip_toml() {
        let config = FenrirConfig::default();
        let toml = toml::to_string_pretty(&config).unwrap();
        let parsed: FenrirConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.ui.start_url, config.ui.start_url);
        assert_eq!(parsed.ai.model_id, config.ai.model_id);
    }

    #[test]
    fn search_engine_urls() {
        assert!(SearchEngine::DuckDuckGo.search_url("test").contains("duckduckgo"));
        assert!(SearchEngine::Brave.search_url("test").contains("brave"));
    }
}
