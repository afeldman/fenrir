//! GQA Attention mit QK-Norm (RMSNorm per Head).

use super::{RamoConfig, RmsNorm, rope::{RotaryEmbedding, apply_rotary}};
use candle_core::{Result, Tensor, D};
use candle_nn::{Linear, Module, VarBuilder, linear_no_bias};

pub struct Attention {
    q_proj: Linear,
    k_proj: Linear,
    v_proj: Linear,
    o_proj: Linear,
    q_norm: Option<RmsNorm>,
    k_norm: Option<RmsNorm>,
    num_heads: usize,
    num_kv_heads: usize,
    num_kv_groups: usize,
    head_dim: usize,
    scale: f64,
    rotary_emb: RotaryEmbedding,
    // KV Cache: (key_cache, value_cache) pro Layer
    kv_cache: Option<(Tensor, Tensor)>,
}

impl Attention {
    pub fn new(vb: VarBuilder, cfg: &RamoConfig, _rotary_emb: &RotaryEmbedding) -> Result<Self> {
        let hidden = cfg.hidden_size;
        let num_heads = cfg.num_attention_heads;
        let num_kv_heads = cfg.num_key_value_heads;
        let head_dim = hidden / num_heads;

        let q_proj = linear_no_bias(hidden, num_heads * head_dim, vb.pp("q_proj"))?;
        let k_proj = linear_no_bias(hidden, num_kv_heads * head_dim, vb.pp("k_proj"))?;
        let v_proj = linear_no_bias(hidden, num_kv_heads * head_dim, vb.pp("v_proj"))?;
        let o_proj = linear_no_bias(num_heads * head_dim, hidden, vb.pp("o_proj"))?;

        let (q_norm, k_norm) = if cfg.use_qk_norm {
            let q_norm = RmsNorm::new(vb.pp("q_norm"), head_dim, cfg.rms_norm_eps)?;
            let k_norm = RmsNorm::new(vb.pp("k_norm"), head_dim, cfg.rms_norm_eps)?;
            (Some(q_norm), Some(k_norm))
        } else {
            (None, None)
        };

        // RoPE Kopie für diesen Attention-Layer
        let rotary_emb = RotaryEmbedding::new(cfg, vb.device())?;

        Ok(Self {
            q_proj, k_proj, v_proj, o_proj,
            q_norm, k_norm,
            num_heads,
            num_kv_heads,
            num_kv_groups: num_heads / num_kv_heads,
            head_dim,
            scale: (head_dim as f64).sqrt().recip(),
            rotary_emb,
            kv_cache: None,
        })
    }

    pub fn forward(&mut self, hidden: &Tensor, seqlen_offset: usize) -> Result<Tensor> {
        let (bsz, seq_len, _) = hidden.dims3()?;

        // 1. Projektion
        let q = self.q_proj.forward(hidden)?
            .reshape((bsz, seq_len, self.num_heads, self.head_dim))?
            .permute((0, 2, 1, 3))?; // (bsz, heads, seq, head_dim)

        let k = self.k_proj.forward(hidden)?
            .reshape((bsz, seq_len, self.num_kv_heads, self.head_dim))?
            .permute((0, 2, 1, 3))?;

        let v = self.v_proj.forward(hidden)?
            .reshape((bsz, seq_len, self.num_kv_heads, self.head_dim))?
            .permute((0, 2, 1, 3))?;

        // 2. QK-Norm (per Head, über head_dim)
        let q = if let Some(norm) = &self.q_norm { norm.forward(&q)? } else { q };
        let k = if let Some(norm) = &self.k_norm { norm.forward(&k)? } else { k };

        // 3. RoPE
        let (cos, sin) = self.rotary_emb.get(seq_len, seqlen_offset)?;
        let (q, k) = apply_rotary(&q, &k, &cos, &sin)?;

        // 4. KV-Cache Update
        let (k, v) = self.update_cache(k, v)?;

        // 5. GQA: KV auf alle Heads expandieren
        let k = repeat_kv(&k, self.num_kv_groups)?;
        let v = repeat_kv(&v, self.num_kv_groups)?;

        // 6. Scaled Dot-Product Attention
        // scores: (bsz, heads, seq, seq_total)
        let scores = (q.matmul(&k.transpose(D::Minus2, D::Minus1)?)? * self.scale)?;

        // Causal Mask für neue Tokens
        let scores = apply_causal_mask(&scores, seqlen_offset)?;

        let probs = candle_nn::ops::softmax(&scores, D::Minus1)?;
        let attn_out = probs.matmul(&v)?; // (bsz, heads, seq, head_dim)

        // 7. Output Projektion
        let out = attn_out
            .permute((0, 2, 1, 3))?
            .reshape((bsz, seq_len, self.num_heads * self.head_dim))?;

        self.o_proj.forward(&out)
    }

    fn update_cache(&mut self, k: Tensor, v: Tensor) -> Result<(Tensor, Tensor)> {
        match &self.kv_cache {
            None => {
                self.kv_cache = Some((k.clone(), v.clone()));
                Ok((k, v))
            }
            Some((cached_k, cached_v)) => {
                let new_k = Tensor::cat(&[cached_k, &k], 2)?;
                let new_v = Tensor::cat(&[cached_v, &v], 2)?;
                self.kv_cache = Some((new_k.clone(), new_v.clone()));
                Ok((new_k, new_v))
            }
        }
    }
}

/// GQA: KV Heads auf num_kv_groups expandieren.
fn repeat_kv(x: &Tensor, n_rep: usize) -> Result<Tensor> {
    if n_rep == 1 {
        return Ok(x.clone());
    }
    let (bsz, n_kv_heads, seq_len, head_dim) = x.dims4()?;
    x.unsqueeze(2)?                                        // (bsz, kv, 1, seq, dim)
        .expand((bsz, n_kv_heads, n_rep, seq_len, head_dim))?
        .reshape((bsz, n_kv_heads * n_rep, seq_len, head_dim))
}

/// Causal Mask: zukünftige Tokens auf -inf setzen.
fn apply_causal_mask(scores: &Tensor, _offset: usize) -> Result<Tensor> {
    let (bsz, heads, q_len, kv_len) = scores.dims4()?;
    if q_len == 1 {
        // Decoding-Schritt: kein Masking nötig
        return Ok(scores.clone());
    }
    // Prefill: unteres Dreieck erlaubt
    let mask = Tensor::tril2(kv_len, scores.dtype(), scores.device())?;
    let mask = mask.broadcast_as((bsz, heads, kv_len, kv_len))?;
    let neg_inf = Tensor::full(f32::NEG_INFINITY, (bsz, heads, q_len, kv_len), scores.device())?;
    mask.where_cond(scores, &neg_inf)
}
