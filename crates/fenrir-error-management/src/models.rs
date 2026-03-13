use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Fehler-Kategorien für die Klassifikation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Netzwerk-bezogene Fehler
    Network,
    /// Konfigurationsfehler
    Configuration,
    /// Eingabe/Ausgabe Fehler
    Io,
    /// Kryptographie-Fehler
    Crypto,
    /// Speicher/Storage Fehler
    Storage,
    /// Berechtigungsfehler
    Permission,
    /// AI/ML-bezogene Fehler
    Ai,
    /// Browser-spezifische Fehler (Servo, WebView, etc.)
    Browser,
    /// Unbekannte oder nicht klassifizierbare Fehler
    Unknown,
    /// Systemfehler (Betriebssystem, Hardware)
    System,
    /// Anwendungslogik-Fehler
    Application,
}

impl Default for ErrorCategory {
    fn default() -> Self {
        ErrorCategory::Unknown
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::Io => write!(f, "I/O"),
            ErrorCategory::Crypto => write!(f, "Cryptography"),
            ErrorCategory::Storage => write!(f, "Storage"),
            ErrorCategory::Permission => write!(f, "Permission"),
            ErrorCategory::Ai => write!(f, "AI/ML"),
            ErrorCategory::Browser => write!(f, "Browser"),
            ErrorCategory::Unknown => write!(f, "Unknown"),
            ErrorCategory::System => write!(f, "System"),
            ErrorCategory::Application => write!(f, "Application"),
        }
    }
}

/// Hauptstruktur für einen Fehler-Eintrag (id, message, hilfe)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    /// Eindeutige ID des Fehlers (UUID v4)
    pub id: String,
    /// Fehler-Nachricht (Beschreibung)
    pub message: String,
    /// Hilfe-Text für den Fehler (kann None sein)
    pub help: Option<String>,
    /// Kategorie des Fehlers
    pub category: ErrorCategory,
    /// Zeitpunkt der Erstellung
    pub created_at: DateTime<Utc>,
    /// Zeitpunkt der letzten Aktualisierung
    pub updated_at: DateTime<Utc>,
    /// Anzahl der Vorkommen dieses Fehlers
    pub occurrence_count: u32,
    /// Letzte Fehlermeldung (kann sich von message unterscheiden)
    pub last_message: String,
    /// Tags für zusätzliche Klassifikation
    pub tags: Vec<String>,
    /// Quelle/Modul, in dem der Fehler aufgetreten ist
    pub source: Option<String>,
}

impl ErrorRecord {
    /// Erstellt einen neuen ErrorRecord
    pub fn new(
        message: String,
        category: ErrorCategory,
        help: Option<String>,
        source: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.clone(),
            help,
            category,
            created_at: now,
            updated_at: now,
            occurrence_count: 1,
            last_message: message,
            tags,
            source,
        }
    }

    /// Aktualisiert den Fehler bei erneutem Auftreten
    pub fn update_occurrence(&mut self, new_message: Option<String>) {
        self.occurrence_count += 1;
        self.updated_at = Utc::now();
        
        if let Some(msg) = new_message {
            self.last_message = msg;
        }
    }

    /// Prüft, ob Hilfe vorhanden ist
    pub fn has_help(&self) -> bool {
        self.help.is_some()
    }

    /// Gibt die Hilfe zurück oder einen Platzhalter
    pub fn get_help_or_placeholder(&self) -> String {
        self.help.clone().unwrap_or_else(|| {
            format!("Keine Hilfe für Fehler '{}' verfügbar.", self.message)
        })
    }
}

/// Statistik-Daten für Fehler-Analyse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    /// Gesamtanzahl aller registrierten Fehler
    pub total_errors: u64,
    /// Anzahl der Fehler mit Hilfe
    pub errors_with_help: u64,
    /// Anzahl der Fehler ohne Hilfe
    pub errors_without_help: u64,
    /// Fehler nach Kategorie
    pub errors_by_category: Vec<(ErrorCategory, u64)>,
    /// Häufigste Fehler (Top 10)
    pub most_frequent_errors: Vec<(String, u32)>,
    /// Zeitliche Verteilung (letzte 30 Tage)
    pub daily_distribution: Vec<(String, u64)>,
}

/// AI-generierter Hilfe-Vorschlag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpSuggestion {
    /// Fehler-ID, für die der Vorschlag gilt
    pub error_id: String,
    /// AI-generierter Hilfe-Text
    pub suggestion: String,
    /// Konfidenz-Score der AI (0.0 bis 1.0)
    pub confidence: f32,
    /// Zeitpunkt der Generierung
    pub generated_at: DateTime<Utc>,
    /// Model-Name, der verwendet wurde
    pub model_name: String,
    /// Ob der Vorschlag vom Benutzer akzeptiert wurde
    pub accepted: bool,
}

impl HelpSuggestion {
    /// Erstellt einen neuen HelpSuggestion
    pub fn new(error_id: String, suggestion: String, confidence: f32, model_name: String) -> Self {
        Self {
            error_id,
            suggestion,
            confidence,
            generated_at: Utc::now(),
            model_name,
            accepted: false,
        }
    }
}
