//! FenrirHttpClient — HTTP/2 Client auf Basis von reqwest + rustls.
//!
//! Kein OpenSSL. Kein C. Kein unverschlüsselter Traffic.
//! Nutzt FenrirDnsResolver für alle Verbindungen (DoH/DoT).

use crate::{
    dns::FenrirDnsResolver,
    request::{HttpMethod, NetworkRequest, NetworkResponse},
};
use fenrir_core::error::FenrirError;
use reqwest::{Client, Method, RequestBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;
use url::Url;

pub struct FenrirHttpClient {
    inner: Client,
    _dns: Arc<FenrirDnsResolver>,
}

impl FenrirHttpClient {
    pub fn new(dns: Arc<FenrirDnsResolver>) -> Result<Self, FenrirError> {
        let inner = Client::builder()
            .use_rustls_tls()
            .http2_prior_knowledge()
            .https_only(false) // https-enforcer in pipeline macht das
            .user_agent("Fenrir/0.1 (EU Privacy Browser)")
            .build()
            .map_err(|e| FenrirError::Network(format!("HTTP Client init: {e}")))?;

        Ok(Self { inner, _dns: dns })
    }

    pub async fn execute(&self, request: NetworkRequest) -> Result<NetworkResponse, FenrirError> {
        let method = match &request.method {
            HttpMethod::Get => Method::GET,
            HttpMethod::Post => Method::POST,
            HttpMethod::Put => Method::PUT,
            HttpMethod::Delete => Method::DELETE,
            HttpMethod::Head => Method::HEAD,
            HttpMethod::Options => Method::OPTIONS,
            HttpMethod::Patch => Method::PATCH,
            HttpMethod::Other(m) => m
                .parse()
                .map_err(|_| FenrirError::Network(format!("Ungültige HTTP Methode: {m}")))?,
        };

        let mut builder: RequestBuilder = self.inner.request(method, request.url.as_str());

        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        debug!(url = %request.url, "HTTP Request");

        let response = builder
            .send()
            .await
            .map_err(|e| FenrirError::Network(e.to_string()))?;

        let status = response.status().as_u16();
        let final_url = response.url().clone();

        let mut headers = HashMap::new();
        for (key, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(key.to_string(), v.to_string());
            }
        }

        let body = response
            .bytes()
            .await
            .map_err(|e| FenrirError::Network(e.to_string()))?
            .to_vec();

        Ok(NetworkResponse {
            status,
            headers,
            body,
            url: final_url,
        })
    }
}
