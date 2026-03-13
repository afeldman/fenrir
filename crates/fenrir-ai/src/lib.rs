//! fenrir-ai — Lokale LLM Inference für Fenrir Browser.
//!
//! Implementiert das Noeum-1-Nano Modell (Ramo-Architektur) nativ in Rust/Candle.
//!
//! # Modell: noeum/noeum-1-nano
//! - Herkunft: Österreich (EU-native)
//! - Architektur: Ramo (MoE + GQA + QK-Norm + YaRN-RoPE)
//! - Größe: 0.6B total / ~0.2B aktiv
//! - Lizenz: Apache 2.0
//! - Thinking Mode: `<think>` → System-2 Reasoning
//!
//! # Nutzung
//! ```ignore
//! use fenrir_ai::{NoeumEngine, InferenceRequest};
//!
//! let engine = NoeumEngine::load().await?;
//! let response = engine.complete(InferenceRequest {
//!     prompt: "Was ist Privacy?".into(),
//!     thinking: Some(()),
//!     max_tokens: Some(128),
//! }).await?;
//! ```

pub mod bookmarks;
pub mod download;
pub mod inference;
pub mod model;
pub mod tokenizer;
pub mod noeum;

pub use bookmarks::{BookmarkAnalysis, analyze_bookmark};
pub use download::ModelDownloader;
pub use inference::{InferenceRequest, InferenceResponse, InferenceEngine};
pub use noeum::{NoeumEngine};

