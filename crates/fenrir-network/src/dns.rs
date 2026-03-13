//! FenrirDnsResolver — DNS-over-HTTPS und DNS-over-TLS via hickory-dns.
//!
//! Privacy-first DNS: kein unverschlüsselter DNS-Traffic.
//! Standard: Cloudflare DoT (1.1.1.1) — konfigurierbar auf eigene Server.

use fenrir_core::error::FenrirError;
use hickory_resolver::{
    TokioResolver,
    config::{CLOUDFLARE, ResolverConfig, ResolverOpts},
    net::runtime::TokioRuntimeProvider,
};
use std::net::IpAddr;
use tracing::{debug, info, warn};

pub struct FenrirDnsResolver {
    inner: TokioResolver,
}

impl FenrirDnsResolver {
    /// DNS-over-HTTPS mit Cloudflare — Standard.
    pub async fn new_doh() -> Result<Self, FenrirError> {
        let config = ResolverConfig::https(&CLOUDFLARE);
        Self::build(config)
    }

    /// DNS-over-TLS mit Cloudflare — für strikte Privacy-Setups.
    pub async fn new_dot() -> Result<Self, FenrirError> {
        let config = ResolverConfig::tls(&CLOUDFLARE);
        Self::build(config)
    }

    /// System-DNS als Fallback (z.B. für lokale Entwicklung).
    pub async fn new_system() -> Result<Self, FenrirError> {
        let inner = TokioResolver::builder_tokio()
            .map_err(|e| FenrirError::Network(format!("System DNS init: {e}")))?
            .build()
            .map_err(|e| FenrirError::Network(format!("System DNS build: {e}")))?;
        info!("System DNS Resolver initialisiert");
        Ok(Self { inner })
    }

    /// Custom ResolverConfig (z.B. eigener DoH-Server, Pi-hole).
    pub async fn new_custom(config: ResolverConfig) -> Result<Self, FenrirError> {
        Self::build(config)
    }

    fn build(config: ResolverConfig) -> Result<Self, FenrirError> {
        let mut opts = ResolverOpts::default();
        opts.cache_size = 512;

        let inner = TokioResolver::builder_with_config(config, TokioRuntimeProvider::default())
            .with_options(opts)
            .build()
            .map_err(|e| FenrirError::Network(format!("DNS Resolver build: {e}")))?;

        info!("DNS Resolver initialisiert");
        Ok(Self { inner })
    }

    /// Hostname → IP-Adressen auflösen.
    pub async fn resolve(&self, host: &str) -> Result<Vec<IpAddr>, FenrirError> {
        debug!(host = %host, resolver = "DoH", "dns: lookup");
        
        let lookup = self
            .inner
            .lookup_ip(host)
            .await
            .map_err(|e| {
                warn!(host = %host, error = %e, "dns: lookup fehlgeschlagen");
                FenrirError::Network(format!("DNS Fehler für '{host}': {e}"))
            })?;

        let addrs: Vec<IpAddr> = lookup.iter().collect();
        if addrs.is_empty() {
            warn!(host = %host, "dns: keine Einträge gefunden");
            return Err(FenrirError::Network(format!("Keine DNS-Einträge für '{host}'")));
        }

        debug!(host = %host, addrs = ?addrs, "dns: lookup erfolgreich");
        Ok(addrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn doh_resolver_creates() {
        // Nur Initialisierung prüfen, kein echter DNS-Request in Tests
        let result = FenrirDnsResolver::new_doh().await;
        assert!(result.is_ok(), "DoH Resolver sollte erstellt werden können");
    }

    #[tokio::test]
    async fn system_resolver_creates() {
        let result = FenrirDnsResolver::new_system().await;
        assert!(result.is_ok());
    }
}
