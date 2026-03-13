# Goose Task: Code Review Fenrir Browser

## Kontext

Fenrir ist ein Browser-Embedder geschrieben in Rust (Tauri v2 + Servo als Rendering Engine).
Du bist im Verzeichnis `/Users/anton.feldmann/Projects/priv/browser/fenrir`.

Aktueller Status:
- Servo wird via Git-Dependency (`v0.0.5`) eingebunden — kein lokaler Checkout nötig
- `fenrir-app` (Binary): winit Event-Loop, egui Toolbar, Servo WebView
- `crates/fenrir-servo`: Servo Wrapper + tokio Integration
- `crates/fenrir-network`: HTTP Interceptor Pipeline (DoH, HTTPS enforcer)
- `crates/fenrir-core`: Basis-Typen und Fehler
- `crates/fenrir-mcp`: MCP Server/Host (Planungsphase)
- `crates/fenrir-ai`: Lokale LLM Inference (candle)
- Debug Panel: F12 togglebar, zeigt Render-Pipeline, Servo-Events, Input-Events

## Bekannte Probleme

1. **Seite lädt aber zeigt "Application error: a client-side exception has occurred"**
   - DuckDuckGo lädt (HTTP 200 verifiziert), aber Inhalt wird nicht korrekt gerendert/angezeigt
   - blit-Callback Status noch unklar (Debug Panel soll das sichtbar machen)

2. **URL-Bar verliert sofort Fokus**
   - egui TextEdit widget verliert focus wenn Servo input events konsumiert
   - `egui_consumed` Flag in browser.rs wird gesetzt, aber Cursor verschwindet trotzdem

## Dein Auftrag

Bitte führe einen vollständigen Code-Review durch:

### 1. Rendering Pipeline (`fenrir-app/src/browser.rs`)
- Prüfe `render()` Funktion: ist `servo_ctx.make_current()` + `window_ctx.prepare_for_rendering()` korrekt?
- Vergleiche mit servoshell Pattern (v0.0.5): `rendering_context.make_current()` → `parent_context().prepare_for_rendering()` → `paint()` → `parent_context().present()`
- Fehlt ein `make_current()` Aufruf in der Render-Kette?
- Ist die Reihenfolge von `egui.paint()` und `window_ctx.present()` korrekt?

### 2. URL-Bar Fokus Problem (`fenrir-app/src/toolbar.rs` + `browser.rs`)
- Warum verliert die URL-Bar sofort den Fokus?
- `egui_consumed` in `on_window_event` sollte Servo-Events blockieren wenn egui konsumiert
- Prüfe ob `CursorMoved` Events an Servo durchkommen obwohl Toolbar aktiv ist
- Schlage Fixes vor oder implementiere sie

### 3. Allgemeine Code-Qualität
- Unused imports bereinigen (`fenrir-network`, `fenrir-servo`, `fenrir-ai` haben alle Warnings)
- Prüfe `crates/fenrir-servo/src/` — wird `FenrirHost` überhaupt noch verwendet oder ist es dead code?
- `browser.rs` hat einige doppelte `self.debug.borrow_mut()` Aufrufe — refactoren wenn sinnvoll

### 4. Build-Sauberkeit
```bash
cargo build 2>&1 | grep "^warning\|^error"
cargo clippy 2>&1 | grep "^warning\|^error" | head -30
```

## Wichtige Dateien

- `fenrir-app/src/browser.rs` — Haupt-Browser-State, Rendering, WebViewDelegate
- `fenrir-app/src/app.rs` — winit ApplicationHandler, Event-Loop
- `fenrir-app/src/toolbar.rs` — egui Toolbar + URL-Bar
- `fenrir-app/src/debug_panel.rs` — Debug Overlay
- `fenrir-app/src/resources.rs` — Servo Resource Reader

## Regeln

- Keine neuen Dependencies hinzufügen
- Keine Dateien > 150 Zeilen
- Kein `unwrap()` in Production-Code (nur in Tests)
- Antworten auf Deutsch
- Fixes direkt implementieren wenn sicher; sonst kommentieren
