# Goose Task: fenrir-network BandwidthMonitor

## Ziel
Implementiere `bandwidth.rs` in `fenrir-network`. Misst ein- und ausgehenden
Traffic in Bytes, pro Session und kumulativ.

## Arbeitsverzeichnis
`/Users/anton.feldmann/Projects/priv/browser/fenrir/crates/fenrir-network/src/`

Die Datei `bandwidth.rs` existiert noch nicht — erstelle sie.

## Implementierung

```rust
//! BandwidthMonitor — misst Netzwerk-Traffic in Bytes.
//!
//! Thread-sicher via Atomics (kein Mutex nötig).
//! Genutzt von FenrirHttpClient nach jedem Request.

use atomic_float::AtomicF64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Default)]
pub struct BandwidthMonitor {
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    /// Requests pro Sekunde (gleitender Durchschnitt)
    requests_total: AtomicU64,
}

impl BandwidthMonitor {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Ausgehende Bytes erfassen (Request Body + Headers).
    pub fn record_sent(&self, bytes: u64) {
        self.bytes_sent.fetch_add(bytes, Ordering::Relaxed);
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Eingehende Bytes erfassen (Response Body + Headers).
    pub fn record_received(&self, bytes: u64) {
        self.bytes_received.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn total_sent(&self) -> u64 {
        self.bytes_sent.load(Ordering::Relaxed)
    }

    pub fn total_received(&self) -> u64 {
        self.bytes_received.load(Ordering::Relaxed)
    }

    pub fn total_requests(&self) -> u64 {
        self.requests_total.load(Ordering::Relaxed)
    }

    /// Zusammenfassung als lesbarer String (für UI/Debug).
    pub fn summary(&self) -> String {
        format!(
            "↑ {} ↓ {} ({} Requests)",
            format_bytes(self.total_sent()),
            format_bytes(self.total_received()),
            self.total_requests()
        )
    }

    /// Zähler zurücksetzen (z.B. bei Session-Ende).
    pub fn reset(&self) {
        self.bytes_sent.store(0, Ordering::Relaxed);
        self.bytes_received.store(0, Ordering::Relaxed);
        self.requests_total.store(0, Ordering::Relaxed);
    }
}

/// Bytes in lesbare Einheit umrechnen (B, KB, MB, GB).
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_sent_and_received() {
        let monitor = BandwidthMonitor::new();
        monitor.record_sent(1024);
        monitor.record_received(2048);
        assert_eq!(monitor.total_sent(), 1024);
        assert_eq!(monitor.total_received(), 2048);
        assert_eq!(monitor.total_requests(), 1);
    }

    #[test]
    fn reset_clears_all() {
        let monitor = BandwidthMonitor::new();
        monitor.record_sent(500);
        monitor.record_received(1000);
        monitor.reset();
        assert_eq!(monitor.total_sent(), 0);
        assert_eq!(monitor.total_received(), 0);
        assert_eq!(monitor.total_requests(), 0);
    }

    #[test]
    fn format_bytes_correct_units() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1500), "1.5 KB");
        assert_eq!(format_bytes(1_500_000), "1.43 MB");
    }

    #[test]
    fn summary_format() {
        let monitor = BandwidthMonitor::new();
        monitor.record_sent(1024);
        monitor.record_received(2048);
        let s = monitor.summary();
        assert!(s.contains("KB"));
        assert!(s.contains("1 Requests"));
    }

    #[test]
    fn thread_safe_concurrent_access() {
        use std::thread;
        let monitor = Arc::new(BandwidthMonitor::default());
        let mut handles = vec![];
        for _ in 0..10 {
            let m = Arc::clone(&monitor);
            handles.push(thread::spawn(move || {
                m.record_sent(100);
                m.record_received(200);
            }));
        }
        for h in handles { h.join().unwrap(); }
        assert_eq!(monitor.total_sent(), 1000);
        assert_eq!(monitor.total_received(), 2000);
        assert_eq!(monitor.total_requests(), 10);
    }
}
```

## Abschluss

```bash
cd /Users/anton.feldmann/Projects/priv/browser/fenrir
cargo test -p fenrir-network 2>&1 | grep -E "ok|FAILED|error"
```

Wenn alles grün: done.
