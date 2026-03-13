//! Fenrir Browser — EU-first Privacy Browser.
//!
//! Einstiegspunkt: initialisiert rustls, winit Event-Loop und den Browser.

mod app;
mod browser;
mod debug_panel;
mod input;
mod resources;
mod toolbar;
mod waker;

use tracing::info;
use winit::event_loop::EventLoop;

use app::BrowserApp;
use waker::FenrirWaker;

fn main() -> anyhow::Result<()> {
    // Logging initialisieren
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fenrir=debug".parse()?)
                .add_directive("servo=info".parse()?),
        )
        .init();

    // TLS Crypto-Provider (von Servo/rustls benötigt)
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("rustls crypto provider");

    info!("Fenrir Browser startet …");

    // Servo-Ressourcen registrieren (CSS, Icons, etc.)
    resources::init();

    // winit Event-Loop mit Custom-Event für Servo-Wakeups
    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("EventLoop");

    let waker = FenrirWaker::new(event_loop.create_proxy());
    let mut app = BrowserApp::new(waker);

    event_loop.run_app(&mut app)?;
    Ok(())
}
