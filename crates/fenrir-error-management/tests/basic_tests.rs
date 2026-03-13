use fenrir_error_management::{ErrorManager, ErrorCategory};

#[tokio::test]
async fn test_error_registration() -> Result<(), Box<dyn std::error::Error>> {
    // In-Memory Datenbank für Tests
    let manager = ErrorManager::new(":memory:").await?;
    
    // Fehler registrieren
    let error_id = manager.register_error(
        "Test error message",
        ErrorCategory::Application,
        Some("test-module"),
        vec!["test", "unit"],
    ).await?;
    
    assert!(!error_id.is_empty());
    
    // Hilfe abrufen (sollte Regel-basierte oder Fallback-Hilfe geben)
    let help = manager.get_help(&error_id).await?;
    assert!(!help.is_empty());
    assert!(help.contains("Test error message") || help.contains("Keine"));
    
    Ok(())
}

#[tokio::test]
async fn test_error_search() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ErrorManager::new(":memory:").await?;
    
    // Mehrere Fehler registrieren
    manager.register_error(
        "Network timeout error",
        ErrorCategory::Network,
        None,
        vec![],
    ).await?;
    
    manager.register_error(
        "Configuration error",
        ErrorCategory::Configuration,
        None,
        vec![],
    ).await?;
    
    manager.register_error(
        "Another network error",
        ErrorCategory::Network,
        None,
        vec![],
    ).await?;
    
    // Nach Network-Fehlern suchen
    let network_errors = manager.search_errors(
        None,
        Some(ErrorCategory::Network),
        Some(10),
    ).await?;
    
    assert_eq!(network_errors.len(), 2);
    
    // Nach Text suchen
    let config_errors = manager.search_errors(
        Some("Configuration"),
        None,
        Some(10),
    ).await?;
    
    assert_eq!(config_errors.len(), 1);
    
    Ok(())
}

#[tokio::test]
async fn test_statistics() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ErrorManager::new(":memory:").await?;
    
    // Einige Fehler registrieren
    for i in 0..5 {
        manager.register_error(
            &format!("Test error {}", i),
            ErrorCategory::Application,
            None,
            vec![],
        ).await?;
    }
    
    // Zusätzliche Network-Fehler
    for i in 0..3 {
        manager.register_error(
            &format!("Network error {}", i),
            ErrorCategory::Network,
            None,
            vec![],
        ).await?;
    }
    
    // Statistiken abrufen
    let stats = manager.get_statistics().await?;
    
    assert_eq!(stats.total_errors, 8);
    
    // Überprüfen, dass Kategorien gezählt wurden
    let network_count = stats.errors_by_category
        .iter()
        .find(|(cat, _)| *cat == ErrorCategory::Network)
        .map(|(_, count)| *count)
        .unwrap_or(0);
    
    assert_eq!(network_count, 3);
    
    Ok(())
}

#[tokio::test]
async fn test_manual_help() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ErrorManager::new(":memory:").await?;
    
    // Fehler registrieren
    let error_id = manager.register_error(
        "Test error for manual help",
        ErrorCategory::Application,
        None,
        vec![],
    ).await?;
    
    // Manuelle Hilfe hinzufügen
    let manual_help = "This is a manual help text for testing.";
    manager.add_manual_help(&error_id, manual_help).await?;
    
    // Hilfe abrufen und überprüfen
    let help = manager.get_help(&error_id).await?;
    assert!(help.contains(manual_help));
    
    Ok(())
}
