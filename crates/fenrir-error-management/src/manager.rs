use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, warn, debug};

use crate::database::ErrorDatabase;
use crate::ai_helper::{AiErrorHelper, RuleBasedHelper};
use crate::models::{ErrorRecord, ErrorCategory, ErrorStatistics, HelpSuggestion};
use crate::error::ErrorManagementError;
use crate::ErrorManagementResult;

/// Haupt-Manager für das Fehler-Management-System
pub struct ErrorManager {
    database: Arc<ErrorDatabase>,
    ai_helper: Arc<Mutex<AiErrorHelper>>,
    rule_helper: RuleBasedHelper,
}

impl ErrorManager {
    /// Erstellt einen neuen ErrorManager
    pub async fn new(db_path: &str) -> ErrorManagementResult<Self> {
        let database = Arc::new(ErrorDatabase::new(db_path)?);
        let ai_helper = Arc::new(Mutex::new(AiErrorHelper::new().await));
        let rule_helper = RuleBasedHelper;
        
        info!("ErrorManager initialized with database: {}", db_path);
        
        Ok(Self {
            database,
            ai_helper,
            rule_helper,
        })
    }

    /// Registriert einen neuen Fehler
    pub async fn register_error(
        &self,
        message: &str,
        category: ErrorCategory,
        source: Option<&str>,
        tags: Vec<&str>,
    ) -> ErrorManagementResult<String> {
        let error_record = ErrorRecord::new(
            message.to_string(),
            category,
            None, // Keine vordefinierte Hilfe
            source.map(|s| s.to_string()),
            tags.iter().map(|&t| t.to_string()).collect(),
        );
        
        let error_id = self.database.register_error(&error_record)?;
        
        debug!("Error registered: {} - {}", error_id, message);
        
        Ok(error_id)
    }

    /// Holt Hilfe für einen Fehler (mit AI-Generierung falls nötig)
    pub async fn get_help(&self, error_id: &str) -> ErrorManagementResult<String> {
        // Fehler aus der Datenbank holen
        let error_record = match self.database.get_error(error_id)? {
            Some(record) => record,
            None => return Err(ErrorManagementError::NotFound(
                format!("Error with id {} not found", error_id)
            )),
        };
        
        // Prüfen, ob bereits Hilfe vorhanden ist
        if let Some(help) = error_record.help {
            return Ok(help);
        }
        
        // Regelbasierte Hilfe versuchen
        if let Some(rule_help) = self.rule_helper.generate_help(&error_record) {
            // Regelbasierte Hilfe in der Datenbank speichern
            self.database.update_error_help(error_id, &rule_help)?;
            return Ok(rule_help);
        }
        
        // AI-Hilfe generieren
        let ai_helper = self.ai_helper.lock().await;
        if ai_helper.is_available() {
            match ai_helper.generate_help_suggestion(&error_record).await {
                Ok(Some(suggestion)) => {
                    // AI-Vorschlag in der Datenbank speichern
                    self.database.add_help_suggestion(&suggestion)?;
                    
                    // Als aktuelle Hilfe setzen (mit niedriger Konfidenz-Markierung)
                    let ai_help = format!(
                        "[AI-Vorschlag - Konfidenz: {:.0}%]\n{}",
                        suggestion.confidence * 100.0,
                        suggestion.suggestion
                    );
                    
                    self.database.update_error_help(error_id, &ai_help)?;
                    return Ok(ai_help);
                }
                Ok(None) => {
                    warn!("AI helper available but returned no suggestion");
                }
                Err(e) => {
                    warn!("Failed to generate AI help: {}", e);
                }
            }
        }
        
        // Fallback: Standard-Nachricht
        Ok(format!(
            "Keine spezifische Hilfe für diesen Fehler verfügbar.\n\
            Fehler: {}\n\
            Kategorie: {}\n\
            Bitte versuchen Sie:\n\
            1. Den Browser neu zu starten\n\
            2. Die Aktion zu einem späteren Zeitpunkt zu wiederholen\n\
            3. Bei fortbestehendem Problem ein Issue auf GitHub zu erstellen",
            error_record.message,
            error_record.category
        ))
    }

    /// Sucht Fehler nach verschiedenen Kriterien
    pub async fn search_errors(
        &self,
        query: Option<&str>,
        category: Option<ErrorCategory>,
        limit: Option<u32>,
    ) -> ErrorManagementResult<Vec<ErrorRecord>> {
        self.database.search_errors(query, category, limit)
    }

    /// Holt Statistiken über alle Fehler
    pub async fn get_statistics(&self) -> ErrorManagementResult<ErrorStatistics> {
        self.database.get_statistics()
    }

    /// Fügt manuell Hilfe für einen Fehler hinzu
    pub async fn add_manual_help(&self, error_id: &str, help_text: &str) -> ErrorManagementResult<()> {
        self.database.update_error_help(error_id, help_text)
    }

    /// Holt alle Hilfe-Vorschläge für einen Fehler
    pub async fn get_help_suggestions(&self, error_id: &str) -> ErrorManagementResult<Vec<HelpSuggestion>> {
        self.database.get_help_suggestions(error_id)
    }

    /// Akzeptiert einen AI-Vorschlag als offizielle Hilfe
    pub async fn accept_help_suggestion(&self, _suggestion_id: i64) -> ErrorManagementResult<()> {
        // In einer realen Implementierung würde dies den Vorschlag als akzeptiert markieren
        // und möglicherweise als offizielle Hilfe setzen
        warn!("accept_help_suggestion not fully implemented yet");
        Ok(())
    }

    /// Konvertiert einen FenrirError in ein ErrorRecord und registriert ihn
    pub async fn register_fenrir_error(
        &self,
        fenrir_error: &fenrir_core::error::FenrirError,
        source: Option<&str>,
    ) -> ErrorManagementResult<String> {
        let (message, category) = match fenrir_error {
            fenrir_core::error::FenrirError::ServoInit(msg) => (
                format!("Servo initialization failed: {}", msg),
                ErrorCategory::Browser,
            ),
            fenrir_core::error::FenrirError::WebViewCreate(msg) => (
                format!("WebView creation failed: {}", msg),
                ErrorCategory::Browser,
            ),
            fenrir_core::error::FenrirError::InvalidUrl => (
                "Invalid URL".to_string(),
                ErrorCategory::Application,
            ),
            fenrir_core::error::FenrirError::Config(msg) => (
                format!("Configuration error: {}", msg),
                ErrorCategory::Configuration,
            ),
            fenrir_core::error::FenrirError::Crypto(msg) => (
                format!("Crypto error: {}", msg),
                ErrorCategory::Crypto,
            ),
            fenrir_core::error::FenrirError::Network(msg) => (
                format!("Network error: {}", msg),
                ErrorCategory::Network,
            ),
            fenrir_core::error::FenrirError::Storage(msg) => (
                format!("Storage error: {}", msg),
                ErrorCategory::Storage,
            ),
            fenrir_core::error::FenrirError::PermissionDenied(msg) => (
                format!("Permission denied: {}", msg),
                ErrorCategory::Permission,
            ),
            fenrir_core::error::FenrirError::Io(io_err) => (
                format!("IO error: {}", io_err),
                ErrorCategory::Io,
            ),
            fenrir_core::error::FenrirError::Ai(msg) => (
                format!("AI/ML error: {}", msg),
                ErrorCategory::Ai,
            ),
            fenrir_core::error::FenrirError::Candle(candle_err) => (
                format!("Candle ML error: {}", candle_err),
                ErrorCategory::Ai,
            ),
            fenrir_core::error::FenrirError::EngineNotRunning => (
                "Engine not running".to_string(),
                ErrorCategory::Browser,
            ),
            fenrir_core::error::FenrirError::TabNotFound(uuid) => (
                format!("Tab not found: {}", uuid),
                ErrorCategory::Browser,
            ),
        };
        
        let tags = vec!["fenrir-core"];
        
        self.register_error(&message, category, source, tags).await
    }
}
