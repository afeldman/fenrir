//! Inference Engine — Text-Generierung mit KV-Cache und Thinking Mode.
//!
//! Thinking Mode: Prompt wird mit `/think` Prefix versehen, Modell
//! generiert erst internen Reasoning-Block, dann finale Antwort.

use crate::{model::RamoModel, tokenizer::NoeumTokenizer};
use fenrir_core::error::FenrirError;
use candle_core::{Device, Tensor};
use std::path::PathBuf;
use tracing::{debug, info};

/// Konfiguration für Noeum's Thinking Mode (System-2 Reasoning).
#[derive(Debug, Clone)]
pub struct ThinkingConfig {
    /// Maximale Thinking-Budget in Tokens (empfohlen: 128).
    pub budget_tokens: u32,
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        Self { budget_tokens: 128 }
    }
}

/// Inference-Request an das lokale Modell.
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// Eingabe-Prompt.
    pub prompt: String,
    /// System-Prompt (optional).
    pub system: Option<String>,
    /// Thinking Mode aktivieren (empfohlen für Reasoning-Tasks).
    pub thinking: Option<ThinkingConfig>,
    /// Maximale Ausgabe-Tokens.
    pub max_new_tokens: u32,
    /// Temperature (0.0 = deterministisch, 1.0 = kreativ).
    pub temperature: f32,
}

impl Default for InferenceRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            system: None,
            thinking: None,
            max_new_tokens: 512,
            temperature: 0.1, // Noeum-Empfehlung für Reasoning
        }
    }
}

/// Inference-Antwort vom Modell.
#[derive(Debug, Clone)]
pub struct InferenceResponse {
    /// Generierter Text (ohne Thinking-Block).
    pub text: String,
    /// Thinking-Block falls Thinking Mode aktiv war.
    pub thinking_text: Option<String>,
    /// Anzahl generierter Tokens.
    pub tokens_generated: u32,
}

/// Interne Inference Engine.
pub struct InferenceEngine {
    model: RamoModel,
    tokenizer: NoeumTokenizer,
    device: Device,
}

impl InferenceEngine {
    pub async fn load(model_dir: PathBuf) -> Result<Self, FenrirError> {
        let device = Device::Cpu; // Phase 2: Metal/CUDA Detection

        let tokenizer = NoeumTokenizer::load(&model_dir)?;
        info!("Tokenizer geladen (vocab_size={})", tokenizer.vocab_size());

        let model = RamoModel::load(&model_dir, &device)?;
        info!("Ramo Modell geladen auf {:?}", device);

        Ok(Self { model, tokenizer, device })
    }

    pub async fn complete(&mut self, request: InferenceRequest) -> Result<InferenceResponse, FenrirError> {
        // Prompt zusammenbauen gemäß Noeum's Chat-Format
        let formatted = self.format_prompt(&request);
        debug!(prompt_len = formatted.len(), "Inference starten");

        // Tokenisieren
        let input_ids = self.tokenizer.encode(&formatted)?;
        let input_tensor = Tensor::new(input_ids.as_slice(), &self.device)
            .map_err(|e| FenrirError::Crypto(format!("Tensor error: {e}")))?
            .unsqueeze(0)
            .map_err(|e| FenrirError::Crypto(format!("Tensor error: {e}")))?;

        // Generierung (greedy decoding)
        let output_ids = self.model.generate(
            &input_tensor,
            request.max_new_tokens,
            request.temperature,
            self.tokenizer.eos_token_id(),
        )?;

        // Dekodieren
        let output_text = self.tokenizer.decode(&output_ids)?;

        // Thinking-Block separieren falls vorhanden
        let (thinking_text, final_text) = split_thinking(&output_text);

        Ok(InferenceResponse {
            text: final_text,
            thinking_text,
            tokens_generated: output_ids.len() as u32,
        })
    }

    /// Prompt gemäß Noeum's Chat-Template formatieren.
    fn format_prompt(&self, request: &InferenceRequest) -> String {
        let mut parts = Vec::new();

        if let Some(system) = &request.system {
            parts.push(format!("<|system|>\n{system}<|endoftext|>"));
        }

        // Thinking Mode: /think Befehl voranstellen
        let user_content = if request.thinking.is_some() {
            let budget = request.thinking.as_ref().unwrap().budget_tokens;
            format!("/think\n/budget {budget}\n{}", request.prompt)
        } else {
            format!("/no think\n{}", request.prompt)
        };

        parts.push(format!("<|user|>\n{user_content}<|endoftext|>"));
        parts.push("<|assistant|>\n".to_string());

        parts.join("\n")
    }
}

/// Separiert Thinking-Block `<think>...</think>` von der Antwort.
fn split_thinking(text: &str) -> (Option<String>, String) {
    if let (Some(start), Some(end)) = (text.find("<think>"), text.find("</think>")) {
        if start < end {
            let thinking = text[start + 7..end].trim().to_string();
            let after = text[end + 8..].trim().to_string();
            return (Some(thinking), after);
        }
    }
    (None, text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_thinking_with_block() {
        let text = "<think>Lass mich nachdenken...</think>Die Antwort ist 42.";
        let (think, answer) = split_thinking(text);
        assert_eq!(think, Some("Lass mich nachdenken...".to_string()));
        assert_eq!(answer, "Die Antwort ist 42.");
    }

    #[test]
    fn split_thinking_without_block() {
        let text = "Direkte Antwort ohne Thinking.";
        let (think, answer) = split_thinking(text);
        assert!(think.is_none());
        assert_eq!(answer, "Direkte Antwort ohne Thinking.");
    }

    #[test]
    fn thinking_config_default_budget() {
        let config = ThinkingConfig::default();
        assert_eq!(config.budget_tokens, 128);
    }
}
