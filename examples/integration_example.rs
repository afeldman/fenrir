//! Beispiel: "Benutzer gibt URL ein und klickt auf 'Ordner mounten'"
//!
//! Dieses Beispiel zeigt, wie alle vier Crates zusammenarbeiten:
//! 1. fenrir-secure: Origin-Policies und Pfadberechtigungen
//! 2. fenrir-sandbox: Servo-Instanz in Tauri-Fenster
//! 3. fenrir-api: Kommunikation zwischen Servo und Hauptprozess
//! 4. fenrir-ui: Native Benutzeroberfläche

use fenrir_api::{ApiManager, ApiCommand, CommandRequest};
use fenrir_sandbox::{SandboxManager, WindowConfig};
use fenrir_secure::{SecurityManager, Origin};
use fenrir_ui::{UiManager, NavigationAction};
use std::sync::Arc;
use tauri::App;
use tokio::runtime::Runtime;
use url::Url;

/// Hauptfunktion, die die Integration aller Crates demonstriert
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tokio Runtime für asynchrone Operationen
    let rt = Runtime::new()?;
    
    rt.block_on(async {
        // 1. Security Manager erstellen (Kern-Sicherheit)
        let security_manager = Arc::new(SecurityManager::new());
        
        // 2. Sandbox Manager erstellen (Servo-Instanzen in Tauri-Fenstern)
        let sandbox_manager = Arc::new(SandboxManager::new(Arc::clone(&security_manager)));
        
        // 3. API Manager erstellen (Kommunikation zwischen Servo und Hauptprozess)
        let api_manager = Arc::new(ApiManager::new(Arc::clone(&security_manager)));
        
        // 4. Tauri App erstellen (würde normalerweise von Tauri gemacht)
        println!("Initialisiere Fenrir Browser mit Servo-Integration...");
        
        // Beispiel: Benutzer gibt URL "https://example.com" ein
        let url = Url::parse("https://example.com").unwrap();
        println!("Benutzer gibt URL ein: {}", url);
        
        // Simuliere UI-Ereignis: URL wurde eingegeben
        simulate_url_entered(
            Arc::clone(&security_manager),
            Arc::clone(&sandbox_manager),
            Arc::clone(&api_manager),
            url,
        ).await?;
        
        // Beispiel: Benutzer klickt auf "Ordner mounten" für diese Seite
        let folder_path = "/Users/benutzer/Dokumente".to_string();
        println!("Benutzer möchte Ordner mounten: {}", folder_path);
        
        simulate_mount_folder(
            Arc::clone(&security_manager),
            folder_path,
            "https://example.com".to_string(),
        ).await?;
        
        // Beispiel: Servo möchte eine Datei lesen (über API)
        println!("Servo möchte eine Datei lesen...");
        
        simulate_file_read(
            Arc::clone(&api_manager),
            Arc::clone(&security_manager),
            "/Users/benutzer/Dokumente/test.txt".to_string(),
            "https://example.com".to_string(),
        ).await?;
        
        println!("Beispiel erfolgreich abgeschlossen!");
        
        Ok(())
    })
}

/// Simuliert die Eingabe einer URL durch den Benutzer
async fn simulate_url_entered(
    security_manager: Arc<SecurityManager>,
    sandbox_manager: Arc<SandboxManager>,
    api_manager: Arc<ApiManager>,
    url: Url,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Schritt 1: URL-Eingabe ===");
    
    // Extrahiere Origin aus URL
    let origin = Origin::from_url(&url).unwrap();
    println!("  Origin: {}", origin);
    
    // Erstelle eine Servo-Instanz in einem sandboxed Tauri-Fenster
    // In einer echten App würde dies über den UiManager gehen
    println!("  Erstelle Servo-Instanz für {}...", url);
    
    // Window-Konfiguration für diese URL
    let window_config = WindowConfig::for_url(&url);
    
    // In einer echten App:
    // let instance_id = sandbox_manager.create_instance(&app_handle, url.clone(), window_config).await?;
    // println!("  Servo-Instanz erstellt: {}", instance_id);
    
    // Registriere die Instanz beim Security Manager
    // security_manager.register_servo_instance(instance_id.clone(), origin, servo_instance);
    
    println!("  ✓ Servo-Instanz würde jetzt in sandboxed Fenster gestartet");
    
    Ok(())
}

/// Simuliert das Mounten eines Ordners für eine Origin
async fn simulate_mount_folder(
    security_manager: Arc<SecurityManager>,
    folder_path: String,
    url_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Schritt 2: Ordner mounten ===");
    
    // Parse URL zu Origin
    let url = Url::parse(&url_str).unwrap();
    let origin = Origin::from_url(&url).unwrap();
    
    println!("  Mounte Ordner '{}' für Origin {}", folder_path, origin);
    
    // Mounte das Verzeichnis für diese Origin
    // Dies gibt der Origin Berechtigung, auf diesen Pfad zuzugreifen
    let mut sec_mgr = SecurityManager::new(); // In echtem Code: &mut *security_manager
    sec_mgr.mount_directory(
        origin.clone(),
        std::path::PathBuf::from(&folder_path),
    )?;
    
    println!("  ✓ Ordner erfolgreich gemounted");
    println!("  Origin {} kann jetzt auf {} zugreifen", origin, folder_path);
    
    // Zeige die gemounteten Pfade für diese Origin
    // In echtem Code: security_manager.get_mounted_paths(&origin)
    
    Ok(())
}

/// Simuliert eine Dateilese-Anfrage von Servo über die API
async fn simulate_file_read(
    api_manager: Arc<ApiManager>,
    security_manager: Arc<SecurityManager>,
    file_path: String,
    origin_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Schritt 3: Dateilesen über API ===");
    
    // Servo sendet eine Nachricht über die Embedding-API
    // Diese wird von fenrir-api in ein CommandRequest umgewandelt
    
    let origin = Origin::from_url(&Url::parse(&origin_url).unwrap()).unwrap();
    println!("  Servo (Origin {}) möchte Datei lesen: {}", origin, file_path);
    
    // 1. Servo sendet Nachricht (simuliert)
    let servo_message = serde_json::json!({
        "type": "file_read",
        "path": file_path,
        "encoding": "UTF-8",
        "instance_id": "servo-instance-123"
    });
    
    println!("  Servo sendet Nachricht: {:?}", servo_message);
    
    // 2. API Manager empfängt und verarbeitet die Nachricht
    // In echtem Code: api_manager.handle_servo_message("servo-instance-123", servo_message).await
    
    // 3. Security Manager prüft Berechtigungen
    let path_buf = std::path::PathBuf::from("/Users/benutzer/Dokumente/test.txt");
    let sec_mgr = SecurityManager::new(); // In echtem Code: &*security_manager
    
    match sec_mgr.check_path_permission(&origin, &path_buf) {
        Ok(allowed_path) => {
            println!("  ✓ Berechtigung erteilt für Pfad: {:?}", allowed_path);
            
            // 4. Command wird ausgeführt (Datei wird gelesen)
            println!("  Datei würde jetzt gelesen werden...");
            
            // In echtem Code würde hier die Datei tatsächlich gelesen werden
            // und das Ergebnis zurück an Servo gesendet werden
            
            println!("  ✓ Datei erfolgreich gelesen (simuliert)");
        }
        Err(e) => {
            println!("  ✗ Berechtigung verweigert: {}", e);
            println!("  Servo erhält Fehler zurück");
        }
    }
    
    Ok(())
}

/// Zeigt die Architektur und Abhängigkeiten
fn show_architecture() {
    println!("\n=== Architektur-Übersicht ===");
    println!();
    println!("fenrir-secure (Core-Sicherheit)");
    println!("  ├── Origin-Policy-System (kompatibel mit Servo)");
    println!("  ├── Pfadkanonisierung und -validierung");
    println!("  ├── Berechtigungsregister für Servo-Instanzen");
    println!("  └── Integration mit Servo's Permission-Delegation");
    println!();
    println!("fenrir-sandbox (Prozess-Isolation)");
    println!("  ├── Servo-Instanz-Management in Tauri-Fenstern");
    println!("  ├── WindowBuilder-Integration für Sandboxing");
    println!("  ├── Servo-Embedding-API-Nutzung");
    println!("  └── Prozess-Lebenszyklus-Management");
    println!();
    println!("fenrir-api (Kommunikation)");
    println!("  ├── IPC-Command-Definitionen");
    println!("  ├── Servo-Embedding-Nachrichten-Adapter");
    println!("  ├── Berechtigungsprüfung mit fenrir-secure");
    println!("  └── Asynchrone API-Implementierung");
    println!();
    println!("fenrir-ui (Benutzeroberfläche)");
    println!("  ├── Native Tauri-UI-Komponenten (kein WebView!)");
    println!("  ├── URL-Leiste und Tab-Management");
    println!("  ├── Integration mit fenrir-sandbox");
    println!("  └── Servo-Ereignis-Handling");
    println!();
    println!("=== Servo-Integration ===");
    println!("• Nutzt Servo's Embedding-API (components/servo/lib.rs)");
    println!("• Respektiert Servo's Multiprozess-Architektur");
    println!("• Integriert mit Servo's Permission-Delegation");
    println!("• Keine Modifikation von Servo nötig");
}

// Führe das Beispiel aus
fn main_with_arch() -> Result<(), Box<dyn std::error::Error>> {
    show_architecture();
    println!("\n=== Laufendes Beispiel ===");
    main()
}

// Für direkte Ausführung
pub fn run_example() {
    if let Err(e) = main_with_arch() {
        eprintln!("Fehler: {}", e);
        std::process::exit(1);
    }
}
