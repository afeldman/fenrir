//! LlmProvider Trait — gemeinsames Interface für alle LLM-Backends.
//!
//! Jeder Provider (lokal, mammoth.ai, Ollama, ...) implementiert diesen Trait.
//! Der Router wählt dann den besten verfügbaren Provider aus.

use async_trait::async_trait;
use fenrir_core::error::FenrirError;
use serde::{Deserialize, Serialize};

/// Welche Art von Provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    /// Lokal auf dem Gerät (Privacy-first, kein Cloud-Zwang).
    Local,
    /// mammoth.ai Cloud API.
    Mammoth,
    /// Ollama lokal (HTTP).
    Ollama,
    /// Beliebiger anderer MCP-kompatibler Provider.
    Custom(String),
}

/// Metadaten eines Providers.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub kind: ProviderKind,
    pub name: &'static str,
    pub supports_streaming: bool,
}

/// Ein einzelner LLM-Request (vereinfachtes Interface über MCP sampling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Systemnachricht (Kontext/Persona für das Modell).
    pub system: Option<String>,
    /// Gesprächsverlauf. Letzter Eintrag ist der aktuelle User-Turn.
    pub messages: Vec<ChatMessage>,
    /// Maximale Anzahl Tokens in der Antwort.
    pub max_tokens: Option<u32>,
    /// Temperature (0.0 = deterministisch, 1.0 = kreativ).
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
}

/// Antwort eines LLM-Providers.
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Generierter Text.
    pub content: String,
    /// Provider der geantwortet hat.
    pub provider: ProviderKind,
    /// Token-Statistik (falls verfügbar).
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Trait den jeder LLM-Provider implementieren muss.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Metadaten dieses Providers.
    fn info(&self) -> ProviderInfo;

    /// Prüft ob der Provider aktuell erreichbar/bereit ist.
    async fn is_available(&self) -> bool;

    /// Führt einen LLM-Request durch und gibt die Antwort zurück.
    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, FenrirError>;
}
