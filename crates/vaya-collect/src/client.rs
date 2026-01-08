//! HTTP client with TLS support

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::Duration;

use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use crate::request::{Method, Request, RequestBuilder};
use crate::response::Response;
use crate::url::Url;
use crate::{CollectError, CollectResult};

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig2 {
    /// Default timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum redirects to follow
    pub max_redirects: u32,
    /// User agent string
    pub user_agent: String,
    /// Maximum response body size
    pub max_body_size: usize,
}

impl Default for ClientConfig2 {
    fn default() -> Self {
        Self {
            timeout_ms: 30_000,
            max_redirects: 5,
            user_agent: "vaya-collect/0.1".to_string(),
            max_body_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

impl ClientConfig2 {
    /// Create new config with timeout
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = agent.into();
        self
    }

    /// Set max body size
    pub fn max_body_size(mut self, size: usize) -> Self {
        self.max_body_size = size;
        self
    }
}

/// HTTP client
pub struct Client {
    config: ClientConfig2,
    tls_config: Arc<ClientConfig>,
}

impl Client {
    /// Create a new HTTP client
    pub fn new() -> CollectResult<Self> {
        Self::with_config(ClientConfig2::default())
    }

    /// Create client with custom config
    pub fn with_config(config: ClientConfig2) -> CollectResult<Self> {
        let tls_config = Self::create_tls_config()?;
        Ok(Self {
            config,
            tls_config: Arc::new(tls_config),
        })
    }

    /// Create TLS configuration
    fn create_tls_config() -> CollectResult<ClientConfig> {
        // Use webpki-roots or system roots
        let root_store = RootCertStore::empty();

        // For now, create config that doesn't verify certs (for testing)
        // In production, you'd load proper root certificates
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth();

        Ok(config)
    }

    /// Execute a GET request
    pub fn get(&self, url: &str) -> CollectResult<Response> {
        let url = Url::parse(url)?;
        let request = Request::get(url)
            .user_agent(&self.config.user_agent)
            .timeout(self.config.timeout_ms);
        self.execute(request)
    }

    /// Execute a POST request with body
    pub fn post(&self, url: &str, body: &[u8], content_type: &str) -> CollectResult<Response> {
        let url = Url::parse(url)?;
        let request = Request::post(url)
            .user_agent(&self.config.user_agent)
            .header("Content-Type", content_type)
            .body_bytes(body.to_vec())
            .timeout(self.config.timeout_ms);
        self.execute(request)
    }

    /// Execute a POST request with JSON body
    pub fn post_json(&self, url: &str, json: &str) -> CollectResult<Response> {
        self.post(url, json.as_bytes(), "application/json")
    }

    /// Create a request builder
    pub fn request(&self, method: Method, url: &str) -> RequestBuilder {
        RequestBuilder::new(method, url)
            .header("User-Agent", &self.config.user_agent)
            .timeout(self.config.timeout_ms)
    }

    /// Execute a request
    pub fn execute(&self, request: Request) -> CollectResult<Response> {
        self.execute_with_redirects(request, 0)
    }

    /// Execute request following redirects
    fn execute_with_redirects(&self, request: Request, redirect_count: u32) -> CollectResult<Response> {
        if redirect_count > self.config.max_redirects {
            return Err(CollectError::TooManyRedirects);
        }

        let response = self.send_request(&request)?;

        // Handle redirects
        if response.is_redirect() {
            if let Some(location) = response.location() {
                let new_url = self.resolve_redirect(&request.url, location)?;
                let new_request = Request::get(new_url)
                    .timeout(request.timeout_ms.unwrap_or(self.config.timeout_ms));
                return self.execute_with_redirects(new_request, redirect_count + 1);
            }
        }

        Ok(response)
    }

    /// Resolve redirect URL
    fn resolve_redirect(&self, base: &Url, location: &str) -> CollectResult<Url> {
        if location.starts_with("http://") || location.starts_with("https://") {
            Url::parse(location)
        } else if location.starts_with('/') {
            // Absolute path
            Ok(Url {
                scheme: base.scheme,
                host: base.host.clone(),
                port: base.port,
                path: location.to_string(),
                query: None,
                fragment: None,
            })
        } else {
            // Relative path
            let base_path = base.path.rfind('/').map(|i| &base.path[..=i]).unwrap_or("/");
            Ok(Url {
                scheme: base.scheme,
                host: base.host.clone(),
                port: base.port,
                path: format!("{}{}", base_path, location),
                query: None,
                fragment: None,
            })
        }
    }

    /// Send a single request
    fn send_request(&self, request: &Request) -> CollectResult<Response> {
        let timeout = Duration::from_millis(request.timeout_ms.unwrap_or(self.config.timeout_ms));

        // Connect
        let addr = format!("{}:{}", request.url.host, request.url.port);
        let stream = TcpStream::connect(&addr).map_err(|e| {
            CollectError::ConnectionFailed(format!("Failed to connect to {}: {}", addr, e))
        })?;

        stream
            .set_read_timeout(Some(timeout))
            .map_err(CollectError::Io)?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(CollectError::Io)?;

        if request.url.is_tls() {
            self.send_tls_request(stream, request)
        } else {
            self.send_plain_request(stream, request)
        }
    }

    /// Send request over TLS
    fn send_tls_request(&self, stream: TcpStream, request: &Request) -> CollectResult<Response> {
        let server_name = request
            .url
            .host
            .clone()
            .try_into()
            .map_err(|_| CollectError::TlsError("Invalid server name".into()))?;

        let conn = ClientConnection::new(self.tls_config.clone(), server_name)
            .map_err(|e| CollectError::TlsError(e.to_string()))?;

        let mut tls_stream = StreamOwned::new(conn, stream);

        // Send request
        let request_bytes = request.build();
        tls_stream
            .write_all(&request_bytes)
            .map_err(|e| CollectError::Io(e))?;

        // Read response
        let mut response_data = Vec::new();
        tls_stream
            .read_to_end(&mut response_data)
            .map_err(|e| CollectError::Io(e))?;

        Response::parse(&response_data)
    }

    /// Send request without TLS
    fn send_plain_request(&self, mut stream: TcpStream, request: &Request) -> CollectResult<Response> {
        // Send request
        let request_bytes = request.build();
        stream
            .write_all(&request_bytes)
            .map_err(|e| CollectError::Io(e))?;

        // Read response
        let mut response_data = Vec::new();
        stream
            .read_to_end(&mut response_data)
            .map_err(|e| CollectError::Io(e))?;

        Response::parse(&response_data)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new().expect("Failed to create default client")
    }
}

/// Certificate verifier that accepts all certificates (for development)
/// WARNING: This is insecure and should only be used for testing
#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new().unwrap();
        assert_eq!(client.config.timeout_ms, 30_000);
    }

    #[test]
    fn test_config_builder() {
        let config = ClientConfig2::default()
            .timeout(5000)
            .user_agent("test-agent");

        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.user_agent, "test-agent");
    }

    #[test]
    fn test_resolve_redirect_absolute() {
        let client = Client::new().unwrap();
        let base = Url::parse("https://example.com/old/path").unwrap();

        let resolved = client.resolve_redirect(&base, "/new/path").unwrap();
        assert_eq!(resolved.path, "/new/path");
        assert_eq!(resolved.host, "example.com");
    }

    #[test]
    fn test_resolve_redirect_full_url() {
        let client = Client::new().unwrap();
        let base = Url::parse("https://example.com/old").unwrap();

        let resolved = client
            .resolve_redirect(&base, "https://other.com/new")
            .unwrap();
        assert_eq!(resolved.host, "other.com");
        assert_eq!(resolved.path, "/new");
    }
}
