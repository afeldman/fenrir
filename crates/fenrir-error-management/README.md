# fenrir-error-management

Ein umfassendes Fehler-Management-System für den Fenrir Browser mit AI-Integration.

## Überblick

Dieses Crate bietet ein strukturiertes System zur Verwaltung aller auftretenden Fehler im Fenrir Browser. Jeder Fehler wird mit der Struktur `(id, message, hilfe)` gespeichert. Wenn keine Hilfe verfügbar ist, wird automatisch Fenrir-AI genutzt, um einen Vorschlag zu generieren.

## Features

- **Strukturierte Fehler-Datenbank**: SQLite-basierte Speicherung aller Fehler
- **Automatische Klassifikation**: 11 Fehler-Kategorien mit Tags
- **AI-Integration**: Nutzung von Fenrir-AI (Noeum-1-Nano) für Hilfe-Vorschläge
- **Regelbasierte Fallback-Hilfe**: Für bekannte Fehlermuster
- **Umfassende Statistiken**: Fehlerhäufigkeiten, Kategorie-Verteilung, Erfolgsquoten
- **Thread-safe Design**: Sichere Nutzung in multithreaded Umgebungen
- **Einfache Integration**: Nahtlose Integration mit bestehenden `FenrirError`-Typen

## Installation

Fügen Sie folgendes zu Ihrer `Cargo.toml` hinzu:

```toml
[dependencies]
fenrir-error-management = { path = "../crates/fenrir-error-management" }
```

Für AI-Integration aktivieren Sie das Feature:

```toml
[dependencies]
fenrir-error-management = { path = "../crates/fenrir-error-management", features = ["ai-integration"] }
```

## Schnellstart

```rust
use fenrir_error_management::{ErrorManager, ErrorCategory};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ErrorManager initialisieren (In-Memory für Tests)
    let manager = ErrorManager::new(":memory:").await?;
    
    // Fehler registrieren
    let error_id = manager.register_error(
        "Network request timed out after 30 seconds",
        ErrorCategory::Network,
        Some("fenrir-network"),
        vec!["timeout", "http", "connection"],
    ).await?;
    
    // Hilfe abrufen (wird automatisch generiert falls nötig)
    let help = manager.get_help(&error_id).await?;
    println!("Hilfe: {}", help);
    
    // Statistiken anzeigen
    let stats = manager.get_statistics().await?;
    println!("Gesamtfehler: {}", stats.total_errors);
    
    Ok(())
}
```

## Fehler-Kategorien

Das System unterstützt folgende Fehler-Kategorien:

| Kategorie | Beschreibung |
|-----------|--------------|
| `Network` | Netzwerk-bezogene Fehler |
| `Configuration` | Konfigurationsfehler |
| `Io` | Eingabe/Ausgabe Fehler |
| `Crypto` | Kryptographie-Fehler |
| `Storage` | Speicher/Storage Fehler |
| `Permission` | Berechtigungsfehler |
| `Ai` | AI/ML-bezogene Fehler |
| `Browser` | Browser-spezifische Fehler |
| `System` | Systemfehler (Betriebssystem, Hardware) |
| `Application` | Anwendungslogik-Fehler |
| `Unknown` | Unbekannte oder nicht klassifizierbare Fehler |

## Hilfe-Generierungspipeline

Das System versucht Hilfe in folgender Reihenfolge zu generieren:

1. **Vordefinierte Hilfe**: Wenn beim Registrieren angegeben
2. **Regelbasierte Hilfe**: Für bekannte Fehlermuster (z.B. "network timeout")
3. **AI-generierte Hilfe**: Mit Fenrir-AI (Noeum-1-Nano), wenn Feature aktiviert
4. **Fallback-Hilfe**: Generische Hilfe mit grundlegenden Troubleshooting-Schritten

## Integration mit bestehenden Fehlern

Das System kann direkt mit bestehenden `FenrirError`-Typen aus `fenrir-core` integriert werden:

```rust
use fenrir_core::error::FenrirError;
use fenrir_error_management::ErrorManager;

async fn handle_error(manager: &ErrorManager, error: FenrirError) {
    let error_id = manager.register_fenrir_error(&error, Some("my-module")).await.unwrap();
    let help = manager.get_help(&error_id).await.unwrap();
    
    // Hilfe dem Benutzer anzeigen oder loggen
    eprintln!("Fehler aufgetreten: {}", error);
    eprintln!("Hilfe: {}", help);
}
```

## API-Übersicht

### ErrorManager
Haupt-API für das Fehler-Management:

- `new(db_path: &str) -> ErrorManagementResult<Self>`: Initialisiert den Manager
- `register_error(...) -> Result<String, ...>`: Registriert einen neuen Fehler
- `get_help(error_id: &str) -> Result<String, ...>`: Holt Hilfe für einen Fehler
- `search_errors(...) -> Result<Vec<ErrorRecord>, ...>`: Sucht Fehler nach Kriterien
- `get_statistics() -> Result<ErrorStatistics, ...>`: Holt Statistiken
- `register_fenrir_error(...) -> Result<String, ...>`: Registriert einen FenrirError

### Datenmodelle

- `ErrorRecord`: Hauptstruktur für Fehler (id, message, hilfe, category, etc.)
- `ErrorCategory`: Enum der Fehler-Kategorien
- `ErrorStatistics`: Statistik-Daten für Analyse
- `HelpSuggestion`: AI-generierte Hilfe-Vorschläge

## Konfiguration

### Datenbank-Pfad
Die Datenbank kann im Dateisystem oder In-Memory gespeichert werden:

```rust
// Persistente Speicherung
let manager = ErrorManager::new("~/.fenrir/errors.db").await?;

// In-Memory für Tests
let manager = ErrorManager::new(":memory:").await?;
```

### AI-Integration
Die AI-Integration ist optional und wird über das Feature-Flag `ai-integration` aktiviert. Wenn aktiviert, wird automatisch versucht, das Noeum-1-Nano Modell zu laden.

## Beispiele

Das Repository enthält ein vollständiges Beispiel:

```bash
cd crates/fenrir-error-management
cargo run --example basic
```

## Tests

Führen Sie die Tests aus mit:

```bash
cargo test
```

## Lizenz

Dieses Projekt ist unter der EUPL-1.2 Lizenz lizenziert.

## Beitragen

Beiträge sind willkommen! Bitte erstellen Sie ein Issue oder einen Pull Request auf GitHub.
