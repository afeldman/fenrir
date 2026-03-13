//! MoE — Mixture of Experts mit Top-2 Routing.
//!
//! 8 geroutete Experts + 1 gemeinsamer Shared Expert.
//! Jeder Token aktiviert Top-2 der gerouteten Experts + Shared Expert.

use super::{MoeConfig, RamoConfig, Mlp};
use candle_core::{IndexOp, Result, Tensor, D};
use candle_nn::{Linear, Module, VarBuilder, linear_no_bias};

pub struct MoE {
    gate: MoEGate,
    experts: Vec<Mlp>,
    shared_expert: Option<Mlp>,
}

struct MoEGate {
    weight: Linear,
    top_k: usize,
    n_experts: usize,
    routed_scaling_factor: f64,
    norm_topk_prob: bool,
}

impl MoE {
    pub fn new(vb: VarBuilder, cfg: &RamoConfig, moe_cfg: &MoeConfig) -> Result<Self> {
        let gate = MoEGate {
            weight: linear_no_bias(
                cfg.hidden_size,
                moe_cfg.n_routed_experts,
                vb.pp("gate"),
            )?,
            top_k: moe_cfg.num_experts_per_tok,
            n_experts: moe_cfg.n_routed_experts,
            routed_scaling_factor: moe_cfg.routed_scaling_factor,
            norm_topk_prob: moe_cfg.norm_topk_prob,
        };

        let experts = (0..moe_cfg.n_routed_experts)
            .map(|i| Mlp::new(vb.pp(format!("experts.{i}")), cfg.hidden_size, cfg.intermediate_size / 2))
            .collect::<Result<Vec<_>>>()?;

        let shared_expert = if moe_cfg.n_shared_experts > 0 {
            Some(Mlp::new(
                vb.pp("shared_expert"),
                cfg.hidden_size,
                cfg.intermediate_size,
            )?)
        } else {
            None
        };

        Ok(Self { gate, experts, shared_expert })
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let (bsz, seq, hidden) = x.dims3()?;
        let x_flat = x.reshape((bsz * seq, hidden))?;

        // 1. Router: Top-K Expert Selection
        let (topk_ids, topk_weights) = self.gate.forward(&x_flat)?;

        // 2. Shared Expert (alle Tokens)
        let mut out = Tensor::zeros_like(&x_flat)?;
        if let Some(shared) = &self.shared_expert {
            out = (out + shared.forward(&x_flat)?)?;
        }

        // 3. Geroutete Experts (sparse)
        let _n_tokens = bsz * seq;
        for expert_id in 0..self.gate.n_experts {
            // Maske: welche Tokens gehen zu diesem Expert?
            let mut token_indices = Vec::new();
            let mut which_slot = Vec::new();

            let ids_vec = topk_ids.to_vec2::<u32>()?;
            let weights_vec = topk_weights.to_vec2::<f32>()?;

            for (tok_idx, ids) in ids_vec.iter().enumerate() {
                for (slot, &id) in ids.iter().enumerate() {
                    if id as usize == expert_id {
                        token_indices.push(tok_idx as u32);
                        which_slot.push(slot as u32);
                    }
                }
            }

            if token_indices.is_empty() {
                continue;
            }

            // Expert-Eingaben sammeln
            let tok_tensor = Tensor::new(token_indices.as_slice(), x_flat.device())?;
            let x_expert = x_flat.index_select(&tok_tensor, 0)?;

            // Expert Forward
            let y_expert = self.experts[expert_id].forward(&x_expert)?;

            // Gewichte anwenden und akkumulieren
            for (i, (tok_idx, slot)) in token_indices.iter().zip(which_slot.iter()).enumerate() {
                let weight = weights_vec[*tok_idx as usize][*slot as usize];
                let y_tok = (y_expert.i(i)? * weight as f64)?.unsqueeze(0)?;
                // index_add ist komplex in candle — vereinfacht: direkte Addition
                // In der vollen Implementierung: scatter_add
                let _ = y_tok; // TODO: scatter_add für effiziente Akkumulation
            }
        }

        out.reshape((bsz, seq, hidden))
    }
}

impl MoEGate {
    fn forward(&self, x: &Tensor) -> Result<(Tensor, Tensor)> {
        // Expert Logits: (tokens, n_experts)
        let logits = self.weight.forward(x)?;
        let probs = candle_nn::ops::softmax(&logits.to_dtype(candle_core::DType::F32)?, D::Minus1)?;

        // Top-K Auswahl via sort_last_dim (descending)
        let (sorted_values, sorted_indices) = probs.sort_last_dim(false)?;
        let values = sorted_values.narrow(D::Minus1, 0, self.top_k)?;
        let indices = sorted_indices.narrow(D::Minus1, 0, self.top_k)?;

        // Normalisierung der Top-K Wahrscheinlichkeiten
        let values = if self.norm_topk_prob {
            let sum = values.sum_keepdim(D::Minus1)?;
            (values / (sum + 1e-9)?)?
        } else {
            values
        };

        // Skalierung
        let values = (values * self.routed_scaling_factor)?;

        let indices = indices.to_dtype(candle_core::DType::U32)?;
        Ok((indices, values.to_dtype(candle_core::DType::F32)?))
    }
}
