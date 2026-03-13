//! Fenrir Browser — EU-first Privacy Browser.
//!
//! Einstiegspunkt: initialisiert rustls, winit Event-Loop und den Browser.

mod app;
mod browser;
mod debug_panel;
mod input;
mod logging;
mod resources;
mod toolbar;
mod waker;

use fenrir_config::load;
use fenrir_i18n::{set_language, Language};
use winit::event_loop::EventLoop;

use app::BrowserApp;
use waker::FenrirWaker;

fn init_i18n() -> anyhow::Result<()> {
    // Load configuration to get language setting
    let config = load().unwrap_or_default();
    
    // Parse language from config
    let language = Language::from_str(&config.ui.language)
        .unwrap_or_else(|_| {
            tracing::warn!("Invalid language '{}' in config, using default", config.ui.language);
            Language::default()
        });
    
    // Initialize i18n with the configured language
    set_language(language)
        .map_err(|e| anyhow::anyhow!("Failed to initialize i18n: {}", e))?;
    
    tracing::info!("Initialized i18n with language: {}", language);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // ERST Logger initialisieren, dann alles andere
    let log_guard = logging::init()?;
    
    // TLS Crypto-Provider (von Servo/rustls benötigt)
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("rustls crypto provider");

    // Initialize internationalization
    init_i18n()?;

    // Servo-Ressourcen registrieren (CSS, Icons, etc.)
    resources::init();

    // winit Event-Loop mit Custom-Event für Servo-Wakeups
    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("EventLoop");

    let waker = FenrirWaker::new(event_loop.create_proxy());
    let mut app = BrowserApp::new(waker);

    event_loop.run_app(&mut app)?;
    
    // LogGuard wird hier gelöscht und flushed automatisch
    drop(log_guard);
    Ok(())
}
