//! fenrir-mcp — MCP Host + LLM Router für Fenrir Browser.
//!
//! # Architektur
//!
//! Fenrir fungiert als MCP Host (Client) der sich zu mehreren MCP Servern verbindet:
//!
//! ```text
//! Fenrir (MCP Host)
//!     ├── LocalLlmServer   (stdio, Phi-3 Mini via candle — Phase 3)
//!     ├── MammothProvider  (HTTP/SSE, mammoth.ai)
//!     └── weitere Provider (Ollama, LM Studio, etc.)
//! ```
//!
//! Der `LlmRouter` wählt automatisch den besten verfügbaren Provider:
//! - Lokal bevorzugt (Privacy, kein Cloud-Zwang)
//! - Cloud als Fallback wenn lokal nicht verfügbar
//! - User kann Provider-Priorität konfigurieren
//!
//! # MCP Sampling
//!
//! LLM-Aufrufe nutzen `sampling/createMessage` (MCP Standard):
//! - Request: [`LlmRequest`] → wird zu `CreateMessageRequest`
//! - Response: [`LlmResponse`] ← kommt von `CreateMessageResult`

pub mod host;
pub mod provider;
pub mod router;

// Provider-Implementierungen
pub mod local;
pub mod mammoth;

pub use host::FenrirMcpHost;
pub use provider::{LlmProvider, LlmRequest, LlmResponse, ProviderInfo, ProviderKind};
pub use router::LlmRouter;
