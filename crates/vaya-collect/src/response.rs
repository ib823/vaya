//! HTTP client response parsing

use std::collections::HashMap;

use crate::{CollectError, CollectResult};

/// HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP status code
    pub status: u16,
    /// Status reason phrase
    pub reason: String,
    /// Response headers
    pub headers: ResponseHeaders,
    /// Response body
    pub body: Vec<u8>,
}

/// Response headers
#[derive(Debug, Clone, Default)]
pub struct ResponseHeaders {
    inner: HashMap<String, Vec<String>>,
}

impl ResponseHeaders {
    /// Create empty headers
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Add a header value (appends to existing)
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into().to_lowercase();
        self.inner
            .entry(name)
            .or_insert_with(Vec::new)
            .push(value.into());
    }

    /// Get first header value
    pub fn get(&self, name: &str) -> Option<&str> {
        self.inner
            .get(&name.to_lowercase())
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    /// Get all values for a header
    pub fn get_all(&self, name: &str) -> Option<&[String]> {
        self.inner.get(&name.to_lowercase()).map(|v| v.as_slice())
    }

    /// Check if header exists
    pub fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(&name.to_lowercase())
    }

    /// Iterate over headers (name, first value)
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner
            .iter()
            .filter_map(|(k, v)| v.first().map(|val| (k.as_str(), val.as_str())))
    }
}

impl Response {
    /// Parse response from bytes
    pub fn parse(data: &[u8]) -> CollectResult<Self> {
        // Find header/body separator
        let header_end = find_header_end(data)
            .ok_or_else(|| CollectError::InvalidResponse("No header separator found".into()))?;

        let header_bytes = &data[..header_end];
        let body = data[header_end + 4..].to_vec(); // Skip \r\n\r\n

        // Parse status line
        let header_str = std::str::from_utf8(header_bytes)
            .map_err(|_| CollectError::InvalidResponse("Invalid UTF-8 in headers".into()))?;

        let mut lines = header_str.lines();
        let status_line = lines
            .next()
            .ok_or_else(|| CollectError::InvalidResponse("Empty response".into()))?;

        let (status, reason) = parse_status_line(status_line)?;

        // Parse headers
        let mut headers = ResponseHeaders::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((name, value)) = parse_header_line(line) {
                headers.add(name, value);
            }
        }

        Ok(Response {
            status,
            reason,
            headers,
            body,
        })
    }

    /// Check if response is successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if response is a redirect (3xx)
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// Check if response is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if response is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Get body as string
    pub fn text(&self) -> CollectResult<String> {
        String::from_utf8(self.body.clone())
            .map_err(|e| CollectError::ParseError(format!("Invalid UTF-8: {}", e)))
    }

    /// Get content type
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type")
    }

    /// Get content length
    pub fn content_length(&self) -> Option<usize> {
        self.headers
            .get("content-length")
            .and_then(|s| s.parse().ok())
    }

    /// Get redirect location
    pub fn location(&self) -> Option<&str> {
        self.headers.get("location")
    }

    /// Convert to error if not successful
    pub fn error_for_status(self) -> CollectResult<Self> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(CollectError::HttpError(self.status, self.reason))
        }
    }
}

/// Find the end of headers (position before \r\n\r\n)
fn find_header_end(data: &[u8]) -> Option<usize> {
    for i in 0..data.len().saturating_sub(3) {
        if &data[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

/// Parse HTTP status line
fn parse_status_line(line: &str) -> CollectResult<(u16, String)> {
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(CollectError::InvalidResponse(format!(
            "Invalid status line: {}",
            line
        )));
    }

    let status: u16 = parts[1]
        .parse()
        .map_err(|_| CollectError::InvalidResponse(format!("Invalid status code: {}", parts[1])))?;

    let reason = parts.get(2).unwrap_or(&"").to_string();

    Ok((status, reason))
}

/// Parse a header line
fn parse_header_line(line: &str) -> Option<(String, String)> {
    let colon_pos = line.find(':')?;
    let name = line[..colon_pos].trim().to_string();
    let value = line[colon_pos + 1..].trim().to_string();
    Some((name, value))
}

/// Chunked transfer encoding decoder
pub struct ChunkedDecoder {
    buffer: Vec<u8>,
    complete: bool,
}

impl ChunkedDecoder {
    /// Create new decoder
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            complete: false,
        }
    }

    /// Feed data to decoder
    pub fn feed(&mut self, data: &[u8]) -> CollectResult<()> {
        if self.complete {
            return Ok(());
        }

        self.buffer.extend_from_slice(data);
        self.process()?;
        Ok(())
    }

    /// Process buffered data
    fn process(&mut self) -> CollectResult<()> {
        // This is a simplified chunked decoder
        // A full implementation would handle partial chunks
        Ok(())
    }

    /// Check if decoding is complete
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Get decoded data
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Take decoded data
    pub fn take(self) -> Vec<u8> {
        self.buffer
    }
}

impl Default for ChunkedDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response() {
        let data = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello";
        let response = Response::parse(data).unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(response.reason, "OK");
        assert!(response.is_success());
        assert_eq!(response.content_type(), Some("text/plain"));
        assert_eq!(response.content_length(), Some(5));
        assert_eq!(response.text().unwrap(), "hello");
    }

    #[test]
    fn test_parse_redirect() {
        let data = b"HTTP/1.1 301 Moved Permanently\r\nLocation: https://example.com/new\r\n\r\n";
        let response = Response::parse(data).unwrap();

        assert_eq!(response.status, 301);
        assert!(response.is_redirect());
        assert_eq!(response.location(), Some("https://example.com/new"));
    }

    #[test]
    fn test_error_for_status() {
        let data = b"HTTP/1.1 404 Not Found\r\n\r\n";
        let response = Response::parse(data).unwrap();

        assert!(response.is_client_error());
        assert!(response.error_for_status().is_err());
    }

    #[test]
    fn test_multiple_headers() {
        let data = b"HTTP/1.1 200 OK\r\nSet-Cookie: a=1\r\nSet-Cookie: b=2\r\n\r\n";
        let response = Response::parse(data).unwrap();

        let cookies = response.headers.get_all("set-cookie").unwrap();
        assert_eq!(cookies.len(), 2);
    }
}
