use fenrir_config::{FenrirConfig, FeatureFlags, SearchEngine};

fn main() {
    // Test 1: Default Config
    let config = FenrirConfig::default();
    println!("Default Config:");
    println!("  Start URL: {}", config.ui.start_url);
    println!("  Window: {}x{}", config.ui.window_width, config.ui.window_height);
    println!("  Block Trackers: {}", config.privacy.block_trackers);
    println!("  AI Enabled: {}", config.ai.enabled);
    println!("  Model ID: {}", config.ai.model_id);
    
    // Test 2: Feature Flags
    let features = FeatureFlags::default();
    println!("\nFeature Flags:");
    println!("  WebRTC: {}", features.is_enabled("webrtc"));
    println!("  WebGPU: {}", features.is_enabled("webgpu"));
    println!("  AI Assistant: {}", features.is_enabled("ai_assistant"));
    
    // Test 3: Search Engine URLs
    println!("\nSearch Engine URLs:");
    let query = "rust programming";
    println!("  DuckDuckGo: {}", SearchEngine::DuckDuckGo.search_url(query));
    println!("  Brave: {}", SearchEngine::Brave.search_url(query));
    println!("  Kagi: {}", SearchEngine::Kagi.search_url(query));
    
    // Test 4: Config Path
    let path = fenrir_config::config_path();
    println!("\nConfig Path: {}", path.display());
    
    // Test 5: TOML Serialization
    let toml = toml::to_string_pretty(&config).unwrap();
    println!("\nTOML Preview (first 200 chars):");
    println!("{}", &toml[..200.min(toml.len())]);
}
