//! Tauri Commands (IPC-Endpunkte) für Fenrir Browser

use crate::app_state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tracing::{info, error};
use url::Url;

/// Navigiere zu einer URL im aktiven Tab
#[tauri::command]
pub async fn navigate(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    url: String,
) -> Result<(), String> {
    info!("Navigate command: {}", url);
    
    // Parse URL
    let parsed_url = Url::parse(&url)
        .map_err(|e| format!("Ungültige URL: {}", e))?;
    
    // Aktiven Tab finden (in echtem Code: aus UI-State)
    // Für jetzt: erste Instanz nehmen oder neue erstellen
    let instances = state.sandbox_manager().get_all_instances();
    
    if let Some(instance) = instances.first() {
        // Bestehende Instanz navigieren
        state.navigate_servo_instance(instance.id(), parsed_url)
            .map_err(|e| e.to_string())?;
    } else {
        // Neue Instanz erstellen
        state.create_servo_instance(parsed_url, None)
            .await
            .map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// Lade aktiven Tab neu
#[tauri::command]
pub async fn reload(
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Reload command");
    
    // Aktiven Tab finden
    let instances = state.sandbox_manager().get_all_instances();
    
    if let Some(instance) = instances.first() {
        state.reload_servo_instance(instance.id())
            .map_err(|e| e.to_string())?;
    } else {
        return Err("Kein aktiver Tab gefunden".to_string());
    }
    
    Ok(())
}

/// Gehe im aktiven Tab zurück
#[tauri::command]
pub async fn go_back(
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Go back command");
    
    let instances = state.sandbox_manager().get_all_instances();
    
    if let Some(instance) = instances.first() {
        state.go_back_servo_instance(instance.id())
            .map_err(|e| e.to_string())?;
    } else {
        return Err("Kein aktiver Tab gefunden".to_string());
    }
    
    Ok(())
}

/// Gehe im aktiven Tab vorwärts
#[tauri::command]
pub async fn go_forward(
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Go forward command");
    
    let instances = state.sandbox_manager().get_all_instances();
    
    if let Some(instance) = instances.first() {
        state.go_forward_servo_instance(instance.id())
            .map_err(|e| e.to_string())?;
    } else {
        return Err("Kein aktiver Tab gefunden".to_string());
    }
    
    Ok(())
}

/// Stoppe Laden im aktiven Tab
#[tauri::command]
pub async fn stop_loading(
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Stop loading command");
    
    let instances = state.sandbox_manager().get_all_instances();
    
    if let Some(instance) = instances.first() {
        state.stop_servo_instance(instance.id())
            .map_err(|e| e.to_string())?;
    } else {
        return Err("Kein aktiver Tab gefunden".to_string());
    }
    
    Ok(())
}

/// Erstelle neuen Tab
#[tauri::command]
pub async fn create_tab(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    url: Option<String>,
) -> Result<String, String> {
    let url_str = url.unwrap_or_else(|| "about:blank".to_string());
    info!("Create tab command: {}", url_str);
    
    let parsed_url = Url::parse(&url_str)
        .map_err(|e| format!("Ungültige URL: {}", e))?;
    
    // Event-Handler für Servo-Events
    let event_handler: Box<dyn Fn(fenrir_sandbox::InstanceEvent) + Send + Sync> = Box::new({
        let state = state.inner().clone();
        move |event| {
            // Event an App-State weiterleiten
            // In echtem Code: instance_id aus Closure erfassen
        }
    });
    
    let instance_id = state.create_servo_instance(parsed_url, Some(event_handler))
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(instance_id)
}

/// Schließe Tab
#[tauri::command]
pub async fn close_tab(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), String> {
    info!("Close tab command: {}", tab_id);
    
    state.close_servo_instance(&tab_id)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Wechsle zu Tab
#[tauri::command]
pub async fn switch_tab(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), String> {
    info!("Switch tab command: {}", tab_id);
    
    // In echtem Code: UI-State aktualisieren und Fenster fokussieren
    // Für jetzt: nur loggen
    
    Ok(())
}

/// Mounte Ordner für aktuelle Seite
#[tauri::command]
pub async fn mount_folder(
    state: State<'_, AppState>,
    folder_path: String,
    origin_url: Option<String>,
) -> Result<(), String> {
    info!("Mount folder command: {}", folder_path);
    
    // Origin-URL bestimmen (entweder angegeben oder aktuelle Seite)
    let origin_url = match origin_url {
        Some(url) => url,
        None => {
            // Aktuelle URL aus aktiven Tab holen
            let instances = state.sandbox_manager().get_all_instances();
            if let Some(instance) = instances.first() {
                instance.url().to_string()
            } else {
                return Err("Kein aktiver Tab für Origin".to_string());
            }
        }
    };
    
    state.mount_folder(origin_url, folder_path)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Zeige Einstellungen
#[tauri::command]
pub async fn show_settings(
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Show settings command");
    
    state.show_settings()
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Zeige Berechtigungs-Manager
#[tauri::command]
pub async fn show_permissions(
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Show permissions command");
    
    state.show_permissions()
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Datenstruktur für Tab-Informationen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    pub loading: bool,
    pub progress: f32,
}

/// Hole alle Tabs
#[tauri::command]
pub async fn get_tabs(
    state: State<'_, AppState>,
) -> Result<Vec<TabInfo>, String> {
    let instances = state.sandbox_manager().get_all_instances();
    
    let tabs: Vec<TabInfo> = instances.iter()
        .map(|instance| TabInfo {
            id: instance.id().to_string(),
            title: "".to_string(), // In echtem Code: aus UI-State
            url: instance.url().to_string(),
            loading: false, // In echtem Code: aus Servo-Event
            progress: 0.0,
        })
        .collect();
    
    Ok(tabs)
}

/// Hole aktiven Tab
#[tauri::command]
pub async fn get_active_tab(
    state: State<'_, AppState>,
) -> Result<Option<TabInfo>, String> {
    let instances = state.sandbox_manager().get_all_instances();
    
    let active_tab = instances.first()
        .map(|instance| TabInfo {
            id: instance.id().to_string(),
            title: "".to_string(),
            url: instance.url().to_string(),
            loading: false,
            progress: 0.0,
        });
    
    Ok(active_tab)
}
