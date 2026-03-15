# MCP Provider Architecture

Status: **Goose-Task aktiv** — `task_fenrir_mcp_providers.md`

---

## Übersicht

`fenrir-mcp` ist der zentrale KI-Router. Er entscheidet welcher Provider eine Anfrage bearbeitet und abstrahiert das Modell-Backend vom Rest des Browsers.

```
fenrir-ai / fenrir-bookmarks / fenrir-ui
          ↓
    LlmRouter (fenrir-mcp)
          ↓
  ┌───────┬───────────┬─────────┐
  │ Local │ Mammoth   │ Ollama  │
  │(stdio)│(HTTP/SSE) │(HTTP)   │
  └───────┴───────────┴─────────┘
```

---

## Provider Trait

```rust
// crates/fenrir-mcp/src/provider.rs
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, FenrirError>;
    async fn health_check(&self) -> bool;
}
```

---

## Provider: Local (Priorität 1)

- **Transport:** stdio (MCP-Protokoll)
- **Modell:** noeum-1-nano via `fenrir-ai`
- **Datei:** `crates/fenrir-mcp/src/local.rs`
- **Status:** Stub — `fenrir-ai`-Inferenz muss zuerst fertig sein

```rust
pub struct LocalLlmProvider {
    inference: Arc<FenrirAiRuntime>,
}
```

---

## Provider: Mammoth (Priorität 2)

- **Transport:** HTTP + SSE-Streaming
- **Endpoint:** Konfigurierbar via `fenrir-config`
- **Datei:** `crates/fenrir-mcp/src/mammoth.rs`
- **Status:** Stub — HTTP-Client und SSE-Parser fehlen
- **Auth:** Bearer-Token aus Keychain (via `fenrir-crypto` in Phase 3)

---

## Provider: Ollama (Priorität 3)

- **Transport:** HTTP (`http://localhost:11434`)
- **API:** `/api/generate`, `/api/chat` (OpenAI-kompatibel)
- **Datei:** noch nicht angelegt
- **Status:** Geplant für Phase 2

---

## LlmRouter

`LlmRouter` wählt Provider nach Verfügbarkeit:

1. Local (wenn `fenrir-ai` bereit)
2. Mammoth (wenn konfiguriert und erreichbar)
3. Ollama (wenn läuft)
4. Fehler (kein Cloud-Fallback erlaubt)

```rust
pub struct LlmRouter {
    providers: Vec<Box<dyn LlmProvider>>,
}

impl LlmRouter {
    pub async fn route(&self, req: LlmRequest) -> Result<LlmResponse, FenrirError> {
        for provider in &self.providers {
            if provider.health_check().await {
                return provider.complete(req).await;
            }
        }
        Err(FenrirError::Ai("No provider available".into()))
    }
}
```

---

## Konfiguration

```toml
# fenrir-config: ai-Sektion
[ai]
provider_priority = ["local", "mammoth", "ollama"]

[ai.mammoth]
endpoint = "https://api.mammoth.ai/v1"
# token via keychain — nie in Config-Datei

[ai.ollama]
endpoint = "http://localhost:11434"
```

---

## Nächste Schritte (Goose-Task)

1. `local.rs` — `FenrirAiRuntime`-Integration sobald `inference.rs` fertig
2. `mammoth.rs` — reqwest-Client + SSE-Streaming implementieren
3. `ollama.rs` — neue Datei, OpenAI-kompatibler HTTP-Client
4. `LlmRouter` — Health-Check-Loop, Fallback-Logik
5. Tests: Mock-Provider für Unit-Tests
