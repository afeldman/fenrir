use rusqlite::{Connection, params, OptionalExtension};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::sync::Arc;
use log::{info, warn, error};

use crate::models::{ErrorRecord, ErrorCategory, ErrorStatistics, HelpSuggestion};
use crate::error::ErrorManagementError;
use crate::ErrorManagementResult;

/// SQLite-Datenbank für Fehler-Management
pub struct ErrorDatabase {
    connection: Arc<RwLock<Connection>>,
}

impl ErrorDatabase {
    /// Erstellt eine neue Datenbank-Verbindung
    pub fn new(db_path: &str) -> ErrorManagementResult<Self> {
        let connection = Connection::open(db_path)?;
        
        // Datenbank-Schema initialisieren
        Self::init_schema(&connection)?;
        
        info!("Error database initialized at: {}", db_path);
        
        Ok(Self {
            connection: Arc::new(RwLock::new(connection)),
        })
    }

    /// Initialisiert das Datenbank-Schema
    fn init_schema(conn: &Connection) -> ErrorManagementResult<()> {
        // Tabelle für Fehler-Einträge
        conn.execute(
            "CREATE TABLE IF NOT EXISTS errors (
                id TEXT PRIMARY KEY,
                message TEXT NOT NULL,
                help TEXT,
                category TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                occurrence_count INTEGER NOT NULL DEFAULT 1,
                last_message TEXT NOT NULL,
                tags TEXT, -- JSON-Array
                source TEXT
            )",
            [],
        )?;

        // Tabelle für Hilfe-Vorschläge (AI-generiert)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS help_suggestions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                error_id TEXT NOT NULL,
                suggestion TEXT NOT NULL,
                confidence REAL NOT NULL,
                generated_at TEXT NOT NULL,
                model_name TEXT NOT NULL,
                accepted BOOLEAN NOT NULL DEFAULT 0,
                FOREIGN KEY (error_id) REFERENCES errors(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Indexe für performante Abfragen
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_errors_category ON errors(category)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_errors_created_at ON errors(created_at)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_help_suggestions_error_id ON help_suggestions(error_id)",
            [],
        )?;

        Ok(())
    }

    /// Registriert einen neuen Fehler oder aktualisiert einen bestehenden
    pub fn register_error(&self, error: &ErrorRecord) -> ErrorManagementResult<String> {
        let conn = self.connection.write();
        
        // Prüfen, ob ähnlicher Fehler bereits existiert
        let existing_id: Option<String> = conn.query_row(
            "SELECT id FROM errors WHERE message = ?1 LIMIT 1",
            params![error.message],
            |row| row.get(0),
        ).optional()?;

        match existing_id {
            Some(id) => {
                // Bestehenden Fehler aktualisieren
                conn.execute(
                    "UPDATE errors SET 
                        occurrence_count = occurrence_count + 1,
                        updated_at = ?1,
                        last_message = ?2
                     WHERE id = ?3",
                    params![
                        error.updated_at.to_rfc3339(),
                        error.last_message,
                        id
                    ],
                )?;
                Ok(id)
            }
            None => {
                // Neuen Fehler einfügen
                let tags_json = serde_json::to_string(&error.tags)?;
                
                conn.execute(
                    "INSERT INTO errors (
                        id, message, help, category, created_at, updated_at,
                        occurrence_count, last_message, tags, source
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        error.id,
                        error.message,
                        error.help,
                        error.category.to_string(),
                        error.created_at.to_rfc3339(),
                        error.updated_at.to_rfc3339(),
                        error.occurrence_count,
                        error.last_message,
                        tags_json,
                        error.source
                    ],
                )?;
                
                Ok(error.id.clone())
            }
        }
    }

    /// Holt einen Fehler anhand seiner ID
    pub fn get_error(&self, error_id: &str) -> ErrorManagementResult<Option<ErrorRecord>> {
        let conn = self.connection.read();
        
        let result: Option<ErrorRecord> = conn.query_row(
            "SELECT id, message, help, category, created_at, updated_at,
                    occurrence_count, last_message, tags, source
             FROM errors WHERE id = ?1",
            params![error_id],
            |row| {
                let category_str: String = row.get(3)?;
                let category = match category_str.as_str() {
                    "Network" => ErrorCategory::Network,
                    "Configuration" => ErrorCategory::Configuration,
                    "I/O" => ErrorCategory::Io,
                    "Cryptography" => ErrorCategory::Crypto,
                    "Storage" => ErrorCategory::Storage,
                    "Permission" => ErrorCategory::Permission,
                    "AI/ML" => ErrorCategory::Ai,
                    "Browser" => ErrorCategory::Browser,
                    "System" => ErrorCategory::System,
                    "Application" => ErrorCategory::Application,
                    _ => ErrorCategory::Unknown,
                };
                
                let tags_json: String = row.get(8)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                
                let created_at_str: String = row.get(4)?;
                let updated_at_str: String = row.get(5)?;
                
                Ok(ErrorRecord {
                    id: row.get(0)?,
                    message: row.get(1)?,
                    help: row.get(2)?,
                    category,
                    created_at: DateTime::parse_from_rfc3339(&created_at_str)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                        .with_timezone(&Utc),
                    occurrence_count: row.get(6)?,
                    last_message: row.get(7)?,
                    tags,
                    source: row.get(9)?,
                })
            },
        ).optional()?;
        
        Ok(result)
    }

    /// Sucht Fehler nach verschiedenen Kriterien
    pub fn search_errors(
        &self,
        query: Option<&str>,
        category: Option<ErrorCategory>,
        limit: Option<u32>,
    ) -> ErrorManagementResult<Vec<ErrorRecord>> {
        let conn = self.connection.read();
        let mut sql = String::from(
            "SELECT id, message, help, category, created_at, updated_at,
                    occurrence_count, last_message, tags, source
             FROM errors WHERE 1=1"
        );
        
        let mut params_vec = Vec::new();
        
        if let Some(q) = query {
            sql.push_str(" AND (message LIKE ? OR last_message LIKE ?)");
            let search_term = format!("%{}%", q);
            params_vec.push(search_term.clone());
            params_vec.push(search_term);
        }
        
        if let Some(cat) = category {
            sql.push_str(" AND category = ?");
            params_vec.push(cat.to_string());
        }
        
        sql.push_str(" ORDER BY occurrence_count DESC, updated_at DESC");
        
        if let Some(l) = limit {
            sql.push_str(&format!(" LIMIT {}", l));
        }
        
        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p as &dyn rusqlite::ToSql).collect();
        
        let error_iter = stmt.query_map(params.as_slice(), |row| {
            let category_str: String = row.get(3)?;
            let category = match category_str.as_str() {
                "Network" => ErrorCategory::Network,
                "Configuration" => ErrorCategory::Configuration,
                "I/O" => ErrorCategory::Io,
                "Cryptography" => ErrorCategory::Crypto,
                "Storage" => ErrorCategory::Storage,
                "Permission" => ErrorCategory::Permission,
                "AI/ML" => ErrorCategory::Ai,
                "Browser" => ErrorCategory::Browser,
                "System" => ErrorCategory::System,
                "Application" => ErrorCategory::Application,
                _ => ErrorCategory::Unknown,
            };
            
            let tags_json: String = row.get(8)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            
            let created_at_str: String = row.get(4)?;
            let updated_at_str: String = row.get(5)?;
            
            Ok(ErrorRecord {
                id: row.get(0)?,
                message: row.get(1)?,
                help: row.get(2)?,
                category,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                occurrence_count: row.get(6)?,
                last_message: row.get(7)?,
                tags,
                source: row.get(9)?,
            })
        })?;
        
        let mut errors = Vec::new();
        for error_result in error_iter {
            errors.push(error_result?);
        }
        
        Ok(errors)
    }

    /// Fügt einen Hilfe-Vorschlag hinzu
    pub fn add_help_suggestion(&self, suggestion: &HelpSuggestion) -> ErrorManagementResult<()> {
        let conn = self.connection.write();
        
        conn.execute(
            "INSERT INTO help_suggestions (
                error_id, suggestion, confidence, generated_at, model_name, accepted
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                suggestion.error_id,
                suggestion.suggestion,
                suggestion.confidence,
                suggestion.generated_at.to_rfc3339(),
                suggestion.model_name,
                suggestion.accepted,
            ],
        )?;
        
        Ok(())
    }

    /// Holt Hilfe-Vorschläge für einen Fehler
    pub fn get_help_suggestions(&self, error_id: &str) -> ErrorManagementResult<Vec<HelpSuggestion>> {
        let conn = self.connection.read();
        
        let mut stmt = conn.prepare(
            "SELECT error_id, suggestion, confidence, generated_at, model_name, accepted
             FROM help_suggestions WHERE error_id = ?1 ORDER BY confidence DESC, generated_at DESC"
        )?;
        
        let suggestion_iter = stmt.query_map(params![error_id], |row| {
            let generated_at_str: String = row.get(3)?;
            
            Ok(HelpSuggestion {
                error_id: row.get(0)?,
                suggestion: row.get(1)?,
                confidence: row.get(2)?,
                generated_at: DateTime::parse_from_rfc3339(&generated_at_str)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
                    .with_timezone(&Utc),
                model_name: row.get(4)?,
                accepted: row.get(5)?,
            })
        })?;
        
        let mut suggestions = Vec::new();
        for suggestion_result in suggestion_iter {
            suggestions.push(suggestion_result?);
        }
        
        Ok(suggestions)
    }

    /// Aktualisiert die Hilfe für einen Fehler
    pub fn update_error_help(&self, error_id: &str, help_text: &str) -> ErrorManagementResult<()> {
        let conn = self.connection.write();
        
        let updated = conn.execute(
            "UPDATE errors SET help = ?1, updated_at = ?2 WHERE id = ?3",
            params![help_text, Utc::now().to_rfc3339(), error_id],
        )?;
        
        if updated == 0 {
            return Err(ErrorManagementError::NotFound(format!("Error with id {} not found", error_id)));
        }
        
        Ok(())
    }

    /// Holt Statistiken über alle Fehler
    pub fn get_statistics(&self) -> ErrorManagementResult<ErrorStatistics> {
        let conn = self.connection.read();
        
        // Gesamtanzahl
        let total_errors: u64 = conn.query_row(
            "SELECT COUNT(*) FROM errors",
            [],
            |row| row.get(0),
        )?;
        
        // Fehler mit Hilfe
        let errors_with_help: u64 = conn.query_row(
            "SELECT COUNT(*) FROM errors WHERE help IS NOT NULL AND help != ''",
            [],
            |row| row.get(0),
        )?;
        
        // Fehler nach Kategorie
        let mut category_stmt = conn.prepare(
            "SELECT category, COUNT(*) FROM errors GROUP BY category ORDER BY COUNT(*) DESC"
        )?;
        
        let category_iter = category_stmt.query_map([], |row| {
            let category_str: String = row.get(0)?;
            let count: u64 = row.get(1)?;
            let category = match category_str.as_str() {
                "Network" => ErrorCategory::Network,
                "Configuration" => ErrorCategory::Configuration,
                "I/O" => ErrorCategory::Io,
                "Cryptography" => ErrorCategory::Crypto,
                "Storage" => ErrorCategory::Storage,
                "Permission" => ErrorCategory::Permission,
                "AI/ML" => ErrorCategory::Ai,
                "Browser" => ErrorCategory::Browser,
                "System" => ErrorCategory::System,
                "Application" => ErrorCategory::Application,
                _ => ErrorCategory::Unknown,
            };
            Ok((category, count))
        })?;
        
        let mut errors_by_category = Vec::new();
        for category_result in category_iter {
            errors_by_category.push(category_result?);
        }
        
        // Häufigste Fehler
        let mut frequent_stmt = conn.prepare(
            "SELECT message, occurrence_count FROM errors ORDER BY occurrence_count DESC LIMIT 10"
        )?;
        
        let frequent_iter = frequent_stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        
        let mut most_frequent_errors = Vec::new();
        for frequent_result in frequent_iter {
            most_frequent_errors.push(frequent_result?);
        }
        
        // Zeitliche Verteilung (letzte 30 Tage)
        let mut distribution_stmt = conn.prepare(
            "SELECT DATE(created_at) as day, COUNT(*) 
             FROM errors 
             WHERE created_at >= datetime('now', '-30 days')
             GROUP BY DATE(created_at)
             ORDER BY day DESC"
        )?;
        
        let distribution_iter = distribution_stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        
        let mut daily_distribution = Vec::new();
        for distribution_result in distribution_iter {
            daily_distribution.push(distribution_result?);
        }
        
        Ok(ErrorStatistics {
            total_errors,
            errors_with_help,
            errors_without_help: total_errors - errors_with_help,
            errors_by_category,
            most_frequent_errors,
            daily_distribution,
        })
    }

    /// Löscht einen Fehler (nur für Testzwecke)
    pub fn delete_error(&self, error_id: &str) -> ErrorManagementResult<()> {
        let conn = self.connection.write();
        
        let deleted = conn.execute(
            "DELETE FROM errors WHERE id = ?1",
            params![error_id],
        )?;
        
        if deleted == 0 {
            return Err(ErrorManagementError::NotFound(format!("Error with id {} not found", error_id)));
        }
        
        Ok(())
    }
}
