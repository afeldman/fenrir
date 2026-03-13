use fenrir_error_management::{ErrorManager, ErrorCategory};
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Temporäre Datenbank für Tests erstellen
    let temp_file = NamedTempFile::new()?;
    let db_path = temp_file.path().to_str().unwrap();
    
    println!("Initialisiere ErrorManager mit Datenbank: {}", db_path);
    
    // ErrorManager initialisieren
    let manager = ErrorManager::new(db_path).await?;
    
    // Beispiel-Fehler registrieren
    println!("\n--- Registriere Beispiel-Fehler ---");
    
    let error1_id = manager.register_error(
        "Network request timed out after 30 seconds",
        ErrorCategory::Network,
        Some("fenrir-network"),
        vec!["timeout", "http", "connection"],
    ).await?;
    
    println!("Fehler 1 registriert: {}", error1_id);
    
    let error2_id = manager.register_error(
        "Invalid configuration file: missing required field 'api_key'",
        ErrorCategory::Configuration,
        Some("fenrir-config"),
        vec!["config", "validation", "startup"],
    ).await?;
    
    println!("Fehler 2 registriert: {}", error2_id);
    
    let error3_id = manager.register_error(
        "Permission denied: cannot write to cache directory",
        ErrorCategory::Permission,
        Some("fenrir-core"),
        vec!["filesystem", "cache", "permissions"],
    ).await?;
    
    println!("Fehler 3 registriert: {}", error3_id);
    
    // Hilfe für Fehler abrufen
    println!("\n--- Hole Hilfe für Fehler ---");
    
    let help1 = manager.get_help(&error1_id).await?;
    println!("Hilfe für Fehler 1 (Network Timeout):\n{}\n", help1);
    
    let help2 = manager.get_help(&error2_id).await?;
    println!("Hilfe für Fehler 2 (Configuration):\n{}\n", help2);
    
    let help3 = manager.get_help(&error3_id).await?;
    println!("Hilfe für Fehler 3 (Permission):\n{}\n", help3);
    
    // Fehler suchen
    println!("--- Suche Fehler nach Kategorie ---");
    
    let network_errors = manager.search_errors(
        None,
        Some(ErrorCategory::Network),
        Some(5),
    ).await?;
    
    println!("Gefundene Network-Fehler: {}", network_errors.len());
    for error in &network_errors {
        println!("  - {}: {} (Vorkommen: {})", 
            error.id, error.message, error.occurrence_count);
    }
    
    // Statistiken abrufen
    println!("\n--- Hole Statistiken ---");
    
    let stats = manager.get_statistics().await?;
    println!("Gesamtanzahl Fehler: {}", stats.total_errors);
    println!("Fehler mit Hilfe: {}", stats.errors_with_help);
    println!("Fehler ohne Hilfe: {}", stats.errors_without_help);
    
    println!("\nFehler nach Kategorie:");
    for (category, count) in &stats.errors_by_category {
        println!("  - {}: {}", category, count);
    }
    
    println!("\nHäufigste Fehler:");
    for (message, count) in &stats.most_frequent_errors {
        println!("  - {} ({}x)", message, count);
    }
    
    // Manuelle Hilfe hinzufügen
    println!("\n--- Füge manuelle Hilfe hinzu ---");
    
    let custom_error_id = manager.register_error(
        "Custom application error: data validation failed",
        ErrorCategory::Application,
        Some("my-app"),
        vec!["validation", "data"],
    ).await?;
    
    manager.add_manual_help(
        &custom_error_id,
        "Dies ist ein benutzerdefinierter Validierungsfehler.\n\
        - Prüfen Sie die Eingabedaten auf Korrektheit\n\
        - Stellen Sie sicher, dass alle erforderlichen Felder ausgefüllt sind\n\
        - Bei fortbestehendem Problem kontaktieren Sie den Support",
    ).await?;
    
    let custom_help = manager.get_help(&custom_error_id).await?;
    println!("Manuelle Hilfe für benutzerdefinierten Fehler:\n{}", custom_help);
    
    // Test: Unbekannter Fehler (sollte Fallback-Hilfe generieren)
    println!("\n--- Test: Unbekannter Fehlertyp ---");
    
    let unknown_error_id = manager.register_error(
        "Very specific and unusual error condition XYZ-123",
        ErrorCategory::Unknown,
        None,
        vec!["unusual", "specific"],
    ).await?;
    
    let unknown_help = manager.get_help(&unknown_error_id).await?;
    println!("Hilfe für unbekannten Fehler (Fallback):\n{}", unknown_help);
    
    println!("\n--- Test abgeschlossen ---");
    
    Ok(())
}
