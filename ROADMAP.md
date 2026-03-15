# Fenrir Roadmap

Status-Legende: ✅ Done · 🔄 Active · 📋 Planned · 🔮 Vision

---

## Phase 1 — Foundation ✅

**Ziel:** Browser startet, Seiten laden, Infrastruktur steht.

### Fertige Crates
| Crate | Was es tut |
|---|---|
| `fenrir-app` | winit + egui, Toolbar, URL-Bar, Debug-Panel (F12) |
| `fenrir-core` | BrowserState, TabId, FenrirUrl, Event-Bus, FenrirError |
| `fenrir-config` | TOML-Loader, Feature-Flags, User-Profile |
| `fenrir-network` | HTTP-Client (reqwest), DNS (hickory), HTTPS (rustls), Interceptor |
| `fenrir-servo` | Servo-Wrapper, WebView-Delegate, FenrirHost |
| `fenrir-log` | tracing + logroller, Rolling-Logs |
| `fenrir-error-management` | Zentrales Error-Routing, DB-Logging, AI-Helper-Stubs |
| `fenrir-metrics` | Render-, Memory-, Network-Metriken, Anomalie-Erkennung |
| `fenrir-secure` | Origin-Validierung, Path-Sanitization, Permissions-Registry |

### Features
- Browser startet und navigiert HTTP/HTTPS
- Servo-Rendering funktioniert (einfache Seiten)
- F12 Debug-Panel
- Rolling-Logs nach `~/Library/Application Support/fenrir/logs/`

### Docs: `docs/current/`

---

## Phase 2 — AI + Config + MCP 🔄

**Ziel:** Lokale AI-Inferenz läuft, Config-System vollständig, MCP-Provider verbunden.

**Goose-Tasks aktiv:** `task_ai_noeum.md`, `task_fenrir_mcp_providers.md`, `task_fenrir_config.md`, `task_fenrir_core.md`, `task_logging.md`, `task_cleanup.md`

### Crates in Arbeit
| Crate | Aufgabe | Abhängigkeit |
|---|---|---|
| `fenrir-ai` | noeum-1-nano: HF-Download, Tokenizer, Candle-Inference, Bookmarks-API | — |
| `fenrir-mcp` | Provider-Impl: `local.rs` (stdio), `mammoth.rs` (HTTP/SSE), Ollama | `fenrir-ai` |
| `fenrir-config` | AI-Config-Sektion, MCP-Config, CLI-Override | — |
| `fenrir-core` | Event-Bus vollständig, Tab-State-Refactoring | — |

### Features
- Lokale Inferenz mit noeum-1-nano (0.6B, Apache 2.0)
- MCP-Router verbindet Local + Mammoth + Ollama
- Config-System mit Profilen
- Compiler-Warnings bereinigt, `browser.rs` refactored (433 → ~3 Dateien)

### Bekannte Blockierer
- Rendering: blit-callback-Status unklar, URL-Bar verliert Fokus bei Servo-Input
- fenrir-ai: `inference.rs`, `tokenizer.rs`, `download.rs` sind Stubs

### Docs: `docs/active/ai-inference-architecture.md`, `docs/active/mcp-providers.md`

---

## Phase 3 — Core Browser Features 📋

**Ziel:** Vollwertiger Privacy-Browser mit Bookmarks, Krypto-Fundament, Tracker-Blocking.

**Voraussetzung:** Phase 2 abgeschlossen (AI-Inferenz läuft)

### Neue Crates
| Crate | Was es tut | Abhängigkeiten |
|---|---|---|
| `fenrir-crypto` | ChaCha20-Poly1305, Ed25519, Argon2id, BLAKE3, keyring-rs, zeroize | — (Leaf-Crate) |
| `fenrir-bookmarks` | Speicherung (redb), AI-Klassifikation, Import/Export | `fenrir-ai`, `fenrir-crypto` |
| `fenrir-privacy` | Tracker-Blocking (uBlock-Format), DNS-Sinkhole, Request-Filter | `fenrir-network`, `fenrir-secure` |

### Stubs fertigstellen
| Crate | Aufgabe |
|---|---|
| `fenrir-i18n` | Fluent-/TOML-Translations laden |
| `fenrir-ui` | Workspace-State-Typen, Tab-Gruppen-Datenmodell (kein Rendering) |

### Features
- Bookmark-Manager mit AI-Auto-Tagging
- Tracker-Blocking-Engine
- `fenrir-crypto` als Fundament für alle sicheren Features in Phase 4+

### Hinweis
`fenrir-crypto` ist **kritischer Pfad** — ohne ihn kann Phase 4 nicht starten.

---

## Phase 4 — Mid-term Platform 📋

**Ziel:** Extension-Runtime, vollständige AI-Workspace-UI, Passwort-Manager.

**Voraussetzung:** `fenrir-crypto` aus Phase 3

### Neue Crates
| Crate | Was es tut | Abhängigkeiten |
|---|---|---|
| `fenrir-passwords` | Passwort-Manager, Argon2id-KDF, keyring-rs, Autofill | `fenrir-crypto` |
| `fenrir-sandbox` | WASM-Runtime (wasmtime), IPC, Capability-System | `fenrir-secure`, `fenrir-api` |

### Crates vervollständigen
| Crate | Aufgabe | Abhängigkeiten |
|---|---|---|
| `fenrir-api` | Traits → echte Impl, HTTP/IPC-Server für externe Tools | `fenrir-core`, `fenrir-crypto` |
| `fenrir-ui` | Workspace-Rendering (egui), AI-Command-Palette, Tab-Gruppen-UI | `fenrir-ai`, `fenrir-bookmarks` |

### Features
- Extension-Runtime mit WASM-Sandbox
- Vollständige AI-Workspace-UI (Tab-Gruppen, Semantic Search, AI-Panel)
- Passwort-Manager mit Autofill
- Privacy-Dashboard (Tracker-Statistiken, Network-Audit)
- Multi-Process-Vorbereitung: IPC-Abstraktion für Renderer-Isolation

### Docs: `docs/active/` → `docs/current/` für extension-architecture, browser-process-model

---

## Phase 5 — Long-term Vision 🔮

**Ziel:** Souveräner EU-Browser mit Web3, Sync, VPN, Identity.

**Voraussetzung:** Phase 4 vollständig, insbesondere `fenrir-crypto` + `fenrir-api`

### Neue Crates
| Crate | Was es tut |
|---|---|
| `fenrir-wallet` | Web3/Ethereum, MetaMask-kompatibler EIP-1193-Provider |
| `fenrir-web3` | DApp-Browser-Bridge, JSON-RPC, fenrir-wallet-Integration |
| `fenrir-sync` | P2P-Sync via iroh, E2E-verschlüsselt (fenrir-crypto), kein Cloud-Fallback |
| `fenrir-vpn` | WireGuard-Integration (boringtun), Traffic-Routing |
| `fenrir-identity` | Self-Sovereign Identity (SSI), DID, Verifiable Credentials |
| `fenrir-mail` | E2E-verschlüsselte Mail-Integration |
| `fenrir-zerotrust` | Zero-Trust-Network-Access, Zertifikat-basierte Device-Auth |
| `fenrir-compute` | Lokaler AI-Compute-Marketplace |
| `fenrir-update` | Sicheres Auto-Update mit Signatur-Verifikation |

### Docs: `docs/vision/`

---

## Abhängigkeitsgraph (kritischer Pfad)

```
fenrir-crypto (Phase 3)
    ├── fenrir-passwords (Phase 4)
    ├── fenrir-api (Phase 4)
    ├── fenrir-wallet (Phase 5)
    ├── fenrir-sync (Phase 5)
    └── fenrir-identity (Phase 5)

fenrir-ai (Phase 2)
    ├── fenrir-bookmarks (Phase 3)
    ├── fenrir-ui vollständig (Phase 4)
    └── fenrir-compute (Phase 5)

fenrir-sandbox (Phase 4)
    └── Extension-Ökosystem (Phase 4+)
```
