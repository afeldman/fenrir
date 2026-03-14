//! App-State Management für Fenrir Tauri-App

use crate::events::handle_instance_event;
use fenrir_api::ApiManager;
use fenrir_sandbox::{SandboxManager, InstanceEvent};
use fenrir_secure::SecurityManager;
use fenrir_ui::UiManager;
use std::sync::Arc;
use tauri::AppHandle;
use tracing::{info, error};

/// Haupt-App-State, der alle Komponenten verwaltet
pub struct AppState {
    /// Tauri App-Handle
    app_handle: AppHandle,
    /// Security Manager (Berechtigungen und Origin-Policies)
    security_manager: Arc<SecurityManager>,
    /// Sandbox Manager (Servo-Instanzen in Tauri-Fenstern)
    sandbox_manager: Arc<SandboxManager>,
    /// API Manager (Kommunikation zwischen Servo und Hauptprozess)
    api_manager: Arc<ApiManager>,
    /// UI Manager (Native Benutzeroberfläche)
    ui_manager: Option<UiManager>,
    /// Ob die App initialisiert ist
    initialized: bool,
}

impl AppState {
    /// Erstelle neuen App-State
    pub fn new(app_handle: AppHandle) -> Self {
        info!("Erstelle App-State...");
        
        // Security Manager erstellen
        let security_manager = Arc::new(SecurityManager::new());
        
        // Sandbox Manager erstellen (mit Security Manager)
        let sandbox_manager = Arc::new(SandboxManager::new(Arc::clone(&security_manager)));
        
        // API Manager erstellen (mit Security Manager)
        let api_manager = Arc::new(ApiManager::new(Arc::clone(&security_manager)));
        
        Self {
            app_handle,
            security_manager,
            sandbox_manager,
            api_manager,
            ui_manager: None,
            initialized: false,
        }
    }
    
    /// Initialisiere die UI (muss nach App-Setup aufgerufen werden)
    pub fn initialize_ui(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }
        
        info!("Initialisiere UI...");
        
        // UI Manager erstellen
        let ui_manager = UiManager::new(
            self.app_handle.clone(),
            Arc::clone(&self.security_manager),
            Arc::clone(&self.sandbox_manager),
            Arc::clone(&self.api_manager),
        )?;
        
        // UI initialisieren
        ui_manager.initialize()?;
        
        self.ui_manager = Some(ui_manager);
        self.initialized = true;
        
        info!("UI erfolgreich initialisiert");
        
        Ok(())
    }
    
    /// Verarbeite Servo-Events (sollte regelmäßig aus der Haupt-Event-Loop aufgerufen werden)
    pub fn process_servo_events(&self) {
        if !self.initialized {
            return;
        }
        
        // Servo-Events verarbeiten
        self.sandbox_manager.process_events();
    }
    
    /// Behandle ein Instance-Event von einer Servo-Instanz
    pub fn handle_instance_event(&self, instance_id: String, event: InstanceEvent) {
        info!("Behandle Instance-Event für {}: {:?}", instance_id, event);
        
        // Event an Sandbox Manager weiterleiten
        self.sandbox_manager.handle_instance_event(&instance_id, event.clone());
        
        // Event an UI Manager weiterleiten (falls vorhanden)
        if let Some(ui_manager) = &self.ui_manager {
            // UI-spezifische Event-Behandlung
            handle_instance_event(ui_manager, &instance_id, event);
        }
    }
    
    /// Erstelle eine neue Servo-Instanz (Tab)
    pub async fn create_servo_instance(
        &self,
        url: url::Url,
        event_handler: Option<Box<dyn Fn(InstanceEvent) + Send + Sync>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        info!("Erstelle neue Servo-Instanz für {}", url);
        
        // Window-Konfiguration für diese URL
        let window_config = fenrir_sandbox::WindowConfig::for_url(&url);
        
        // Instanz erstellen
        let instance_id = self.sandbox_manager.create_instance(
            &self.app_handle,
            url,
            window_config,
            event_handler,
        ).await?;
        
        info!("Servo-Instanz erstellt: {}", instance_id);
        
        Ok(instance_id)
    }
    
    /// Navigiere eine Servo-Instanz zu einer neuen URL
    pub fn navigate_servo_instance(
        &self,
        instance_id: &str,
        url: url::Url,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Navigiere Instanz {} zu {}", instance_id, url);
        
        self.sandbox_manager.navigate_instance(instance_id, url)?;
        
        Ok(())
    }
    
    /// Lade eine Servo-Instanz neu
    pub fn reload_servo_instance(&self, instance_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Lade Instanz {} neu", instance_id);
        
        self.sandbox_manager.reload_instance(instance_id)?;
        
        Ok(())
    }
    
    /// Gehe in der History einer Servo-Instanz zurück
    pub fn go_back_servo_instance(&self, instance_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Gehe zurück in Instanz {}", instance_id);
        
        self.sandbox_manager.go_back_instance(instance_id)?;
        
        Ok(())
    }
    
    /// Gehe in der History einer Servo-Instanz vorwärts
    pub fn go_forward_servo_instance(&self, instance_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Gehe vorwärts in Instanz {}", instance_id);
        
        self.sandbox_manager.go_forward_instance(instance_id)?;
        
        Ok(())
    }
    
    /// Stoppe das Laden in einer Servo-Instanz
    pub fn stop_servo_instance(&self, instance_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Stoppe Laden in Instanz {}", instance_id);
        
        self.sandbox_manager.stop_instance(instance_id)?;
        
        Ok(())
    }
    
    /// Schließe eine Servo-Instanz (Tab)
    pub fn close_servo_instance(&self, instance_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Schließe Servo-Instanz {}", instance_id);
        
        self.sandbox_manager.close_instance(instance_id)?;
        
        Ok(())
    }
    
    /// Mounte einen Ordner für eine Origin
    pub fn mount_folder(
        &self,
        origin_url: String,
        folder_path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Mounte Ordner {} für {}", folder_path, origin_url);
        
        // Parse URL zu Origin
        let url = url::Url::parse(&origin_url)?;
        let origin = fenrir_secure::Origin::from_url(&url)
            .ok_or_else(|| anyhow::anyhow!("Ungültige URL: {}", origin_url))?;
        
        // Ordner mounten (über UI Manager, falls vorhanden)
        if let Some(ui_manager) = &self.ui_manager {
            ui_manager.handle_mount_folder(folder_path).await?;
        } else {
            // Direkt über Security Manager
            let mut sec_mgr = SecurityManager::new();
            sec_mgr.mount_directory(origin, std::path::PathBuf::from(folder_path))?;
        }
        
        Ok(())
    }
    
    /// Zeige Einstellungen-Dialog
    pub fn show_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Zeige Einstellungen-Dialog");
        
        if let Some(ui_manager) = &self.ui_manager {
            ui_manager.show_settings()?;
        }
        
        Ok(())
    }
    
    /// Zeige Berechtigungs-Manager
    pub fn show_permissions(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Zeige Berechtigungs-Manager");
        
        if let Some(ui_manager) = &self.ui_manager {
            ui_manager.show_permissions_manager()?;
        }
        
        Ok(())
    }
    
    /// Sauberes Herunterfahren aller Komponenten
    pub fn cleanup(&self) {
        info!("Starte Cleanup...");
        
        // Alle Servo-Instanzen schließen
        let instances = self.sandbox_manager.get_all_instances();
        for instance in instances {
            if let Err(e) = self.sandbox_manager.close_instance(instance.id()) {
                error!("Fehler beim Schließen von Instanz {}: {}", instance.id(), e);
            }
        }
        
        info!("Cleanup abgeschlossen");
    }
    
    /// Getter für App-Handle
    pub fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }
    
    /// Getter für Security Manager
    pub fn security_manager(&self) -> Arc<SecurityManager> {
        Arc::clone(&self.security_manager)
    }
    
    /// Getter für Sandbox Manager
    pub fn sandbox_manager(&self) -> Arc<SandboxManager> {
        Arc::clone(&self.sandbox_manager)
    }
    
    /// Getter für API Manager
    pub fn api_manager(&self) -> Arc<ApiManager> {
        Arc::clone(&self.api_manager)
    }
    
    /// Getter für UI Manager
    pub fn ui_manager(&self) -> Option<&UiManager> {
        self.ui_manager.as_ref()
    }
    
    /// Prüfe, ob die App initialisiert ist
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
