//! Internationalization (i18n) support for Fenrir Browser.
//!
//! This module provides localization support using Fluent (Mozilla's localization system).
//! It supports loading translations from locale files and provides a simple API for
//! retrieving localized strings.

use std::collections::HashMap;
use std::sync::Arc;

use fluent::{FluentArgs, FluentBundle, FluentResource};
use fluent_fallback::Localization;
use once_cell::sync::Lazy;
use thiserror::Error;
use tracing::{debug, error, info};
use unic_langid::{langid, LanguageIdentifier};

/// Errors that can occur during i18n operations.
#[derive(Debug, Error)]
pub enum I18nError {
    #[error("Failed to load locale resource: {0}")]
    ResourceLoad(String),
    
    #[error("Failed to parse locale: {0}")]
    LocaleParse(String),
    
    #[error("Translation not found for key: {0}")]
    TranslationNotFound(String),
    
    #[error("Invalid language identifier: {0}")]
    InvalidLanguage(String),
}

/// Supported languages in Fenrir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Language {
    /// English (United States)
    #[serde(rename = "en-US")]
    EnUs,
    
    /// German (Germany)
    #[serde(rename = "de-DE")]
    DeDe,
    
    /// French (France)
    #[serde(rename = "fr-FR")]
    FrFr,
    
    /// Spanish (Spain)
    #[serde(rename = "es-ES")]
    EsEs,
    
    /// Japanese (Japan)
    #[serde(rename = "ja-JP")]
    JaJp,
    
    /// Chinese (Simplified, China)
    #[serde(rename = "zh-CN")]
    ZhCn,
}

impl Language {
    /// Get the LanguageIdentifier for this language.
    pub fn langid(&self) -> LanguageIdentifier {
        match self {
            Language::EnUs => langid!("en-US"),
            Language::DeDe => langid!("de-DE"),
            Language::FrFr => langid!("fr-FR"),
            Language::EsEs => langid!("es-ES"),
            Language::JaJp => langid!("ja-JP"),
            Language::ZhCn => langid!("zh-CN"),
        }
    }
    
    /// Get the display name of the language in its own language.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::EnUs => "English",
            Language::DeDe => "Deutsch",
            Language::FrFr => "Français",
            Language::EsEs => "Español",
            Language::JaJp => "日本語",
            Language::ZhCn => "简体中文",
        }
    }
    
    /// Get all available languages.
    pub fn all() -> Vec<Language> {
        vec![
            Language::EnUs,
            Language::DeDe,
            Language::FrFr,
            Language::EsEs,
            Language::JaJp,
            Language::ZhCn,
        ]
    }
    
    /// Try to parse a language from a string.
    pub fn from_str(s: &str) -> Result<Self, I18nError> {
        match s.to_lowercase().as_str() {
            "en" | "en-us" | "en_us" => Ok(Language::EnUs),
            "de" | "de-de" | "de_de" => Ok(Language::DeDe),
            "fr" | "fr-fr" | "fr_fr" => Ok(Language::FrFr),
            "es" | "es-es" | "es_es" => Ok(Language::EsEs),
            "ja" | "ja-jp" | "ja_jp" => Ok(Language::JaJp),
            "zh" | "zh-cn" | "zh_cn" => Ok(Language::ZhCn),
            _ => Err(I18nError::InvalidLanguage(s.to_string())),
        }
    }
    
    /// Get the system default language.
    pub fn system_default() -> Self {
        // Try to detect system language
        // For now, default to English
        Language::EnUs
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::system_default()
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Main i18n manager that holds all translations.
pub struct I18nManager {
    bundles: HashMap<LanguageIdentifier, FluentBundle<FluentResource>>,
    current_language: Language,
}

impl I18nManager {
    /// Create a new I18nManager with the specified language.
    pub fn new(language: Language) -> Result<Self, I18nError> {
        let mut bundles = HashMap::new();
        
        // Load the requested language
        let langid = language.langid();
        if let Some(bundle) = Self::load_bundle(&langid)? {
            bundles.insert(langid.clone(), bundle);
        }
        
        // Also load English as fallback
        if language != Language::EnUs {
            let en_langid = Language::EnUs.langid();
            if !bundles.contains_key(&en_langid) {
                if let Some(bundle) = Self::load_bundle(&en_langid)? {
                    bundles.insert(en_langid, bundle);
                }
            }
        }
        
        Ok(Self {
            bundles,
            current_language: language,
        })
    }
    
    /// Load a Fluent bundle for a specific language.
    fn load_bundle(langid: &LanguageIdentifier) -> Result<Option<FluentBundle<FluentResource>>, I18nError> {
        // Try to load the locale file
        let resource_path = format!("locales/{}/fenrir.ftl", langid);
        
        // For now, we'll embed the translations in the binary
        // In the future, we can load from files
        let ftl_content = match langid.to_string().as_str() {
            "en-US" => include_str!("../locales/en-US/fenrir.ftl"),
            "de-DE" => include_str!("../locales/de-DE/fenrir.ftl"),
            _ => {
                // Fallback to English if locale not available
                debug!("Locale not available for {}, falling back to English", langid);
                return Ok(None);
            }
        };
        
        let resource = FluentResource::try_new(ftl_content.to_string())
            .map_err(|e| I18nError::ResourceLoad(format!("Failed to parse FTL for {}: {}", langid, e)))?;
        
        let mut bundle = FluentBundle::new(vec![langid.clone()]);
        bundle.add_resource(resource)
            .map_err(|e| I18nError::ResourceLoad(format!("Failed to add resource for {}: {}", langid, e)))?;
        
        Ok(Some(bundle))
    }
    
    /// Get a localized string for the current language.
    pub fn get(&self, key: &str) -> Result<String, I18nError> {
        self.get_with_args(key, None)
    }
    
    /// Get a localized string with arguments.
    pub fn get_with_args(&self, key: &str, args: Option<&FluentArgs>) -> Result<String, I18nError> {
        // Try current language first
        let langid = self.current_language.langid();
        if let Some(bundle) = self.bundles.get(&langid) {
            if let Some(message) = bundle.get_message(key) {
                if let Some(pattern) = message.value() {
                    let mut errors = Vec::new();
                    let result = bundle.format_pattern(pattern, args, &mut errors);
                    
                    if !errors.is_empty() {
                        error!("Errors formatting message '{}': {:?}", key, errors);
                    }
                    
                    return Ok(result.to_string());
                }
            }
        }
        
        // Fallback to English
        if self.current_language != Language::EnUs {
            let en_langid = Language::EnUs.langid();
            if let Some(bundle) = self.bundles.get(&en_langid) {
                if let Some(message) = bundle.get_message(key) {
                    if let Some(pattern) = message.value() {
                        let mut errors = Vec::new();
                        let result = bundle.format_pattern(pattern, args, &mut errors);
                        
                        if !errors.is_empty() {
                            error!("Errors formatting English fallback for '{}': {:?}", key, errors);
                        }
                        
                        return Ok(result.to_string());
                    }
                }
            }
        }
        
        Err(I18nError::TranslationNotFound(key.to_string()))
    }
    
    /// Change the current language.
    pub fn set_language(&mut self, language: Language) -> Result<(), I18nError> {
        let langid = language.langid();
        
        // Load the new language if not already loaded
        if !self.bundles.contains_key(&langid) {
            if let Some(bundle) = Self::load_bundle(&langid)? {
                self.bundles.insert(langid.clone(), bundle);
            }
        }
        
        self.current_language = language;
        info!("Changed language to {}", language);
        Ok(())
    }
    
    /// Get the current language.
    pub fn current_language(&self) -> Language {
        self.current_language
    }
}

/// Global i18n instance.
static I18N: Lazy<std::sync::Mutex<I18nManager>> = Lazy::new(|| {
    // Default to system language or English
    let language = Language::system_default();
    let manager = I18nManager::new(language)
        .unwrap_or_else(|e| {
            error!("Failed to initialize i18n: {}, falling back to English", e);
            I18nManager::new(Language::EnUs).expect("English should always work")
        });
    std::sync::Mutex::new(manager)
});

/// Get a localized string.
pub fn t(key: &str) -> String {
    match I18N.lock().unwrap().get(key) {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to get translation for '{}': {}", key, e);
            format!("[{}]", key)
        }
    }
}

/// Get a localized string with arguments.
pub fn t_with_args(key: &str, args: &FluentArgs) -> String {
    match I18N.lock().unwrap().get_with_args(key, Some(args)) {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to get translation for '{}' with args: {}", key, e);
            format!("[{}]", key)
        }
    }
}

/// Change the current language.
pub fn set_language(language: Language) -> Result<(), I18nError> {
    I18N.lock().unwrap().set_language(language)
}

/// Get the current language.
pub fn current_language() -> Language {
    I18N.lock().unwrap().current_language()
}

/// Get all available languages.
pub fn available_languages() -> Vec<Language> {
    Language::all()
}
