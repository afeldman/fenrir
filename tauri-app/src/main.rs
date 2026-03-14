//! Fenrir Browser - Tauri Hauptanwendung
//!
//! Diese App integriert alle Fenrir-Crates in eine Tauri-Anwendung:
//! - fenrir-secure: Sicherheit und Berechtigungen
//! - fenrir-sandbox: Servo-Instanzen in Tauri-Fenstern
//! - fenrir-api: Kommunikation zwischen Servo und Hauptprozess
//! - fenrir-ui: Native Benutzeroberfläche

mod app_state;
mod commands;
mod events;
mod menu;
mod tray;

use crate::app_state::AppState;
use crate::menu::create_menu;
use crate::tray::create_tray;
use fenrir_api::ApiManager;
use fenrir_config::load;
use fenrir_i18n::{set_language, Language};
use fenrir_log::init_logging;
use fenrir_sandbox::{SandboxManager, InstanceEvent};
use fenrir_secure::SecurityManager;
use fenrir_ui::UiManager;
use std::sync::Arc;
use tauri::{App, AppHandle, Manager, RunEvent};
use tracing::{info, error, warn};

/// Initialisiere die Internationalisierung (i18n)
fn init_i18n() -> anyhow::Result<()> {
    // Konfiguration laden, um Spracheinstellung zu erhalten
    let config = load().unwrap_or_default();
    
    // Sprache aus Konfiguration parsen
    let language = Language::from_str(&config.ui.language)
        .unwrap_or_else(|_| {
            warn!("Ungültige Sprache '{}' in Konfiguration, verwende Standard", config.ui.language);
            Language::default()
        });
    
    // i18n mit konfigurierter Sprache initialisieren
    set_language(language)
        .map_err(|e| anyhow::anyhow!("Fehler bei i18n-Initialisierung: {}", e))?;
    
    info!("i18n initialisiert mit Sprache: {}", language);
    Ok(())
}

/// Hauptfunktion - Einstiegspunkt der Tauri-App
#[tauri::command]
async fn main() {
    // Logging initialisieren
    let _log_guard = init_logging().expect("Logging initialisieren");
    
    info!("Starte Fenrir Browser mit Tauri-Integration...");
    
    // TLS Crypto-Provider (von Servo/rustls benötigt)
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("rustls crypto provider");
    
    // Internationalisierung initialisieren
    if let Err(e) = init_i18n() {
        error!("Fehler bei i18n-Initialisierung: {}", e);
    }
    
    // Tauri-Anwendung erstellen und konfigurieren
    tauri::Builder::default()
        .setup(|app| {
            info!("Tauri-App wird eingerichtet...");
            
            // App-State initialisieren
            let app_state = AppState::new(app.handle());
            app.manage(app_state);
            
            // Menü erstellen
            create_menu(app)?;
            
            // System-Tray erstellen
            create_tray(app)?;
            
            info!("Tauri-App erfolgreich eingerichtet");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::navigate,
            commands::reload,
            commands::go_back,
            commands::go_forward,
            commands::stop_loading,
            commands::create_tab,
            commands::close_tab,
            commands::switch_tab,
            commands::mount_folder,
            commands::show_settings,
            commands::show_permissions,
        ])
        .on_window_event(|window, event| {
            // Fenster-Ereignisse behandeln
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    info!("Fenster-Schließen angefordert: {}", window.label());
                    // Verhindere Standardverhalten, um eigene Logik auszuführen
                    api.prevent_close();
                    // TODO: Eigene Schließen-Logik implementieren
                }
                _ => {}
            }
        })
        .build(tauri::generate_context!())
        .expect("Fehler beim Bauen der Tauri-App")
        .run(|app_handle, event| {
            match event {
                RunEvent::Ready => {
                    info!("Tauri-App ist bereit");
                    
                    // App-State aus dem Handle holen
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        // Haupt-UI initialisieren
                        if let Err(e) = state.initialize_ui() {
                            error!("Fehler bei UI-Initialisierung: {}", e);
                        }
                    }
                }
                RunEvent::ExitRequested { api, .. } => {
                    info!("App-Beenden angefordert");
                    // Verhindere Standardverhalten für sauberes Herunterfahren
                    api.prevent_exit();
                    
                    // Sauberes Herunterfahren aller Komponenten
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        state.cleanup();
                    }
                    
                    // Jetzt wirklich beenden
                    std::process::exit(0);
                }
                RunEvent::MainEventsCleared => {
                    // Servo-Events verarbeiten (Haupt-Event-Loop)
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        state.process_servo_events();
                    }
                }
                _ => {}
            }
        });
}

// Hilfsfunktion für Language-String-Parsing
trait LanguageExt {
    fn from_str(s: &str) -> Result<Language, &'static str>;
}

impl LanguageExt for Language {
    fn from_str(s: &str) -> Result<Language, &'static str> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::English),
            "de" | "german" => Ok(Language::German),
            "fr" | "french" => Ok(Language::French),
            "es" | "spanish" => Ok(Language::Spanish),
            _ => Err("Unbekannte Sprache"),
        }
    }
}
