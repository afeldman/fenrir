# Fenrir Browser - Servo Integration Architecture

Dieses Projekt integriert die Servo-Browser-Engine in eine hochsichere Tauri-Umgebung mit vier spezialisierten Crates.

## Architektur-Übersicht

```
┌─────────────────────────────────────────────────────────────┐
│                    fenrir-ui (Native UI)                    │
│  • Native Tauri/egui Komponenten (kein WebView!)           │
│  • URL-Leiste, Tabs, Browser-Steuerung                     │
│  • Ereignis-Handling von Servo                             │
└───────────────┬─────────────────────────────────────────────┘
                │
┌───────────────▼─────────────────────────────────────────────┐
│                 fenrir-api (Kommunikation)                  │
│  • IPC-Commands zwischen Servo und Hauptprozess            │
│  • Servo-Embedding-Nachrichten-Adapter                     │
│  • Berechtigungsprüfung vor Command-Ausführung             │
└───────────────┬─────────────────────────────────────────────┘
                │
┌───────────────▼─────────────────────────────────────────────┐
│                fenrir-sandbox (Isolation)                   │
│  • Servo-Instanzen in eigenen Tauri-Fenstern               │
│  • WindowBuilder-Integration für Sandboxing                │
│  • Prozess-Lebenszyklus-Management                         │
└───────────────┬─────────────────────────────────────────────┘
                │
┌───────────────▼─────────────────────────────────────────────┐
│                fenrir-secure (Sicherheit)                   │
│  • Origin-Policy-System (kompatibel mit Servo)             │
│  • Pfadkanonisierung und -validierung                      │
│  • Berechtigungsregister für Servo-Instanzen               │
└───────────────┬─────────────────────────────────────────────┘
                │
                ▼
        ┌───────────────┐
        │    Servo      │
        │  Browser-Engine │
        └───────────────┘
```

## Crates im Detail

### 1. fenrir-secure
**Core-Sicherheit und Berechtigungen, SERVO-BEWUSST**

- **Origin-Policy-System**: Kompatibel mit Servo's bestehendem Permission-Delegation
- **Pfadkanonisierung**: Verhindert Path-Traversal-Angriffe
- **Berechtigungsregister**: Zentrale Verwaltung aller aktiven Servo-Instanzen
- **Mount-System**: Bestimmt, welche Origins auf welche lokalen Verzeichnisse zugreifen dürfen

**Wichtige Typen:**
- `SecurityManager`: Haupt-Sicherheitsmanager
- `Origin`: Web-Origin (scheme + host + port)
- `OriginPolicy`: Regeln für was eine Origin darf
- `PermissionSet`: Menge von Berechtigungen

### 2. fenrir-sandbox
**Verwaltung der Servo-Tab-Prozesse in Tauri**

- **Servo-Embedding**: Nutzt Servo's Embedding-APIs (`servo::Servo`, `servo::WebView`)
- **Tauri-Fenster**: Jede Servo-Instanz läuft in eigenem `tauri::Window`
- **Lebenszyklus**: Starten, Stoppen und Aufräumen von Instanzen
- **Ereignis-Handling**: Weiterleitung von Servo-Ereignissen an UI

**Wichtige Typen:**
- `SandboxManager`: Verwaltet alle Servo-Instanzen
- `ServoInstance`: Einzelne Servo-Instanz mit Tauri-Fenster
- `WindowConfig`: Konfiguration für Sandbox-Fenster

### 3. fenrir-api
**API für Kommunikation zwischen Servo-Tabs und Hauptprozess**

- **IPC-Commands**: Definiert, was Servo tun kann (`file_read`, `notification_send`, etc.)
- **Servo-Adapter**: Wandelt Servo-Embedding-Nachrichten in API-Commands um
- **Berechtigungsprüfung**: Jedes Command prüft zuerst `fenrir-secure`
- **Asynchron**: Kompatibel mit Servo's asynchroner Architektur

**Wichtige Typen:**
- `ApiManager`: Haupt-API-Manager
- `ApiCommand`: Definiert verfügbare Commands
- `ServoMessageAdapter`: Konvertiert zwischen Servo- und API-Format

### 4. fenrir-ui
**Native Benutzeroberfläche in Rust mit Tauri**

- **Native Komponenten**: Verwendet Tauri und egui, **kein WebView**
- **Tab-Management**: Native Tab-Leiste und URL-Eingabe
- **Servo-Steuerung**: Startet/stoppt Servo-Instanzen über `fenrir-sandbox`
- **Ereignis-Anzeige**: Zeigt Servo-Ereignisse (Ladefortschritt, Titeländerung)

**Wichtige Typen:**
- `UiManager`: Koordiniert alle UI-Komponenten
- `BrowserState`: Verwaltet Tabs und Browser-Zustand
- `WindowManager`: Verwaltet Tauri-Fenster

## Servo-Integration

### Wichtige Servo-APIs

1. **Embedding-API** (`servo::Servo`, `servo::WebView`)
   - Haupt-Einstiegspunkt für Servo-Integration
   - `ServoBuilder` zum Erstellen von Servo-Instanzen
   - `WebViewBuilder` für einzelne Tabs/Seiten

2. **Embedder Traits** (`embedder_traits`)
   - `WebViewDelegate` für Ereignis-Handling
   - `PermissionRequest` für Berechtigungsanfragen
   - `NavigationRequest` für Navigation

3. **Event Loop** (`servo::Servo::spin_event_loop()`)
   - Servo's Haupt-Event-Loop
   - Muss vom Embedder aufgerufen werden

### Integration mit Servo's Permission-System

```rust
// fenrir-secure integriert sich mit Servo's Permission-Delegation
fn handle_servo_permission_request(
    &self,
    request: PermissionRequest,  // Von Servo
    instance_id: &str,
) -> Result<PermissionGrant, SecurityError> {
    // Prüfe Berechtigungen basierend auf Origin-Policies
    // Entscheidung wird an Servo zurückgegeben
}
```

## Beispiel: "URL eingeben und Ordner mounten"

### Ablauf:

1. **Benutzer gibt URL ein** (`https://example.com`)
   - `fenrir-ui` erfasst die Eingabe
   - `fenrir-sandbox` startet neue Servo-Instanz
   - `fenrir-secure` registriert Instanz mit Origin

2. **Benutzer klickt "Ordner mounten"**
   - `fenrir-ui` zeigt Dateiauswahl-Dialog
   - `fenrir-secure` mounted Verzeichnis für Origin
   - Origin `https://example.com` erhält Zugriff auf `/Users/benutzer/Dokumente`

3. **Servo möchte Datei lesen**
   - Servo sendet `file_read` Nachricht über Embedding-API
   - `fenrir-api` empfängt und konvertiert zu `CommandRequest`
   - `fenrir-secure` prüft: Hat Origin Berechtigung für Pfad?
   - Wenn ja: Datei wird gelesen, Ergebnis zurück an Servo
   - Wenn nein: Fehler zurück an Servo

### Code-Ausschnitt:

```rust
// 1. URL eingeben
let url = Url::parse("https://example.com")?;
let origin = Origin::from_url(&url)?;

// 2. Servo-Instanz erstellen
let instance_id = sandbox_manager.create_instance(app_handle, url, window_config).await?;
security_manager.register_servo_instance(instance_id, origin.clone(), servo_instance);

// 3. Ordner mounten
security_manager.mount_directory(
    origin,
    PathBuf::from("/Users/benutzer/Dokumente"),
)?;

// 4. Servo möchte Datei lesen
// Servo sendet: {"type": "file_read", "path": "/Users/benutzer/Dokumente/test.txt"}
// fenrir-api prüft Berechtigungen über fenrir-secure
// Wenn erlaubt: Datei wird gelesen und zurückgegeben
```

## Sicherheitsmerkmale

### 1. Defense in Depth
- **Servo's Sandboxing**: Bereits vorhandene Multiprozess-Isolation
- **Tauri Sandbox**: Jede Servo-Instanz in eigenem Tauri-Fenster
- **Origin-Policies**: Granulare Berechtigungen pro Origin
- **Pfadvalidierung**: Verhindert Path-Traversal

### 2. Principle of Least Privilege
- Servo-Tabs haben nur API-Zugriff, keine direkten Systemaufrufe
- Jeder API-Call prüft Berechtigungen
- Default: Alles verboten, muss explizit erlaubt werden

### 3. Secure by Design
- **Native UI**: Kein WebView für UI → Keine XSS-Gefahr
- **Type Safety**: Rust's Typsystem verhindert viele Sicherheitsfehler
- **Memory Safety**: Keine Buffer Overflows, Use-After-Free, etc.

## Entwicklung

### Abhängigkeiten

```toml
# Servo (git dependency)
servo = { git = "https://github.com/servo/servo", tag = "v0.0.5" }
embedder_traits = { git = "https://github.com/servo/servo", tag = "v0.0.5" }

# Tauri
tauri = { version = "2.0", features = ["api-all", "menu", "tray"] }

# UI
egui = "0.27"
eframe = "0.27"
```

### Build

```bash
# Gesamtes Projekt
cargo build --release

# Einzelne Crates
cargo build -p fenrir-secure
cargo build -p fenrir-sandbox
cargo build -p fenrir-api
cargo build -p fenrir-ui
```

### Test

```bash
# Integrationstest
cargo run --example integration_example

# Unit Tests
cargo test --all
```

## Nächste Schritte

1. **Servo-Embedding implementieren**: `fenrir-sandbox` muss Servo tatsächlich starten
2. **Tauri-Integration**: `fenrir-ui` muss mit Tauri's Event-System integriert werden
3. **egui-UI**: Native UI-Komponenten mit egui implementieren
4. **API-Handler**: Konkrete Implementierung der API-Commands
5. **Permission-UI**: Benutzeroberfläche für Berechtigungsverwaltung

## Lizenz

EUPL-1.2 (wie der Rest des Fenrir-Projekts)
