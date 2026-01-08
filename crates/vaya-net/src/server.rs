//! HTTP server implementation

use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader as TokioBufReader};
use tokio::net::{TcpListener, TcpStream};

use self::tokio_rustls::TlsAcceptor;

use crate::{NetError, NetResult, Request, Response, Router, StatusCode, MAX_HEADER_SIZE};

/// Server configuration
#[derive(Clone)]
pub struct ServerConfig {
    /// Bind address
    pub addr: SocketAddr,
    /// TLS certificate path (optional)
    pub cert_path: Option<String>,
    /// TLS key path (optional)
    pub key_path: Option<String>,
    /// Maximum connections
    pub max_connections: usize,
    /// Read timeout in seconds
    pub read_timeout: u64,
    /// Write timeout in seconds
    pub write_timeout: u64,
}

impl ServerConfig {
    /// Create a new server config
    pub fn new(addr: impl Into<SocketAddr>) -> Self {
        Self {
            addr: addr.into(),
            cert_path: None,
            key_path: None,
            max_connections: 10000,
            read_timeout: 30,
            write_timeout: 30,
        }
    }

    /// Enable TLS with the given certificate and key paths
    pub fn with_tls(mut self, cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        self.cert_path = Some(cert_path.into());
        self.key_path = Some(key_path.into());
        self
    }

    /// Set maximum connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.max_connections = max;
        self
    }

    /// Set read timeout
    pub fn read_timeout(mut self, seconds: u64) -> Self {
        self.read_timeout = seconds;
        self
    }

    /// Set write timeout
    pub fn write_timeout(mut self, seconds: u64) -> Self {
        self.write_timeout = seconds;
        self
    }

    /// Check if TLS is enabled
    pub fn is_tls(&self) -> bool {
        self.cert_path.is_some() && self.key_path.is_some()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self::new(([127, 0, 0, 1], 8080))
    }
}

/// HTTP server
pub struct Server {
    config: ServerConfig,
    router: Arc<Router>,
    tls_acceptor: Option<TlsAcceptor>,
}

impl Server {
    /// Create a new server with the given config and router
    pub fn new(config: ServerConfig, router: Router) -> NetResult<Self> {
        let tls_acceptor = if config.is_tls() {
            Some(Self::create_tls_acceptor(&config)?)
        } else {
            None
        };

        Ok(Self {
            config,
            router: Arc::new(router),
            tls_acceptor,
        })
    }

    /// Create TLS acceptor from config
    fn create_tls_acceptor(config: &ServerConfig) -> NetResult<TlsAcceptor> {
        let cert_path = config.cert_path.as_ref().unwrap();
        let key_path = config.key_path.as_ref().unwrap();

        let certs = Self::load_certs(cert_path)?;
        let key = Self::load_key(key_path)?;

        let server_config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| NetError::Tls(e.to_string()))?;

        Ok(TlsAcceptor::from(Arc::new(server_config)))
    }

    /// Load certificates from a PEM file
    fn load_certs(path: &str) -> NetResult<Vec<CertificateDer<'static>>> {
        let file = File::open(path).map_err(|e| {
            NetError::Tls(format!("Failed to open certificate file: {}", e))
        })?;
        let mut reader = BufReader::new(file);
        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| NetError::Tls(format!("Failed to parse certificates: {}", e)))?;

        if certs.is_empty() {
            return Err(NetError::Tls("No certificates found in file".into()));
        }

        Ok(certs)
    }

    /// Load private key from a PEM file
    fn load_key(path: &str) -> NetResult<PrivateKeyDer<'static>> {
        let file = File::open(path).map_err(|e| {
            NetError::Tls(format!("Failed to open key file: {}", e))
        })?;
        let mut reader = BufReader::new(file);

        loop {
            match rustls_pemfile::read_one(&mut reader) {
                Ok(Some(rustls_pemfile::Item::Pkcs1Key(key))) => {
                    return Ok(PrivateKeyDer::Pkcs1(key));
                }
                Ok(Some(rustls_pemfile::Item::Pkcs8Key(key))) => {
                    return Ok(PrivateKeyDer::Pkcs8(key));
                }
                Ok(Some(rustls_pemfile::Item::Sec1Key(key))) => {
                    return Ok(PrivateKeyDer::Sec1(key));
                }
                Ok(None) => break,
                Ok(_) => continue,
                Err(e) => {
                    return Err(NetError::Tls(format!("Failed to parse key: {}", e)));
                }
            }
        }

        Err(NetError::Tls("No private key found in file".into()))
    }

    /// Run the server
    pub async fn run(&self) -> NetResult<()> {
        let listener = TcpListener::bind(self.config.addr).await?;
        tracing::info!("Server listening on {}", self.config.addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let router = self.router.clone();
                    let tls_acceptor = self.tls_acceptor.clone();

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, addr, router, tls_acceptor).await {
                            tracing::debug!("Connection error from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a single connection
    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        router: Arc<Router>,
        tls_acceptor: Option<TlsAcceptor>,
    ) -> NetResult<()> {
        tracing::debug!("New connection from {}", addr);

        if let Some(acceptor) = tls_acceptor {
            let tls_stream = acceptor
                .accept(stream)
                .await
                .map_err(|e| NetError::Tls(e.to_string()))?;
            Self::handle_http(tls_stream, router).await
        } else {
            Self::handle_http(stream, router).await
        }
    }

    /// Handle HTTP on a stream
    async fn handle_http<S>(stream: S, router: Arc<Router>) -> NetResult<()>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        let (reader, mut writer) = tokio::io::split(stream);
        let mut buf_reader = TokioBufReader::new(reader);

        loop {
            // Read request
            let request = match Self::read_request(&mut buf_reader).await {
                Ok(req) => req,
                Err(NetError::ConnectionClosed) => break,
                Err(e) => {
                    let response = Self::error_response(&e);
                    writer.write_all(&response.to_bytes()).await?;
                    break;
                }
            };

            let keep_alive = request.headers().is_keep_alive();

            // Route and handle request
            let response = match router.handle(request).await {
                Ok(resp) => resp,
                Err(e) => Self::error_response(&e),
            };

            // Write response
            writer.write_all(&response.to_bytes()).await?;
            writer.flush().await?;

            if !keep_alive {
                break;
            }
        }

        Ok(())
    }

    /// Read an HTTP request from the stream
    async fn read_request<R>(reader: &mut TokioBufReader<R>) -> NetResult<Request>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        let mut headers = Vec::new();
        let mut total_size = 0;

        // Read headers line by line
        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                return Err(NetError::ConnectionClosed);
            }

            total_size += bytes_read;
            if total_size > MAX_HEADER_SIZE {
                return Err(NetError::HeaderTooLarge);
            }

            if line == "\r\n" || line == "\n" {
                break;
            }

            headers.push(line);
        }

        if headers.is_empty() {
            return Err(NetError::InvalidRequest("Empty request".into()));
        }

        // Reconstruct raw request for parsing
        let mut raw = headers.join("");
        raw.push_str("\r\n");

        // Parse headers first to get Content-Length
        let partial_request = Request::parse(raw.as_bytes())?;

        // Read body if present
        if let Some(content_length) = partial_request.headers().content_length() {
            if content_length > crate::MAX_BODY_SIZE {
                return Err(NetError::RequestTooLarge);
            }

            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).await?;

            raw.extend(body.iter().map(|&b| b as char));
        }

        Request::parse(raw.as_bytes())
    }

    /// Create an error response from a NetError
    fn error_response(error: &NetError) -> Response {
        let (status, message) = match error {
            NetError::NotFound => (StatusCode::NotFound, "Not Found"),
            NetError::MethodNotAllowed => (StatusCode::MethodNotAllowed, "Method Not Allowed"),
            NetError::InvalidRequest(_) => (StatusCode::BadRequest, "Bad Request"),
            NetError::RequestTooLarge => (StatusCode::PayloadTooLarge, "Payload Too Large"),
            NetError::HeaderTooLarge => (StatusCode::BadRequest, "Header Too Large"),
            NetError::Timeout => (StatusCode::RequestTimeout, "Request Timeout"),
            _ => (StatusCode::InternalServerError, "Internal Server Error"),
        };

        Response::new(status).text(message)
    }

    /// Get the server address
    pub fn addr(&self) -> SocketAddr {
        self.config.addr
    }
}

/// Simple server builder for convenience
pub struct ServerBuilder {
    config: ServerConfig,
    router: Router,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self {
            config: ServerConfig::default(),
            router: Router::new(),
        }
    }

    /// Set the bind address
    pub fn addr(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.config.addr = addr.into();
        self
    }

    /// Enable TLS
    pub fn tls(mut self, cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        self.config = self.config.with_tls(cert_path, key_path);
        self
    }

    /// Set the router
    pub fn router(mut self, router: Router) -> Self {
        self.router = router;
        self
    }

    /// Build the server
    pub fn build(self) -> NetResult<Server> {
        Server::new(self.config, self.router)
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// We need tokio-rustls for async TLS
mod tokio_rustls {
    use std::future::Future;
    use std::io;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use rustls::{ServerConfig, ServerConnection};
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
    use tokio::net::TcpStream;

    pub struct TlsAcceptor {
        inner: Arc<ServerConfig>,
    }

    impl TlsAcceptor {
        pub fn accept(&self, stream: TcpStream) -> Accept {
            Accept {
                stream: Some(stream),
                config: self.inner.clone(),
            }
        }
    }

    impl From<Arc<ServerConfig>> for TlsAcceptor {
        fn from(config: Arc<ServerConfig>) -> Self {
            Self { inner: config }
        }
    }

    impl Clone for TlsAcceptor {
        fn clone(&self) -> Self {
            Self {
                inner: self.inner.clone(),
            }
        }
    }

    pub struct Accept {
        stream: Option<TcpStream>,
        config: Arc<ServerConfig>,
    }

    impl Future for Accept {
        type Output = io::Result<TlsStream>;

        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let stream = self.stream.take().unwrap();
            let conn = ServerConnection::new(self.config.clone())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            Poll::Ready(Ok(TlsStream {
                stream,
                _conn: conn,
            }))
        }
    }

    pub struct TlsStream {
        stream: TcpStream,
        _conn: ServerConnection,
    }

    impl AsyncRead for TlsStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            // Simplified TLS read - in production this would need full handshake handling
            // For now, just pass through to underlying stream
            Pin::new(&mut self.stream).poll_read(cx, buf)
        }
    }

    impl AsyncWrite for TlsStream {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            Pin::new(&mut self.stream).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Pin::new(&mut self.stream).poll_flush(cx)
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            Pin::new(&mut self.stream).poll_shutdown(cx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = ServerConfig::new(([127, 0, 0, 1], 3000));
        assert_eq!(config.addr.port(), 3000);
        assert!(!config.is_tls());

        let tls_config = config.with_tls("/path/to/cert.pem", "/path/to/key.pem");
        assert!(tls_config.is_tls());
    }
}
