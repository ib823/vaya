//! HTTP client request building

use std::collections::HashMap;

use crate::url::Url;

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl Method {
    /// Get method as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Head => "HEAD",
            Method::Options => "OPTIONS",
        }
    }
}

/// HTTP request headers
#[derive(Debug, Clone, Default)]
pub struct Headers {
    inner: HashMap<String, String>,
}

impl Headers {
    /// Create empty headers
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Set a header (overwrites existing)
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into().to_lowercase();
        self.inner.insert(name, value.into());
    }

    /// Get a header value
    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner.get(&name.to_lowercase()).map(|s| s.as_str())
    }

    /// Remove a header
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.inner.remove(&name.to_lowercase())
    }

    /// Check if header exists
    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(&name.to_lowercase())
    }

    /// Iterate over headers
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method
    pub method: Method,
    /// Request URL
    pub url: Url,
    /// Request headers
    pub headers: Headers,
    /// Request body
    pub body: Option<Vec<u8>>,
    /// Request timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
}

impl Request {
    /// Create a new GET request
    pub fn get(url: Url) -> Self {
        Self {
            method: Method::Get,
            url,
            headers: Headers::new(),
            body: None,
            timeout_ms: Some(30_000),
            max_redirects: 5,
        }
    }

    /// Create a new POST request
    pub fn post(url: Url) -> Self {
        Self {
            method: Method::Post,
            url,
            headers: Headers::new(),
            body: None,
            timeout_ms: Some(30_000),
            max_redirects: 5,
        }
    }

    /// Create a new PUT request
    pub fn put(url: Url) -> Self {
        Self {
            method: Method::Put,
            url,
            headers: Headers::new(),
            body: None,
            timeout_ms: Some(30_000),
            max_redirects: 5,
        }
    }

    /// Create a new DELETE request
    pub fn delete(url: Url) -> Self {
        Self {
            method: Method::Delete,
            url,
            headers: Headers::new(),
            body: None,
            timeout_ms: Some(30_000),
            max_redirects: 5,
        }
    }

    /// Set a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.set(name, value);
        self
    }

    /// Set request body as bytes
    pub fn body_bytes(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    /// Set request body as string
    pub fn body_string(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into().into_bytes());
        self
    }

    /// Set request body as JSON
    pub fn body_json(self, json: &str) -> Self {
        self.header("Content-Type", "application/json")
            .body_string(json)
    }

    /// Set request body as form data
    pub fn body_form(self, params: &[(&str, &str)]) -> Self {
        let body = crate::url::build_query(params);
        self.header("Content-Type", "application/x-www-form-urlencoded")
            .body_string(body)
    }

    /// Set timeout in milliseconds
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set maximum redirects to follow
    pub fn max_redirects(mut self, count: u32) -> Self {
        self.max_redirects = count;
        self
    }

    /// Set bearer token authorization
    pub fn bearer_auth(self, token: &str) -> Self {
        self.header("Authorization", format!("Bearer {}", token))
    }

    /// Set basic authorization
    pub fn basic_auth(self, username: &str, password: &str) -> Self {
        let credentials = format!("{}:{}", username, password);
        let encoded = base64_encode(credentials.as_bytes());
        self.header("Authorization", format!("Basic {}", encoded))
    }

    /// Set user agent
    pub fn user_agent(self, agent: &str) -> Self {
        self.header("User-Agent", agent)
    }

    /// Set accept header
    pub fn accept(self, content_type: &str) -> Self {
        self.header("Accept", content_type)
    }

    /// Build the HTTP request bytes
    pub fn build(&self) -> Vec<u8> {
        let mut request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\n",
            self.method.as_str(),
            self.url.request_path(),
            self.url.host_port()
        );

        // Add headers
        for (name, value) in self.headers.iter() {
            request.push_str(&format!("{}: {}\r\n", name, value));
        }

        // Add content length if body present
        if let Some(body) = &self.body {
            if !self.headers.contains("Content-Length") {
                request.push_str(&format!("Content-Length: {}\r\n", body.len()));
            }
        }

        // Add connection header
        if !self.headers.contains("Connection") {
            request.push_str("Connection: close\r\n");
        }

        // End headers
        request.push_str("\r\n");

        let mut bytes = request.into_bytes();

        // Add body
        if let Some(body) = &self.body {
            bytes.extend_from_slice(body);
        }

        bytes
    }
}

/// Simple base64 encoder (no padding, standard alphabet)
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[(n >> 18) & 0x3F] as char);
        result.push(ALPHABET[(n >> 12) & 0x3F] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[(n >> 6) & 0x3F] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[n & 0x3F] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// Request builder for fluent API
pub struct RequestBuilder {
    method: Method,
    url: String,
    headers: Headers,
    body: Option<Vec<u8>>,
    timeout_ms: Option<u64>,
    max_redirects: u32,
}

impl RequestBuilder {
    /// Create a new request builder
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: Headers::new(),
            body: None,
            timeout_ms: Some(30_000),
            max_redirects: 5,
        }
    }

    /// Create GET request builder
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(Method::Get, url)
    }

    /// Create POST request builder
    pub fn post(url: impl Into<String>) -> Self {
        Self::new(Method::Post, url)
    }

    /// Set a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.set(name, value);
        self
    }

    /// Set body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set JSON body
    pub fn json(mut self, json: &str) -> Self {
        self.headers.set("Content-Type", "application/json");
        self.body = Some(json.as_bytes().to_vec());
        self
    }

    /// Set timeout
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Build the request
    pub fn build(self) -> crate::CollectResult<Request> {
        let url = Url::parse(&self.url)?;
        Ok(Request {
            method: self.method,
            url,
            headers: self.headers,
            body: self.body,
            timeout_ms: self.timeout_ms,
            max_redirects: self.max_redirects,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_get() {
        let url = Url::parse("https://example.com/api").unwrap();
        let req = Request::get(url)
            .header("X-Custom", "value")
            .user_agent("vaya-collect/0.1");

        let bytes = req.build();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.starts_with("GET /api HTTP/1.1\r\n"));
        assert!(text.contains("Host: example.com\r\n"));
        assert!(text.contains("x-custom: value\r\n"));
        assert!(text.contains("user-agent: vaya-collect/0.1\r\n"));
    }

    #[test]
    fn test_request_post_json() {
        let url = Url::parse("https://api.example.com/data").unwrap();
        let req = Request::post(url).body_json(r#"{"key": "value"}"#);

        let bytes = req.build();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.starts_with("POST /data HTTP/1.1\r\n"));
        assert!(text.contains("content-type: application/json\r\n"));
        assert!(text.ends_with(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b"user:pass"), "dXNlcjpwYXNz");
    }

    #[test]
    fn test_request_builder() {
        let req = RequestBuilder::get("https://example.com/api")
            .header("Accept", "application/json")
            .timeout(5000)
            .build()
            .unwrap();

        assert_eq!(req.method, Method::Get);
        assert_eq!(req.url.host, "example.com");
        assert_eq!(req.timeout_ms, Some(5000));
    }
}
