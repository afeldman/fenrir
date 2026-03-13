//! Inference Engine — Text-Generierung mit noeum-1-nano.

use crate::{model::RamoModel, tokenizer::NoeumTokenizer};
use fenrir_core::error::FenrirError;
use candle_core::{Device, Tensor};
use std::path::PathBuf;
use tracing::info;

/// Inference-Request an das lokale Modell.
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// Eingabe-Prompt.
    pub prompt: String,
    /// Maximale Ausgabe-Tokens.
    pub max_tokens: Option<u32>,
    /// Thinking Mode aktivieren.
    pub thinking: Option<()>,
}

/// Inference-Antwort vom Modell.
#[derive(Debug, Clone)]
pub struct InferenceResponse {
    /// Generierter Text.
    pub text: String,
    /// Anzahl generierter Tokens.
    pub tokens_generated: usize,
    /// Thinking-Block falls Thinking Mode aktiv war.
    pub thinking_content: Option<String>,
}

/// Interne Inference Engine.
pub struct InferenceEngine {
    model: RamoModel,
    tokenizer: NoeumTokenizer,
    device: Device,
}

impl InferenceEngine {
    pub async fn load(dir: PathBuf) -> Result<Self, FenrirError> {
        let device = Device::Cpu; // GPU-Support später
        
        info!("Lade Tokenizer...");
        let tokenizer = NoeumTokenizer::load(&dir)?;
        
        info!("Lade Modell-Weights...");
        let model = RamoModel::load(&dir, &device)?;
        
        info!("Weights geladen ✓");
        Ok(Self { model, tokenizer, device })
    }

    pub async fn complete(&mut self, req: InferenceRequest) -> Result<InferenceResponse, FenrirError> {
        let prompt = if req.thinking.is_some() {
            format!("<think>\n{}", req.prompt)
        } else {
            req.prompt.clone()
        };

        let input_ids = self.tokenizer.encode(&prompt, true)?;
        let max_new = req.max_tokens.unwrap_or(256);
        let eos = self.tokenizer.eos_id();

        // Konvertiere input_ids zu Tensor für generate Methode
        let input_tensor = Tensor::new(input_ids.as_slice(), &self.device)?
            .unsqueeze(0)?;
        
        // Verwende die generate Methode des Modells
        let generated_ids = self.model.generate(
            &input_tensor,
            max_new,
            0.1, // temperature
            eos,
        )?;

        let new_tokens = &generated_ids[input_ids.len()..];
        let text = self.tokenizer.decode(new_tokens)?;

        // Thinking-Block extrahieren falls vorhanden
        let thinking_content = if req.thinking.is_some() {
            extract_thinking(&text)
        } else {
            None
        };

        Ok(InferenceResponse {
            text: text.clone(),
            tokens_generated: new_tokens.len(),
            thinking_content,
        })
    }
}

/// Extrahiert Thinking-Block aus Text.
fn extract_thinking(text: &str) -> Option<String> {
    if let (Some(start), Some(end)) = (text.find("<think>"), text.find("</think>")) {
        if start < end {
            return Some(text[start + 7..end].trim().to_string());
        }
    }
    None
}
