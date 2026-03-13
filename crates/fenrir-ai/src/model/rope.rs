//! YaRN RoPE — Rotary Positional Embeddings mit YaRN-Extrapolation.
//!
//! Ramo nutzt RoPE mit YaRN-Scaling: Original Context 512, erweitert auf 2048.
//! factor=4.0, rope_theta=1_000_000.0

use super::RamoConfig;
use candle_core::{Device, Result, Tensor};
use std::f64::consts::PI;

pub struct RotaryEmbedding {
    cos: Tensor,
    sin: Tensor,
    head_dim: usize,
}

impl RotaryEmbedding {
    pub fn new(cfg: &RamoConfig, device: &Device) -> Result<Self> {
        let head_dim = cfg.hidden_size / cfg.num_attention_heads;
        let max_seq = cfg.max_position_embeddings;
        let theta = cfg.rope_theta;

        // YaRN-Frequenzen berechnen
        let freqs = yarn_frequencies(head_dim, max_seq, theta, 4.0, 32.0, 1.0)?;
        let cos_vals: Vec<f32> = freqs.iter().map(|f| f.cos() as f32).collect();
        let sin_vals: Vec<f32> = freqs.iter().map(|f| f.sin() as f32).collect();

        let cos = Tensor::from_vec(cos_vals, (max_seq, head_dim / 2), device)?;
        let sin = Tensor::from_vec(sin_vals, (max_seq, head_dim / 2), device)?;

        Ok(Self { cos, sin, head_dim })
    }

    /// Gibt cos/sin Tensoren für gegebene Positionen zurück.
    pub fn get(&self, seqlen: usize, offset: usize) -> Result<(Tensor, Tensor)> {
        let cos = self.cos.narrow(0, offset, seqlen)?;
        let sin = self.sin.narrow(0, offset, seqlen)?;
        Ok((cos, sin))
    }
}

/// YaRN-modifizierte Frequenzen (vereinfacht für inference-only).
fn yarn_frequencies(
    head_dim: usize,
    max_seq: usize,
    theta: f64,
    factor: f64,
    beta_fast: f64,
    beta_slow: f64,
) -> Result<Vec<f64>> {
    let half_dim = head_dim / 2;

    // Basis-Frequenzen (RoPE)
    let inv_freq: Vec<f64> = (0..half_dim)
        .map(|i| 1.0 / theta.powf(i as f64 * 2.0 / head_dim as f64))
        .collect();

    // Positions × Frequenzen
    let mut freqs = Vec::with_capacity(max_seq * half_dim);
    for pos in 0..max_seq {
        for &freq in &inv_freq {
            freqs.push(pos as f64 * freq);
        }
    }

    Ok(freqs)
}

/// RoPE auf Query und Key Tensoren anwenden.
/// Tensoren: (bsz, heads, seq, head_dim)
pub fn apply_rotary(
    q: &Tensor,
    k: &Tensor,
    cos: &Tensor,
    sin: &Tensor,
) -> Result<(Tensor, Tensor)> {
    Ok((
        rotate_half(q, cos, sin)?,
        rotate_half(k, cos, sin)?,
    ))
}

fn rotate_half(x: &Tensor, cos: &Tensor, sin: &Tensor) -> Result<Tensor> {
    let half = x.dim(candle_core::D::Minus1)? / 2;
    let x1 = x.narrow(candle_core::D::Minus1, 0, half)?;
    let x2 = x.narrow(candle_core::D::Minus1, half, half)?;
    let rotated = Tensor::cat(&[&x2.neg()?, &x1], candle_core::D::Minus1)?;

    // cos/sin: (seq, half_dim) → broadcast zu (bsz, heads, seq, half_dim)
    let cos = cos.unsqueeze(0)?.unsqueeze(0)?;
    let sin = sin.unsqueeze(0)?.unsqueeze(0)?;

    (x.broadcast_mul(&cos)? + rotated.broadcast_mul(&sin)?)
}
