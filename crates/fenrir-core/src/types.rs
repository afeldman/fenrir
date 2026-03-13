use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

/// Eindeutige ID für einen Browser-Tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(Uuid);

impl TabId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for TabId { fn default() -> Self { Self::new() } }

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Eindeutige Geräte-ID (persistent, für Sync).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(Uuid);

impl DeviceId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn as_str(&self) -> String { self.0.to_string() }
}

impl Default for DeviceId { fn default() -> Self { Self::new() } }

/// Unix-Timestamp in Millisekunden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self(ms)
    }
    pub fn as_millis(&self) -> u64 { self.0 }
}

/// Validierte URL (Newtype über url::Url).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FenrirUrl(Url);

impl FenrirUrl {
    pub fn parse(s: &str) -> Result<Self, url::ParseError> {
        Ok(Self(Url::parse(s)?))
    }
    pub fn as_url(&self) -> &Url { &self.0 }
    pub fn as_str(&self) -> &str { self.0.as_str() }
    pub fn scheme(&self) -> &str { self.0.scheme() }
}

impl std::fmt::Display for FenrirUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for FenrirUrl {
    type Error = url::ParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> { Self::parse(s) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_id_uniqueness() {
        let id1 = TabId::new();
        let id2 = TabId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_tab_id_display() {
        let id = TabId::new();
        let display = format!("{}", id);
        assert!(display.len() > 0);
        assert!(display.contains('-'));
    }

    #[test]
    fn test_device_id_as_str() {
        let device_id = DeviceId::new();
        let str_repr = device_id.as_str();
        assert!(str_repr.len() > 0);
        assert!(str_repr.contains('-'));
    }

    #[test]
    fn test_timestamp_now() {
        let ts = Timestamp::now();
        assert!(ts.as_millis() > 0);
    }

    #[test]
    fn test_timestamp_ordering() {
        let ts1 = Timestamp(1000);
        let ts2 = Timestamp(2000);
        assert!(ts1 < ts2);
        assert!(ts2 > ts1);
    }

    #[test]
    fn test_fenrir_url_parse_valid() {
        let url = FenrirUrl::parse("https://example.com");
        assert!(url.is_ok());
        let url = url.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.as_str(), "https://example.com/");
    }

    #[test]
    fn test_fenrir_url_parse_invalid() {
        let url = FenrirUrl::parse("not a valid url");
        assert!(url.is_err());
    }

    #[test]
    fn test_fenrir_url_try_from() {
        let url: Result<FenrirUrl, _> = "https://example.com".try_into();
        assert!(url.is_ok());
    }

    #[test]
    fn test_fenrir_url_display() {
        let url = FenrirUrl::parse("https://example.com").unwrap();
        let display = format!("{}", url);
        assert_eq!(display, "https://example.com/");
    }
}
