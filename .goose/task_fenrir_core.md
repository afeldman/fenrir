# Goose Task: fenrir-core implementieren

## Ziel
Implementiere das Crate `fenrir-core` vollständig. Es ist das Fundament aller
anderen Fenrir-Crates und darf selbst KEINE externen Services, Netzwerk oder
Crypto enthalten. Nur Basis-Typen, Traits, Fehler und Event-Bus.

## Arbeitsverzeichnis
`/Users/anton.feldmann/Projects/priv/browser/fenrir/crates/fenrir-core/src/`

Die Datei `lib.rs` existiert bereits mit folgenden Modulen:
```rust
pub mod error;
pub mod event;
pub mod traits;
pub mod types;
```

## Aufgaben

### 1. `types.rs` — Basis-Typen

Erstelle folgende Typen:

```rust
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

/// Eindeutige ID für einen Browser-Tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(Uuid);

impl TabId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for TabId { fn default() -> Self { Self::new() } }

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Eindeutige Geräte-ID (persistent, für Sync).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(Uuid);

impl DeviceId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn as_str(&self) -> String { self.0.to_string() }
}

impl Default for DeviceId { fn default() -> Self { Self::new() } }

/// Unix-Timestamp in Millisekunden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self(ms)
    }
    pub fn as_millis(&self) -> u64 { self.0 }
}

/// Validierte URL (Newtype über url::Url).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FenrirUrl(Url);

impl FenrirUrl {
    pub fn parse(s: &str) -> Result<Self, url::ParseError> {
        Ok(Self(Url::parse(s)?))
    }
    pub fn as_url(&self) -> &Url { &self.0 }
    pub fn as_str(&self) -> &str { self.0.as_str() }
    pub fn scheme(&self) -> &str { self.0.scheme() }
}

impl std::fmt::Display for FenrirUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for FenrirUrl {
    type Error = url::ParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> { Self::parse(s) }
}
```

### 2. `error.rs` — Fehler-Hierarchie

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FenrirError {
    #[error("Servo initialization failed: {0}")]
    ServoInit(String),

    #[error("WebView creation failed: {0}")]
    WebViewCreate(String),

    #[error("Invalid URL")]
    InvalidUrl,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Kurzform für Result mit FenrirError.
pub type FenrirResult<T> = Result<T, FenrirError>;
```

### 3. `traits.rs` — Globale Traits

```rust
use crate::error::FenrirResult;

/// Alle Fenrir-Module implementieren diesen Trait.
/// Ermöglicht einheitlichen Start/Stop-Lifecycle.
pub trait FenrirModule: Send + Sync {
    /// Modulname für Logging und Fehlerberichte.
    fn name(&self) -> &'static str;

    /// Modul hochfahren (async, kann fehlschlagen).
    fn start(&self) -> impl std::future::Future<Output = FenrirResult<()>> + Send;

    /// Modul sauber herunterfahren.
    fn stop(&self) -> impl std::future::Future<Output = ()> + Send;

    /// Ist das Modul aktuell aktiv?
    fn is_running(&self) -> bool;
}

/// Typen die DSGVO-relevante Daten verwalten, implementieren diesen Trait.
pub trait Auditable {
    /// Beschreibung was dieser Typ an Daten hält und warum.
    fn audit_description(&self) -> &'static str;

    /// Alle gehaltenen Daten für DSGVO-Export serialisieren.
    fn export_data(&self) -> serde_json::Value;

    /// Alle Daten löschen (Recht auf Vergessenwerden).
    fn delete_all_data(&mut self);
}
```

### 4. `event.rs` — Event-Bus

```rust
use crate::types::{FenrirUrl, TabId, Timestamp};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Alle Browser-weiten Events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FenrirEvent {
    /// Tab wurde geöffnet.
    TabOpened { id: TabId, url: FenrirUrl, at: Timestamp },
    /// Tab wurde geschlossen.
    TabClosed { id: TabId, at: Timestamp },
    /// Navigation in einem Tab.
    TabNavigated { id: TabId, url: FenrirUrl, at: Timestamp },
    /// Tab-Titel geändert.
    TabTitleChanged { id: TabId, title: String },
    /// Netzwerkanfrage blockiert (Tracker, Phishing, etc.).
    RequestBlocked { tab: TabId, url: FenrirUrl, reason: BlockReason, at: Timestamp },
    /// Browser wird beendet.
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockReason {
    Tracker,
    Phishing,
    VpnKillSwitch,
    UserRule,
}

/// Kanal-Kapazität für den globalen Event-Bus.
const EVENT_BUS_CAPACITY: usize = 256;

/// Globaler Event-Bus (tokio broadcast).
pub struct EventBus {
    sender: broadcast::Sender<FenrirEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_BUS_CAPACITY);
        Self { sender }
    }

    /// Event publizieren. Ignoriert Fehler wenn keine Subscriber.
    pub fn publish(&self, event: FenrirEvent) {
        let _ = self.sender.send(event);
    }

    /// Subscriber erstellen.
    pub fn subscribe(&self) -> broadcast::Receiver<FenrirEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self { Self::new() }
}
```

## Tests

Erstelle am Ende jeder Datei ein `mod tests {}` mit sinnvollen Unit-Tests:
- `types.rs`: TabId Uniqueness, Timestamp::now() > 0, FenrirUrl::parse valid/invalid
- `error.rs`: FenrirError Display-Ausgaben testen
- `event.rs`: EventBus publish/subscribe, mehrere Subscriber

## Regeln

- Kein `unwrap()` außer in Tests
- Keine `panic!` außer in Tests
- Alle pub Typen mit `#[derive(Debug)]`
- Kein Clone für Secrets/Keys (hier nicht relevant, aber merken)
- Dokumentation auf Englisch
- `cargo check` muss am Ende erfolgreich sein

## Abschluss

Nach der Implementierung:
```bash
cd /Users/anton.feldmann/Projects/priv/browser/fenrir
cargo check -p fenrir-core
```

Wenn erfolgreich: done.
