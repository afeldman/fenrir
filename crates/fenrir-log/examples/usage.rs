//! Beispiel für die Verwendung von fenrir-log in anderen Crates.

use fenrir_log::{FenrirLogger, LogConfig};

/// Beispiel für Performance-Logging in fenrir-network
pub struct NetworkLogger {
    logger: FenrirLogger,
}

impl NetworkLogger {
    pub fn new() -> anyhow::Result<Self> {
        let logger = FenrirLogger::builder()
            .with_tracing()
            .with_perf_backend()  // Wichtig für Netzwerk-Logs
            .with_directive("fenrir_network=trace")
            .build()?;
        
        Ok(Self { logger })
    }
    
    pub fn log_request(&self, url: &str, method: &str, duration_ms: u64) {
        // Performance-kritischer Pfad: verwendet inqjet wenn enabled
        self.logger.perf_log(
            "network::request",
            format!("{} {} ({}ms)", method, url, duration_ms)
        );
    }
    
    pub fn log_dns(&self, hostname: &str, ips: &[std::net::IpAddr]) {
        self.logger.perf_log(
            "network::dns",
            format!("Resolved {} to {:?}", hostname, ips)
        );
    }
}

/// Beispiel für strukturiertes Logging mit Pod-Typen
#[cfg(feature = "perf")]
#[derive(Debug, fenrir_log::perf::Pod)]
pub struct BandwidthUsage {
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub timestamp: std::time::SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_logger() {
        let logger = NetworkLogger::new().unwrap();
        logger.log_request("https://example.com", "GET", 42);
        logger.log_dns("example.com", &[std::net::IpAddr::V4(std::net::Ipv4Addr::new(93, 184, 216, 34))]);
    }
    
    #[test]
    #[cfg(feature = "perf")]
    fn test_pod_logging() {
        use fenrir_log::perf::Pod;
        
        let usage = BandwidthUsage {
            tx_bytes: 1024,
            rx_bytes: 2048,
            timestamp: std::time::SystemTime::now(),
        };
        
        // Verify Pod trait is implemented
        let _ = &usage as &dyn Pod;
    }
}
