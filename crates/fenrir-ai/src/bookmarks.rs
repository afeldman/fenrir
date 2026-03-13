//! AI-gestützte Bookmark-Analyse — Kategorisierung + Description.

use crate::{InferenceRequest, NoeumEngine};
use fenrir_core::error::FenrirError;
use serde::Deserialize;

/// Ergebnis der Bookmark-Analyse.
#[derive(Debug, Clone)]
pub struct BookmarkAnalysis {
    /// Kategorie (z.B. "Technology", "News", "Shopping").
    pub category: String,
    /// Kurze Beschreibung (1 Satz).
    pub description: String,
    /// Tags für die Suche.
    pub tags: Vec<String>,
}

/// Kategorisiert einen Bookmark via noeum-1-nano.
pub async fn analyze_bookmark(
    engine: &mut NoeumEngine,
    url: &str,
    title: &str,
    page_text: &str,  // max ~500 Zeichen Seiteninhalt
) -> Result<BookmarkAnalysis, FenrirError> {
    let prompt = format!(
        "Analysiere diese Webseite kurz und präzise:\nURL: {url}\nTitel: {title}\nInhalt: {}\n\n\
         Antworte NUR in diesem JSON-Format:\n\
         {{\"category\": \"<Kategorie>\", \"description\": \"<1 Satz>\", \"tags\": [\"tag1\", \"tag2\"]}}",
        &page_text[..page_text.len().min(500)]
    );

    let response = engine.complete(InferenceRequest {
        prompt,
        max_tokens: Some(128),
        thinking: None,
    }).await?;

    parse_analysis_response(&response.text)
}

/// Parst die JSON-Antwort vom Modell.
fn parse_analysis_response(text: &str) -> Result<BookmarkAnalysis, FenrirError> {
    // JSON aus Response extrahieren
    let json_start = text.find('{').unwrap_or(0);
    let json_end = text.rfind('}').map(|i| i + 1).unwrap_or(text.len());
    let json_str = &text[json_start..json_end];

    #[derive(Debug, Deserialize)]
    struct Raw {
        category: String,
        description: String,
        tags: Vec<String>,
    }

    let raw: Raw = serde_json::from_str(json_str)
        .map_err(|e| FenrirError::Ai(format!("JSON parse: {e}")))?;

    Ok(BookmarkAnalysis {
        category: raw.category,
        description: raw.description,
        tags: raw.tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_analysis_response() {
        let text = r#"{
            "category": "Technology",
            "description": "A blog about Rust programming",
            "tags": ["rust", "programming", "blog"]
        }"#;
        
        let result = parse_analysis_response(text).unwrap();
        assert_eq!(result.category, "Technology");
        assert_eq!(result.description, "A blog about Rust programming");
        assert_eq!(result.tags, vec!["rust", "programming", "blog"]);
    }

    #[test]
    fn test_parse_analysis_response_with_extra_text() {
        let text = r#"First some text {
            "category": "News",
            "description": "Latest news updates",
            "tags": ["news", "updates"]
        } and some more text after"#;
        
        let result = parse_analysis_response(text).unwrap();
        assert_eq!(result.category, "News");
        assert_eq!(result.description, "Latest news updates");
        assert_eq!(result.tags, vec!["news", "updates"]);
    }

    #[test]
    fn test_parse_analysis_response_invalid_json() {
        let text = r#"{
            "category": "Test",
            "description": "Test",
            "tags": ["test"]
        "#; // Missing closing brace
        
        let result = parse_analysis_response(text);
        assert!(result.is_err());
    }
}
