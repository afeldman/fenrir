//! Event-Handling für Servo- und UI-Events

use fenrir_sandbox::InstanceEvent;
use fenrir_ui::UiManager;
use tracing::{info, warn};

/// Behandle Instance-Events von Servo
pub fn handle_instance_event(
    ui_manager: &UiManager,
    instance_id: &str,
    event: InstanceEvent,
) {
    match event {
        InstanceEvent::TitleChanged(title) => {
            info!("UI: Title changed for {}: {:?}", instance_id, title);
            // UI aktualisieren (Tab-Titel)
            // In echtem Code: UI-State aktualisieren
        }
        InstanceEvent::UrlChanged(url) => {
            info!("UI: URL changed for {}: {}", instance_id, url);
            // URL-Bar aktualisieren
            // In echtem Code: UI-State aktualisieren
        }
        InstanceEvent::LoadStatusChanged(status) => {
            info!("UI: Load status changed for {}: {:?}", instance_id, status);
            // Lade-Indikator aktualisieren
            // In echtem Code: UI-State aktualisieren
        }
        InstanceEvent::PermissionRequest(request) => {
            info!("UI: Permission request for {}: {:?}", instance_id, request);
            // Permission-Dialog zeigen
            // In echtem Code: UI-Dialog öffnen
        }
        InstanceEvent::NavigationRequest(request) => {
            info!("UI: Navigation request for {}: {:?}", instance_id, request);
            // Navigation behandeln (z.B. Popup-Blocker)
            // In echtem Code: Navigation prüfen und ggf. blockieren
        }
        InstanceEvent::Custom(name, data) => {
            info!("UI: Custom event for {}: {} = {:?}", instance_id, name, data);
            // Benutzerdefinierte Events behandeln
        }
    }
}

/// Sende Event an Frontend (WebView)
pub fn emit_to_frontend(
    app_handle: &tauri::AppHandle,
    event_name: &str,
    payload: Option<serde_json::Value>,
) -> Result<(), tauri::Error> {
    // An alle Fenster senden
    app_handle.emit_all(event_name, payload.unwrap_or(serde_json::Value::Null))
}

/// Spezifische Events für Frontend
pub mod frontend_events {
    pub const TAB_CREATED: &str = "tab-created";
    pub const TAB_CLOSED: &str = "tab-closed";
    pub const TAB_UPDATED: &str = "tab-updated";
    pub const URL_CHANGED: &str = "url-changed";
    pub const TITLE_CHANGED: &str = "title-changed";
    pub const LOADING_CHANGED: &str = "loading-changed";
    pub const PERMISSION_REQUESTED: &str = "permission-requested";
}

/// Event-Payload-Strukturen für Frontend
#[derive(serde::Serialize)]
pub struct TabEventPayload {
    pub tab_id: String,
    pub title: Option<String>,
    pub url: Option<String>,
    pub loading: Option<bool>,
    pub progress: Option<f32>,
}

#[derive(serde::Serialize)]
pub struct PermissionEventPayload {
    pub tab_id: String,
    pub permission_type: String,
    pub details: serde_json::Value,
}

/// Helper-Funktionen für spezifische Events
pub fn emit_tab_created(
    app_handle: &tauri::AppHandle,
    tab_id: String,
    url: String,
) -> Result<(), tauri::Error> {
    let payload = TabEventPayload {
        tab_id,
        title: None,
        url: Some(url),
        loading: Some(true),
        progress: Some(0.0),
    };
    
    emit_to_frontend(app_handle, frontend_events::TAB_CREATED, Some(serde_json::to_value(payload)?))
}

pub fn emit_tab_closed(
    app_handle: &tauri::AppHandle,
    tab_id: String,
) -> Result<(), tauri::Error> {
    let payload = TabEventPayload {
        tab_id,
        title: None,
        url: None,
        loading: None,
        progress: None,
    };
    
    emit_to_frontend(app_handle, frontend_events::TAB_CLOSED, Some(serde_json::to_value(payload)?))
}

pub fn emit_url_changed(
    app_handle: &tauri::AppHandle,
    tab_id: String,
    url: String,
) -> Result<(), tauri::Error> {
    let payload = TabEventPayload {
        tab_id,
        title: None,
        url: Some(url),
        loading: None,
        progress: None,
    };
    
    emit_to_frontend(app_handle, frontend_events::URL_CHANGED, Some(serde_json::to_value(payload)?))
}

pub fn emit_title_changed(
    app_handle: &tauri::AppHandle,
    tab_id: String,
    title: String,
) -> Result<(), tauri::Error> {
    let payload = TabEventPayload {
        tab_id,
        title: Some(title),
        url: None,
        loading: None,
        progress: None,
    };
    
    emit_to_frontend(app_handle, frontend_events::TITLE_CHANGED, Some(serde_json::to_value(payload)?))
}

pub fn emit_loading_changed(
    app_handle: &tauri::AppHandle,
    tab_id: String,
    loading: bool,
    progress: f32,
) -> Result<(), tauri::Error> {
    let payload = TabEventPayload {
        tab_id,
        title: None,
        url: None,
        loading: Some(loading),
        progress: Some(progress),
    };
    
    emit_to_frontend(app_handle, frontend_events::LOADING_CHANGED, Some(serde_json::to_value(payload)?))
}

pub fn emit_permission_requested(
    app_handle: &tauri::AppHandle,
    tab_id: String,
    permission_type: String,
    details: serde_json::Value,
) -> Result<(), tauri::Error> {
    let payload = PermissionEventPayload {
        tab_id,
        permission_type,
        details,
    };
    
    emit_to_frontend(app_handle, frontend_events::PERMISSION_REQUESTED, Some(serde_json::to_value(payload)?))
}
