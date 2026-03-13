//! Fenrir Logging — Enhanced Rolling File Logger with hybrid tracing/inqjet backend.
//!
//! Uses fenrir-log crate for hybrid logging architecture.

use std::path::PathBuf;
use fenrir_log::FenrirLogger;

/// Guard muss in main() gehalten werden (sonst flush verloren).
pub struct LogGuard(pub FenrirLogger);

/// Initialisiert Hybrid-Logger mit Rolling-File + stderr + optional inqjet backend.
/// Gibt LogGuard zurück — muss in main() als Variable gehalten werden!
pub fn init() -> anyhow::Result<LogGuard> {
    let log_dir = log_dir();
    
    // Konfiguriere Logger mit allen Features
    let logger = FenrirLogger::builder()
        .with_tracing()
        .with_perf_backend()  // Inqjet für Performance-Logs
        .with_rolling_file(
            &log_dir,
            "fenrir.log",
            10,  // 10MB pro Datei
            7,   // Max 7 Dateien behalten
        )
        .with_directive("fenrir=trace")
        .with_directive("fenrir_app=trace")
        .with_directive("fenrir_core=trace")
        .with_directive("fenrir_servo=trace")
        .with_directive("fenrir_network=trace")
        .with_directive("fenrir_ai=trace")
        .with_directive("servo=debug")
        .with_directive("net=debug")
        .with_directive("constellation=warn")
        .with_directive("winit=debug")
        .with_directive("egui=debug")
        .with_directive("egui_glow=debug")
        .with_directive("egui_winit=debug")
        .build()?;

    tracing::info!(
        log_dir = %log_dir.display(),
        "Fenrir Hybrid-Logger initialisiert mit TRACE-Level"
    );
    tracing::debug!("Log-Datei: {}/fenrir.log", log_dir.display());
    tracing::trace!("Logger vollständig konfiguriert mit tracing + inqjet backend");

    Ok(LogGuard(logger))
}

/// Performance-kritische Logging für Netzwerk, Rendering, etc.
pub fn perf_log(target: &str, message: impl std::fmt::Display) {
    // Diese Funktion kann von überall aufgerufen werden
    // Sie verwendet den globalen Logger oder falls nicht verfügbar, tracing
    // Das target: Argument muss ein String-Literal sein, also verwenden wir einen festen Wert
    // und fügen das dynamische target in die Nachricht ein
    tracing::debug!(target: "fenrir::perf", "[PERF][{}] {}", target, message);
}

/// Beispiel für strukturiertes Logging mit Pod-Typen
#[cfg(feature = "perf")]
pub fn log_network_request(url: &str, method: &str, duration_ms: u64, status_code: u16) {
    use fenrir_log::NetworkRequest;
    
    let request = NetworkRequest {
        url: url.to_string(),
        method: method.to_string(),
        duration_ms,
        status_code,
    };
    
    // In einer realen Implementierung würden wir den globalen Logger verwenden
    tracing::info!(
        target: "network",
        "Request: {} {} ({}ms, status: {})",
        method, url, duration_ms, status_code
    );
}

pub fn log_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("fenrir")
        .join("logs")
}
