//! Ramo Modell-Architektur (noeum-1-nano) in Rust/Candle.
//!
//! Implementiert: GQA Attention + QK-Norm + YaRN-RoPE + MoE (8 Experts + 1 Shared)

mod attention;
mod moe;
mod rope;

use attention::Attention;
use fenrir_core::error::FenrirError;
use moe::MoE;
use rope::RotaryEmbedding;

use candle_core::{DType, Device, IndexOp, Tensor, D};
use candle_nn::{Embedding, Linear, Module, VarBuilder, embedding, linear_no_bias};
use serde::Deserialize;
use std::path::Path;

// ── Konfiguration ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RamoConfig {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub intermediate_size: usize,
    pub max_position_embeddings: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f64,
    pub use_qk_norm: bool,
    pub tie_word_embeddings: bool,
    pub moe_config: Option<MoeConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoeConfig {
    pub n_routed_experts: usize,
    pub num_experts_per_tok: usize,
    pub n_shared_experts: usize,
    pub n_dense_layer: usize,
    pub routed_scaling_factor: f64,
    pub norm_topk_prob: bool,
}

// ── MLP (Dense Feed-Forward) ────────────────────────────────────────────────

struct Mlp {
    gate_proj: Linear,
    up_proj: Linear,
    down_proj: Linear,
}

impl Mlp {
    fn new(vb: VarBuilder, hidden: usize, intermediate: usize) -> candle_core::Result<Self> {
        Ok(Self {
            gate_proj: linear_no_bias(hidden, intermediate, vb.pp("gate_proj"))?,
            up_proj: linear_no_bias(hidden, intermediate, vb.pp("up_proj"))?,
            down_proj: linear_no_bias(intermediate, hidden, vb.pp("down_proj"))?,
        })
    }

    fn forward(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        // SwiGLU: down(silu(gate) * up)
        let gate = self.gate_proj.forward(x)?.silu()?;
        let up = self.up_proj.forward(x)?;
        self.down_proj.forward(&(gate * up)?)
    }
}

// ── RMSNorm ─────────────────────────────────────────────────────────────────

struct RmsNorm {
    weight: Tensor,
    eps: f64,
}

impl RmsNorm {
    fn new(vb: VarBuilder, size: usize, eps: f64) -> candle_core::Result<Self> {
        let weight = vb.get(size, "weight")?;
        Ok(Self { weight, eps })
    }

    fn forward(&self, x: &Tensor) -> candle_core::Result<Tensor> {
        let x_dtype = x.dtype();
        let x = x.to_dtype(DType::F32)?;
        let variance = x.sqr()?.mean_keepdim(D::Minus1)?;
        let x_norm = x.broadcast_div(&(variance + self.eps)?.sqrt()?)?;
        x_norm.to_dtype(x_dtype)?.broadcast_mul(&self.weight)
    }
}

// ── Decoder Layer ────────────────────────────────────────────────────────────

struct DecoderLayer {
    attn_norm: RmsNorm,
    attn: Attention,
    mlp_norm: RmsNorm,
    mlp: MlpOrMoe,
}

enum MlpOrMoe {
    Dense(Mlp),
    Moe(MoE),
}

impl DecoderLayer {
    fn new(
        vb: VarBuilder,
        cfg: &RamoConfig,
        layer_idx: usize,
        rotary_emb: &RotaryEmbedding,
    ) -> candle_core::Result<Self> {
        let attn_norm = RmsNorm::new(vb.pp("attn_norm"), cfg.hidden_size, cfg.rms_norm_eps)?;
        let attn = Attention::new(vb.pp("attn"), cfg, rotary_emb)?;
        let mlp_norm = RmsNorm::new(vb.pp("mlp_norm"), cfg.hidden_size, cfg.rms_norm_eps)?;

        let mlp = match &cfg.moe_config {
            Some(moe_cfg) if layer_idx >= moe_cfg.n_dense_layer => {
                MlpOrMoe::Moe(MoE::new(vb.pp("mlp"), cfg, moe_cfg)?)
            }
            _ => MlpOrMoe::Dense(Mlp::new(vb.pp("mlp"), cfg.hidden_size, cfg.intermediate_size)?),
        };

        Ok(Self { attn_norm, attn, mlp_norm, mlp })
    }

    fn forward(
        &mut self,
        x: &Tensor,
        seqlen_offset: usize,
    ) -> candle_core::Result<Tensor> {
        // 1. Attention Block (Pre-Norm + Residual)
        let residual = x;
        let h = self.attn.forward(&self.attn_norm.forward(x)?, seqlen_offset)?;
        let h = (h + residual)?;

        // 2. MLP/MoE Block (Pre-Norm + Residual)
        let residual = &h;
        let normed = self.mlp_norm.forward(&h)?;
        let mlp_out = match &self.mlp {
            MlpOrMoe::Dense(mlp) => mlp.forward(&normed)?,
            MlpOrMoe::Moe(moe) => moe.forward(&normed)?,
        };
        mlp_out + residual
    }
}

// ── RamoModel (Top-Level) ────────────────────────────────────────────────────

pub struct RamoModel {
    embed_tokens: Embedding,
    layers: Vec<DecoderLayer>,
    head_norm: RmsNorm,
    lm_head: Linear,
}

impl RamoModel {
    pub fn load(model_dir: &Path, device: &Device) -> Result<Self, FenrirError> {
        // Config laden
        let config_str = std::fs::read_to_string(model_dir.join("config.json"))
            .map_err(|e| FenrirError::Io(e))?;
        let cfg: RamoConfig = serde_json::from_str(&config_str)
            .map_err(|e| FenrirError::Config(format!("Config parse: {e}")))?;

        // Gewichte laden (PyTorch .bin Format)
        let weights_path = model_dir.join("pytorch_model.bin");
        let vb = VarBuilder::from_pth(&weights_path, DType::F32, device)
            .map_err(|e| FenrirError::Config(format!("Weights laden: {e}")))?;

        let model = Self::new(vb, &cfg)
            .map_err(|e| FenrirError::Config(format!("Modell init: {e}")))?;

        Ok(model)
    }

    fn new(vb: VarBuilder, cfg: &RamoConfig) -> candle_core::Result<Self> {
        let rotary_emb = RotaryEmbedding::new(cfg, vb.device())?;

        let embed_tokens = embedding(cfg.vocab_size, cfg.hidden_size, vb.pp("model.embed_tokens"))?;

        let layers = (0..cfg.num_hidden_layers)
            .map(|i| DecoderLayer::new(vb.pp(format!("model.layers.{i}")), cfg, i, &rotary_emb))
            .collect::<candle_core::Result<Vec<_>>>()?;

        let head_norm = RmsNorm::new(vb.pp("model.head_norm"), cfg.hidden_size, cfg.rms_norm_eps)?;

        let lm_head = if cfg.tie_word_embeddings {
            Linear::new(embed_tokens.embeddings().clone(), None)
        } else {
            linear_no_bias(cfg.hidden_size, cfg.vocab_size, vb.pp("lm_head"))?
        };

        Ok(Self {
            embed_tokens,
            layers,
            head_norm,
            lm_head,
        })
    }

    /// Autoregressive Generierung mit Greedy Decoding.
    pub fn generate(
        &mut self,
        input_ids: &Tensor,
        max_new_tokens: u32,
        temperature: f32,
        eos_token_id: u32,
    ) -> Result<Vec<u32>, FenrirError> {
        let mut generated: Vec<u32> = Vec::new();
        let mut seqlen_offset = 0;
        let mut current_input = input_ids.clone();

        for _ in 0..max_new_tokens {
            let logits = self
                .forward(&current_input, seqlen_offset)
                .map_err(|e| FenrirError::Config(format!("Forward pass: {e}")))?;

            // Letzten Token-Logit nehmen
            let ai_err = |e: candle_core::Error| FenrirError::Ai(e.to_string());
            let seq_len = logits.dim(1).map_err(ai_err)?;
            let next_token_logits = logits
                .i((.., seq_len - 1, ..))
                .map_err(|e| FenrirError::Ai(e.to_string()))?
                .squeeze(1)
                .map_err(|e| FenrirError::Ai(e.to_string()))?;

            // Temperature anwenden + Greedy
            let next_token = if temperature < 1e-6 {
                next_token_logits.argmax(D::Minus1)
                    .map_err(|e| FenrirError::Ai(e.to_string()))?
            } else {
                let scaled = (next_token_logits / temperature as f64)
                    .map_err(|e| FenrirError::Ai(e.to_string()))?;
                let probs = candle_nn::ops::softmax(&scaled, D::Minus1)
                    .map_err(|e| FenrirError::Ai(e.to_string()))?;
                probs.argmax(D::Minus1)
                    .map_err(|e| FenrirError::Ai(e.to_string()))?
            };

            let token_id: u32 = next_token.to_scalar()
                .map_err(|e| FenrirError::Ai(e.to_string()))?;
            if token_id == eos_token_id {
                break;
            }

            generated.push(token_id);
            seqlen_offset += current_input.dim(1)
                .map_err(|e| FenrirError::Ai(e.to_string()))?;
            current_input = Tensor::new(&[token_id], current_input.device())
                .and_then(|t| t.unsqueeze(0))
                .map_err(|e| FenrirError::Ai(e.to_string()))?;
        }

        Ok(generated)
    }

    fn forward(&mut self, input_ids: &Tensor, seqlen_offset: usize) -> candle_core::Result<Tensor> {
        let mut hidden = self.embed_tokens.forward(input_ids)?;

        for layer in &mut self.layers {
            hidden = layer.forward(&hidden, seqlen_offset)?;
        }

        let hidden = self.head_norm.forward(&hidden)?;
        self.lm_head.forward(&hidden)
    }
}
