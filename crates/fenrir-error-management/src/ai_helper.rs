use log::{info, warn, error};
use crate::models::{ErrorRecord, HelpSuggestion};
use crate::error::ErrorManagementError;
use crate::ErrorManagementResult;

/// AI-Helper für die Generierung von Hilfe-Vorschlägen
#[cfg(feature = "ai-integration")]
pub struct AiErrorHelper {
    /// AI-Engine für die Generierung von Hilfe
    ai_engine: Option<fenrir_ai::NoeumEngine>,
}

#[cfg(feature = "ai-integration")]
impl AiErrorHelper {
    /// Erstellt einen neuen AI-Helper
    pub async fn new() -> Self {
        // Versuche, die AI-Engine zu laden
        let ai_engine = match fenrir_ai::NoeumEngine::load().await {
            Ok(engine) => {
                info!("AI error helper initialized with Noeum engine");
                Some(engine)
            }
            Err(e) => {
                warn!("Failed to load AI engine for error helper: {}", e);
                None
            }
        };
        
        Self { ai_engine }
    }

    /// Generiert einen Hilfe-Vorschlag für einen Fehler
    pub async fn generate_help_suggestion(
        &self,
        error_record: &ErrorRecord,
    ) -> ErrorManagementResult<Option<HelpSuggestion>> {
        if self.ai_engine.is_none() {
            return Ok(None);
        }
        
        let engine = self.ai_engine.as_ref().unwrap();
        
        // Prompt für die AI erstellen
        let prompt = format!(
            "Du bist ein hilfreicher Assistent für den Fenrir Browser. \
            Ein Benutzer hat folgenden Fehler erhalten:\n\n\
            Fehler: {}\n\
            Kategorie: {}\n\
            Quelle: {}\n\
            Tags: {}\n\n\
            Bitte gib eine hilfreiche Erklärung und Lösung für diesen Fehler. \
            Sei präzise und praktisch. Wenn möglich, gib Schritt-für-Schritt-Anleitungen. \
            Antworte auf Deutsch.",
            error_record.message,
            error_record.category,
            error_record.source.as_deref().unwrap_or("Unbekannt"),
            error_record.tags.join(", ")
        );
        
        // AI-Antwort generieren
        let request = fenrir_ai::InferenceRequest {
            prompt,
            thinking: Some(()),
            max_tokens: Some(512),
        };
        
        match engine.complete(request).await {
            Ok(response) => {
                let suggestion = HelpSuggestion::new(
                    error_record.id.clone(),
                    response.text,
                    0.8, // Konfidenz-Score (kann später verfeinert werden)
                    "noeum-1-nano".to_string(),
                );
                
                info!("AI help suggestion generated for error: {}", error_record.id);
                Ok(Some(suggestion))
            }
            Err(e) => {
                error!("Failed to generate AI help suggestion: {}", e);
                Err(ErrorManagementError::AiIntegration(format!("AI generation failed: {}", e)))
            }
        }
    }

    /// Prüft, ob die AI verfügbar ist
    pub fn is_available(&self) -> bool {
        self.ai_engine.is_some()
    }
}

/// Dummy-Implementierung für wenn AI-Integration nicht aktiviert ist
#[cfg(not(feature = "ai-integration"))]
pub struct AiErrorHelper;

#[cfg(not(feature = "ai-integration"))]
impl AiErrorHelper {
    /// Erstellt einen neuen AI-Helper (Dummy)
    pub async fn new() -> Self {
        warn!("AI integration feature is not enabled");
        Self
    }

    /// Generiert einen Hilfe-Vorschlag für einen Fehler (Dummy)
    pub async fn generate_help_suggestion(
        &self,
        _error_record: &ErrorRecord,
    ) -> ErrorManagementResult<Option<HelpSuggestion>> {
        warn!("AI integration feature is not enabled - cannot generate help suggestions");
        Ok(None)
    }

    /// Prüft, ob die AI verfügbar ist (immer false ohne Feature)
    pub fn is_available(&self) -> bool {
        false
    }
}

/// Einfache regelbasierte Hilfe-Generierung (Fallback wenn AI nicht verfügbar)
pub struct RuleBasedHelper;

impl RuleBasedHelper {
    /// Generiert eine regelbasierte Hilfe für häufige Fehler
    pub fn generate_help(&self, error_record: &ErrorRecord) -> Option<String> {
        // Einfache regelbasierte Hilfe für bekannte Fehlermuster
        let lower_message = error_record.message.to_lowercase();
        
        if lower_message.contains("network") && lower_message.contains("timeout") {
            Some(
                "Netzwerk-Timeout: Die Verbindung zum Server wurde unterbrochen.\n\
                - Prüfen Sie Ihre Internetverbindung\n\
                - Versuchen Sie es später erneut\n\
                - Wenn das Problem besteht, könnte der Server überlastet sein".to_string()
            )
        } else if lower_message.contains("permission") && lower_message.contains("denied") {
            Some(
                "Berechtigungsfehler: Der Zugriff wurde verweigert.\n\
                - Stellen Sie sicher, dass Sie die notwendigen Berechtigungen haben\n\
                - Prüfen Sie die Datei- oder Ordnerberechtigungen\n\
                - Starten Sie den Browser neu und versuchen Sie es erneut".to_string()
            )
        } else if lower_message.contains("invalid") && lower_message.contains("url") {
            Some(
                "Ungültige URL: Die eingegebene Adresse ist nicht korrekt.\n\
                - Prüfen Sie die Schreibweise der URL\n\
                - Stellen Sie sicher, dass die URL mit http:// oder https:// beginnt\n\
                - Versuchen Sie, die URL in einem anderen Browser zu öffnen".to_string()
            )
        } else if lower_message.contains("storage") && (lower_message.contains("full") || lower_message.contains("space")) {
            Some(
                "Speicherplatz fehlt: Nicht genügend Speicherplatz verfügbar.\n\
                - Löschen Sie nicht benötigte Dateien\n\
                - Prüfen Sie den verfügbaren Speicherplatz\n\
                - Versuchen Sie, den Browser-Cache zu leeren".to_string()
            )
        } else if lower_message.contains("config") && lower_message.contains("error") {
            Some(
                "Konfigurationsfehler: Die Einstellungen sind nicht korrekt.\n\
                - Prüfen Sie die Browser-Einstellungen\n\
                - Setzen Sie die Einstellungen auf Standardwerte zurück\n\
                - Starten Sie den Browser neu".to_string()
            )
        } else {
            None
        }
    }
}
