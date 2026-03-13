//! fenrir-error-management — Fehler-Management-System für Fenrir Browser.
//!
//! Dieses Modul bietet ein umfassendes System zur Verwaltung aller auftretenden Fehler
//! mit der Struktur (id, message, hilfe). Wenn keine Hilfe gefunden wird, wird
//! Fenrir-AI genutzt, um einen Vorschlag zu machen.
//!
//! # Features
//! - Strukturierte Fehler-Datenbank mit SQLite
//! - Automatische Fehler-Klassifikation
//! - AI-Integration für Hilfe-Vorschläge
//! - Fehler-Statistiken und -Tracking
//! - Thread-safe Design
//!
//! # Beispiel
//! ```no_run
//! use fenrir_error_management::{ErrorManager, ErrorRecord, ErrorCategory};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // ErrorManager initialisieren
//!     let manager = ErrorManager::new(":memory:").await?;
//!
//!     // Fehler registrieren
//!     let error_id = manager.register_error(
//!         "network_timeout",
//!         "Network request timed out after 30 seconds",
//!         ErrorCategory::Network,
//!         None, // Keine vordefinierte Hilfe
//!     ).await?;
//!
//!     // Hilfe für Fehler abrufen (mit AI-Generierung falls nötig)
//!     let help = manager.get_help(&error_id).await?;
//!     println!("Hilfe für Fehler {}: {}", error_id, help);
//!
//!     Ok(())
//! }
//! ```

pub mod database;
pub mod error;
pub mod manager;
pub mod models;
pub mod ai_helper;

pub use database::ErrorDatabase;
pub use error::ErrorManagementError;
pub use manager::ErrorManager;
pub use models::{ErrorRecord, ErrorCategory, ErrorStatistics, HelpSuggestion};

/// Result type für das Error-Management-System
pub type ErrorManagementResult<T> = Result<T, ErrorManagementError>;
