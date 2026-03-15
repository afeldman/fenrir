use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum FenrirError {
    #[error("Servo initialization failed: {0}")]
    ServoInit(String),

    #[error("WebView creation failed: {0}")]
    WebViewCreate(String),

    #[error("Invalid URL")]
    InvalidUrl,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("AI/ML error: {0}")]
    Ai(String),

    #[error("Candle ML error: {0}")]
    Candle(#[from] candle_core::Error),

    #[error("Engine not running")]
    EngineNotRunning,

    #[error("Tab not found: {0}")]
    TabNotFound(Uuid),
}

/// Kurzform für Result mit FenrirError.
pub type FenrirResult<T> = Result<T, FenrirError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let servo_error = FenrirError::ServoInit("test failure".to_string());
        assert_eq!(
            format!("{}", servo_error),
            "Servo initialization failed: test failure"
        );

        let webview_error = FenrirError::WebViewCreate("creation failed".to_string());
        assert_eq!(
            format!("{}", webview_error),
            "WebView creation failed: creation failed"
        );

        let invalid_url = FenrirError::InvalidUrl;
        assert_eq!(format!("{}", invalid_url), "Invalid URL");

        let config_error = FenrirError::Config("missing field".to_string());
        assert_eq!(format!("{}", config_error), "Configuration error: missing field");

        let crypto_error = FenrirError::Crypto("encryption failed".to_string());
        assert_eq!(format!("{}", crypto_error), "Crypto error: encryption failed");

        let network_error = FenrirError::Network("timeout".to_string());
        assert_eq!(format!("{}", network_error), "Network error: timeout");

        let storage_error = FenrirError::Storage("corrupted".to_string());
        assert_eq!(format!("{}", storage_error), "Storage error: corrupted");

        let permission_error = FenrirError::PermissionDenied("access denied".to_string());
        assert_eq!(
            format!("{}", permission_error),
            "Permission denied: access denied"
        );

        let io_error = FenrirError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(format!("{}", io_error).contains("file not found"));

        let engine_error = FenrirError::EngineNotRunning;
        assert_eq!(format!("{}", engine_error), "Engine not running");

        let tab_id = Uuid::new_v4();
        let tab_error = FenrirError::TabNotFound(tab_id);
        assert!(format!("{}", tab_error).contains(&tab_id.to_string()));
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let fenrir_error: FenrirError = io_error.into();
        match fenrir_error {
            FenrirError::Io(_) => (),
            _ => panic!("Expected Io variant"),
        }
    }

    #[test]
    fn test_fenrir_result_type() {
        let ok_result: FenrirResult<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        let err_result: FenrirResult<i32> = Err(FenrirError::InvalidUrl);
        assert!(err_result.is_err());
    }
}
