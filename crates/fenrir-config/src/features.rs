//! Feature Flags — experimentelle Features ein/ausschalten.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeatureFlags {
    /// WebRTC aktivieren (Privacy-Risiko)
    pub webrtc: bool,
    /// WebGPU aktivieren
    pub webgpu: bool,
    /// AI-Inline-Assistent (Seiteninhalt analysieren)
    pub ai_assistant: bool,
    /// MCP Plugin-Host
    pub mcp_plugins: bool,
    /// P2P Sync (Phase 6)
    pub p2p_sync: bool,
    /// Web3 / ENS (Phase 5)
    pub web3: bool,
    /// VPN Integration (Phase 4)
    pub vpn: bool,
    /// Debug Panel immer sichtbar
    pub debug_panel: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            webrtc: false,
            webgpu: false,
            ai_assistant: false, // bis Modell geladen
            mcp_plugins: false,
            p2p_sync: false,
            web3: false,
            vpn: false,
            debug_panel: false,
        }
    }
}

impl FeatureFlags {
    pub fn is_enabled(&self, feature: &str) -> bool {
        match feature {
            "webrtc"       => self.webrtc,
            "webgpu"       => self.webgpu,
            "ai_assistant" => self.ai_assistant,
            "mcp_plugins"  => self.mcp_plugins,
            "p2p_sync"     => self.p2p_sync,
            "web3"         => self.web3,
            "vpn"          => self.vpn,
            "debug_panel"  => self.debug_panel,
            _              => false,
        }
    }
}
