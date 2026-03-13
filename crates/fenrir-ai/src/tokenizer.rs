//! NoeumTokenizer — SentencePiece Tokenizer für noeum-1-nano.
//!
//! vocab_size: 46.957
//! Format: SentencePiece (.model Datei)
//! Besondere Tokens: BOS=1, EOS=2, PAD=46945, UNK=46946

use fenrir_core::error::FenrirError;
use std::path::Path;
use tokenizers::Tokenizer;

pub struct NoeumTokenizer {
    inner: Tokenizer,
    eos_token_id: u32,
    bos_token_id: u32,
}

impl NoeumTokenizer {
    pub fn load(model_dir: &Path) -> Result<Self, FenrirError> {
        let tokenizer_path = model_dir.join("tokenizer.model");

        // HuggingFace tokenizers Bibliothek lädt SentencePiece nativ
        let inner = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| FenrirError::Config(format!("Tokenizer laden: {e}")))?;

        Ok(Self {
            inner,
            eos_token_id: 2,   // noeum-1-nano config
            bos_token_id: 1,
        })
    }

    /// Text → Token-IDs.
    pub fn encode(&self, text: &str, add_bos: bool) -> Result<Vec<u32>, FenrirError> {
        let encoding = self.inner
            .encode(text, false)
            .map_err(|e| FenrirError::Config(format!("Encode Fehler: {e}")))?;

        let mut ids: Vec<u32> = encoding.get_ids().to_vec();
        if add_bos {
            ids.insert(0, self.bos_token_id);
        }
        Ok(ids)
    }

    /// Token-IDs → Text.
    pub fn decode(&self, ids: &[u32]) -> Result<String, FenrirError> {
        self.inner
            .decode(ids, true)
            .map_err(|e| FenrirError::Config(format!("Decode Fehler: {e}")))
    }

    pub fn eos_id(&self) -> u32 {
        self.eos_token_id
    }

    pub fn bos_id(&self) -> u32 {
        self.bos_token_id
    }

    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(true)
    }
}
