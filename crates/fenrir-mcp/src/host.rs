//! FenrirMcpHost — verbindet Fenrir mit externen MCP-Servern.
//!
//! Fenrir als MCP Client: kann Tools auf MCP-Servern aufrufen.
//! Fenrir als MCP Server: exponiert Browser-Fähigkeiten als MCP-Tools.
//!
//! Phase 1: Router + Provider-Stubs
//! Phase 2: Echte MCP-Verbindungen via rmcp (stdio, HTTP/SSE)
//! Phase 3: Fenrir als MCP-Server (Tabs, Bookmarks als Tools)

use crate::router::LlmRouter;
use fenrir_core::error::FenrirError;
use tracing::info;

pub struct FenrirMcpHost {
    pub router: LlmRouter,
}

impl FenrirMcpHost {
    pub fn new(router: LlmRouter) -> Self {
        Self { router }
    }

    /// Kurzform: Text-Completion mit dem besten verfügbaren Provider.
    pub async fn complete(&self, prompt: &str) -> Result<String, FenrirError> {
        use crate::provider::{ChatMessage, LlmRequest, Role};

        info!("MCP complete request");

        let request = LlmRequest {
            system: None,
            messages: vec![ChatMessage {
                role: Role::User,
                content: prompt.to_string(),
            }],
            max_tokens: Some(512),
            temperature: Some(0.7),
        };

        let response = self.router.complete(request).await?;
        Ok(response.content)
    }
}
