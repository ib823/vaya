//! HTTP request parsing and representation

use std::collections::HashMap;
use std::str::FromStr;

use crate::http::{Headers, Method, Version};
use crate::{NetError, NetResult, MAX_BODY_SIZE, MAX_HEADER_SIZE};

/// An HTTP request
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method
    method: Method,
    /// Request path (e.g., "/api/users")
    path: String,
    /// Query string parameters
    query: HashMap<String, String>,
    /// HTTP version
    version: Version,
    /// Request headers
    headers: Headers,
    /// Request body
    body: Vec<u8>,
    /// Path parameters (from router)
    params: HashMap<String, String>,
}

impl Request {
    /// Create a new request
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        let path_str = path.into();
        let (path, query) = Self::parse_path_and_query(&path_str);

        Self {
            method,
            path,
            query,
            version: Version::Http11,
            headers: Headers::new(),
            body: Vec::new(),
            params: HashMap::new(),
        }
    }

    /// Parse raw HTTP request bytes
    pub fn parse(data: &[u8]) -> NetResult<Self> {
        // Find header/body separator
        let header_end = Self::find_header_end(data).ok_or_else(|| {
            NetError::InvalidRequest("Incomplete headers".into())
        })?;

        if header_end > MAX_HEADER_SIZE {
            return Err(NetError::HeaderTooLarge);
        }

        let header_bytes = &data[..header_end];
        let header_str = std::str::from_utf8(header_bytes)
            .map_err(|e| NetError::InvalidRequest(format!("Invalid UTF-8 in headers: {}", e)))?;

        // Parse request line and headers
        let mut lines = header_str.lines();
        let request_line = lines
            .next()
            .ok_or_else(|| NetError::InvalidRequest("Missing request line".into()))?;

        let (method, path, version) = Self::parse_request_line(request_line)?;
        let (path, query) = Self::parse_path_and_query(&path);

        // Parse headers
        let mut headers = Headers::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            let (name, value) = Self::parse_header_line(line)?;
            headers.append(name, value);
        }

        // Parse body if Content-Length is present
        let body = if let Some(content_length) = headers.content_length() {
            if content_length > MAX_BODY_SIZE {
                return Err(NetError::RequestTooLarge);
            }
            let body_start = header_end + 4; // Skip \r\n\r\n
            if data.len() >= body_start + content_length {
                data[body_start..body_start + content_length].to_vec()
            } else {
                return Err(NetError::InvalidRequest("Incomplete body".into()));
            }
        } else {
            Vec::new()
        };

        Ok(Self {
            method,
            path,
            query,
            version,
            headers,
            body,
            params: HashMap::new(),
        })
    }

    /// Find the end of headers (\r\n\r\n)
    fn find_header_end(data: &[u8]) -> Option<usize> {
        for i in 0..data.len().saturating_sub(3) {
            if &data[i..i + 4] == b"\r\n\r\n" {
                return Some(i);
            }
        }
        None
    }

    /// Parse the request line (e.g., "GET /path HTTP/1.1")
    fn parse_request_line(line: &str) -> NetResult<(Method, String, Version)> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(NetError::InvalidRequest(format!(
                "Invalid request line: {}",
                line
            )));
        }

        let method = Method::from_str(parts[0])?;
        let path = parts[1].to_string();
        let version = Version::from_str(parts[2])?;

        Ok((method, path, version))
    }

    /// Parse path and query string
    fn parse_path_and_query(path: &str) -> (String, HashMap<String, String>) {
        if let Some(idx) = path.find('?') {
            let path_part = path[..idx].to_string();
            let query_str = &path[idx + 1..];
            let query = Self::parse_query_string(query_str);
            (path_part, query)
        } else {
            (path.to_string(), HashMap::new())
        }
    }

    /// Parse query string into key-value pairs
    fn parse_query_string(query: &str) -> HashMap<String, String> {
        query
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?.to_string();
                let value = parts.next().unwrap_or("").to_string();
                Some((Self::url_decode(&key), Self::url_decode(&value)))
            })
            .collect()
    }

    /// Simple URL decoding
    fn url_decode(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                result.push('%');
                result.push_str(&hex);
            } else if c == '+' {
                result.push(' ');
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Parse a header line (e.g., "Content-Type: application/json")
    fn parse_header_line(line: &str) -> NetResult<(String, String)> {
        let idx = line.find(':').ok_or_else(|| {
            NetError::InvalidRequest(format!("Invalid header line: {}", line))
        })?;

        let name = line[..idx].trim().to_string();
        let value = line[idx + 1..].trim().to_string();

        Ok((name, value))
    }

    /// Get the HTTP method
    pub fn method(&self) -> Method {
        self.method
    }

    /// Get the request path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the HTTP version
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get the headers
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get mutable headers
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Get the body
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Get the body as a string
    pub fn body_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }

    /// Set the body
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
    }

    /// Get a query parameter
    pub fn query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }

    /// Get all query parameters
    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query
    }

    /// Get a path parameter (set by router)
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    /// Set path parameters (called by router)
    pub fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    /// Get the Content-Type header
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type")
    }

    /// Check if this is a JSON request
    pub fn is_json(&self) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    /// Check if this is a WebSocket upgrade request
    pub fn is_websocket_upgrade(&self) -> bool {
        let upgrade = self.headers.get("upgrade").map(|v| v.to_lowercase());
        let connection = self.headers.get("connection").map(|v| v.to_lowercase());

        upgrade == Some("websocket".into())
            && connection.map(|c| c.contains("upgrade")).unwrap_or(false)
    }

    /// Get the WebSocket key for handshake
    pub fn websocket_key(&self) -> Option<&str> {
        self.headers.get("sec-websocket-key")
    }

    /// Serialize the request to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Request line
        result.extend_from_slice(
            format!("{} {} {}\r\n", self.method, self.path, self.version).as_bytes(),
        );

        // Headers
        for (name, value) in self.headers.iter() {
            result.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }

        // Content-Length if body present
        if !self.body.is_empty() && self.headers.get("content-length").is_none() {
            result.extend_from_slice(format!("Content-Length: {}\r\n", self.body.len()).as_bytes());
        }

        // End of headers
        result.extend_from_slice(b"\r\n");

        // Body
        result.extend_from_slice(&self.body);

        result
    }
}

/// Builder for constructing requests
pub struct RequestBuilder {
    request: Request,
}

impl RequestBuilder {
    /// Create a new request builder
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        Self {
            request: Request::new(method, path),
        }
    }

    /// Add a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.request.headers.set(name, value);
        self
    }

    /// Set the body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.request.body = body.into();
        self
    }

    /// Set JSON body
    pub fn json(self, body: impl Into<String>) -> Self {
        self.header("Content-Type", "application/json")
            .body(body.into().into_bytes())
    }

    /// Build the request
    pub fn build(self) -> Request {
        self.request
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_request() {
        let raw = b"GET /hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = Request::parse(raw).unwrap();

        assert_eq!(req.method(), Method::GET);
        assert_eq!(req.path(), "/hello");
        assert_eq!(req.version(), Version::Http11);
        assert_eq!(req.headers().get("host"), Some("localhost"));
    }

    #[test]
    fn test_parse_request_with_query() {
        let raw = b"GET /search?q=hello&limit=10 HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = Request::parse(raw).unwrap();

        assert_eq!(req.path(), "/search");
        assert_eq!(req.query("q"), Some("hello"));
        assert_eq!(req.query("limit"), Some("10"));
    }

    #[test]
    fn test_parse_request_with_body() {
        let raw = b"POST /api/data HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 13\r\n\r\n{\"key\":\"val\"}";
        let req = Request::parse(raw).unwrap();

        assert_eq!(req.method(), Method::POST);
        assert_eq!(req.body_str(), Some("{\"key\":\"val\"}"));
        assert!(req.is_json());
    }

    #[test]
    fn test_request_builder() {
        let req = RequestBuilder::new(Method::POST, "/api/users")
            .header("Authorization", "Bearer token")
            .json("{\"name\":\"test\"}")
            .build();

        assert_eq!(req.method(), Method::POST);
        assert!(req.is_json());
    }
}
