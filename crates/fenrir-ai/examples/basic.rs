//! Beispiel für die Nutzung der fenrir-ai Bibliothek.
//!
//! Dieses Beispiel zeigt wie man das noeum-1-nano Modell lädt und verwendet.

use fenrir_ai::{NoeumEngine, InferenceRequest, analyze_bookmark};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Fenrir AI Beispiel ===");
    
    // 1. Modell laden (lädt automatisch von HuggingFace wenn nicht vorhanden)
    println!("Lade noeum-1-nano Modell...");
    let mut engine = NoeumEngine::load().await?;
    println!("Modell geladen ✓");
    
    // 2. Einfache Text-Generierung
    println!("\n--- Text-Generierung ---");
    let response = engine.complete(InferenceRequest {
        prompt: "Was ist Privacy-by-Design?".to_string(),
        max_tokens: Some(50),
        thinking: None,
    }).await?;
    
    println!("Antwort: {}", response.text);
    println!("Generierte Tokens: {}", response.tokens_generated);
    
    // 3. Mit Thinking Mode
    println!("\n--- Text-Generierung mit Thinking Mode ---");
    let response = engine.complete(InferenceRequest {
        prompt: "Erkläre den Unterschied zwischen symmetrischer und asymmetrischer Verschlüsselung.".to_string(),
        max_tokens: Some(100),
        thinking: Some(()),
    }).await?;
    
    println!("Antwort: {}", response.text);
    if let Some(thinking) = response.thinking_content {
        println!("Thinking Block: {}", thinking);
    }
    println!("Generierte Tokens: {}", response.tokens_generated);
    
    // 4. Bookmark-Analyse
    println!("\n--- Bookmark-Analyse ---");
    let analysis = analyze_bookmark(
        &mut engine,
        "https://rust-lang.org",
        "The Rust Programming Language",
        "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.",
    ).await?;
    
    println!("Kategorie: {}", analysis.category);
    println!("Beschreibung: {}", analysis.description);
    println!("Tags: {:?}", analysis.tags);
    
    Ok(())
}
