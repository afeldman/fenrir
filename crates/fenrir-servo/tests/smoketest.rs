//! Servo Smoketest — prüft ob Servo headless initialisiert und eine WebView laden kann.
//!
//! WICHTIG: Nur ein Test pro Binary möglich, da Servo globalen Zustand (Options, Logger)
//! nur einmal pro Prozess initialisiert. Jede weitere Initialisierung panics.
//! → Alles in einem Test zusammengefasst.

use dpi::PhysicalSize;
use fenrir_network::FenrirNetworkHandler;
use fenrir_servo::{FenrirServoDelegate, FenrirWebViewDelegate};
use paint_api::rendering_context::SoftwareRenderingContext;
use servo::{LoadStatus, Preferences, RenderingContext, ServoBuilder, WebViewBuilder};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct TestWaker(Arc<AtomicBool>);

impl embedder_traits::EventLoopWaker for TestWaker {
    fn clone_box(&self) -> Box<dyn embedder_traits::EventLoopWaker> {
        Box::new(TestWaker(Arc::clone(&self.0)))
    }
    fn wake(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// Initialisiert Servo headless, öffnet eine WebView und lädt about:blank.
/// Ein einzelner Test weil Servo globale Opts nur einmal pro Prozess setzt.
#[test]
fn servo_init_and_load_about_blank() {
    // --- Servo initialisieren ---
    let woken = Arc::new(AtomicBool::new(false));

    let mut prefs = Preferences::default();
    prefs.network_http_proxy_uri = String::new();
    prefs.network_https_proxy_uri = String::new();

    let servo = ServoBuilder::default()
        .preferences(prefs)
        .event_loop_waker(Box::new(TestWaker(Arc::clone(&woken))))
        .build();

    // Tokio Runtime für Netzwerk-Init (about:blank braucht keine echten Requests)
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let network = Arc::new(
        rt.block_on(FenrirNetworkHandler::with_defaults())
            .expect("FenrirNetworkHandler init"),
    );
    servo.set_delegate(Rc::new(FenrirServoDelegate::new(network)));

    // Event Loop einmal drehen — kein Panic = Servo läuft
    servo.spin_event_loop();

    // --- WebView (Tab) öffnen ---
    let rendering_context = Rc::new(
        SoftwareRenderingContext::new(PhysicalSize {
            width: 800,
            height: 600,
        })
        .expect("SoftwareRenderingContext konnte nicht erstellt werden"),
    );
    rendering_context
        .make_current()
        .expect("make_current fehlgeschlagen");

    let url = url::Url::parse("about:blank").unwrap();
    let webview = WebViewBuilder::new(&servo, rendering_context)
        .url(url)
        .delegate(Rc::new(FenrirWebViewDelegate::new()))
        .build();

    // about:blank laden lassen (max 1 Sekunde)
    for _ in 0..500 {
        servo.spin_event_loop();
        if webview.load_status() == LoadStatus::Complete {
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }

    assert_eq!(
        webview.load_status(),
        LoadStatus::Complete,
        "about:blank wurde nicht geladen"
    );
}
