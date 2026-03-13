//! Nutzerprofile — isolierte Browser-Umgebungen.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
    pub avatar_color: String, // Hex-Farbe für UI
    pub data_dir: Option<String>, // None → auto
}

impl UserProfile {
    pub fn default_profile() -> Self {
        Self {
            id: "default".to_string(),
            name: "Standard".to_string(),
            avatar_color: "#4A90D9".to_string(),
            data_dir: None,
        }
    }
}
