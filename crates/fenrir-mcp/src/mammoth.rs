//! MammothProvider — mammoth.ai Cloud LLM Integration.
//!
//! Wird nur genutzt wenn lokal kein Modell verfügbar (Fallback).
//! Alle Requests laufen durch fenrir-network (Phase 2).

use crate::provider::{
    LlmProvider, LlmRequest, LlmResponse, ProviderInfo, ProviderKind, Role,
    TokenUsage,
};
use async_trait::async_trait;
use fenrir_core::error::FenrirError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

pub struct MammothProvider {
    api_key: String,
    base_url: String,
    model: String,
    client: Client,
}

impl MammothProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.mammoth.ai/v1".to_string(),
            model: "mammoth-1".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }
}

// API-Datentypen für mammoth.ai (OpenAI-kompatibles Format)
#[derive(Serialize)]
struct MammothRequest {
    model: String,
    messages: Vec<MammothMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize)]
struct MammothMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MammothResponse {
    choices: Vec<MammothChoice>,
    usage: Option<MammothUsage>,
}

#[derive(Deserialize)]
struct MammothChoice {
    message: MammothMessage,
}

#[derive(Deserialize)]
struct MammothUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

fn role_to_str(role: &Role) -> &'static str {
    match role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
    }
}

#[async_trait]
impl LlmProvider for MammothProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            kind: ProviderKind::Mammoth,
            name: "mammoth-ai",
            supports_streaming: false, // Phase 2: SSE streaming
        }
    }

    async fn is_available(&self) -> bool {
        if self.api_key.is_empty() {
            return false;
        }
        // Phase 2: Health-Check Endpoint aufrufen
        // Jetzt: API-Key vorhanden = verfügbar annehmen
        true
    }

    async fn complete(&self, request: LlmRequest) -> Result<LlmResponse, FenrirError> {
        debug!(model = self.model, "mammoth.ai request");

        let mut messages: Vec<MammothMessage> = Vec::new();

        if let Some(system) = &request.system {
            messages.push(MammothMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        for msg in &request.messages {
            messages.push(MammothMessage {
                role: role_to_str(&msg.role).to_string(),
                content: msg.content.clone(),
            });
        }

        let body = MammothRequest {
            model: self.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| FenrirError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(FenrirError::Network(
                format!("mammoth.ai API Fehler {}: {}", status, text)
            ));
        }

        let data: MammothResponse = response
            .json()
            .await
            .map_err(|e| FenrirError::Network(format!("JSON Parse Fehler: {}", e)))?;

        let content = data
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        info!(chars = content.len(), "mammoth.ai response erhalten");

        Ok(LlmResponse {
            content,
            provider: ProviderKind::Mammoth,
            usage: data.usage.map(|u| TokenUsage {
                input_tokens: u.prompt_tokens,
                output_tokens: u.completion_tokens,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mammoth_not_available_without_api_key() {
        let provider = MammothProvider::new("");
        assert!(!provider.is_available().await);
    }

    #[tokio::test]
    async fn mammoth_available_with_api_key() {
        let provider = MammothProvider::new("test-key-123");
        assert!(provider.is_available().await);
    }

    #[tokio::test]
    async fn mammoth_provider_info() {
        let provider = MammothProvider::new("key");
        assert_eq!(provider.info().kind, ProviderKind::Mammoth);
    }
}
