use thiserror::Error;

/// Fehler-Typen für das Error-Management-System
#[derive(Debug, Error)]
pub enum ErrorManagementError {
    /// Datenbank-Fehler
    #[error("Database error: {0}")]
    Database(String),

    /// IO-Fehler
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialisierungs-/Deserialisierungsfehler
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Fehler bei der AI-Integration
    #[error("AI integration error: {0}")]
    AiIntegration(String),

    /// Fehler nicht gefunden
    #[error("Error not found: {0}")]
    NotFound(String),

    /// Ungültige Eingabe
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Konfigurationsfehler
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Systemfehler
    #[error("System error: {0}")]
    System(String),
}

impl From<rusqlite::Error> for ErrorManagementError {
    fn from(err: rusqlite::Error) -> Self {
        ErrorManagementError::Database(format!("SQLite error: {}", err))
    }
}

impl From<serde_json::Error> for ErrorManagementError {
    fn from(err: serde_json::Error) -> Self {
        ErrorManagementError::Serialization(format!("JSON error: {}", err))
    }
}
