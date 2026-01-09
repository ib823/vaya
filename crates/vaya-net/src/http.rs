//! HTTP protocol types

use std::fmt;
use std::str::FromStr;

use crate::NetError;

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl Method {
    /// Returns the method as a string slice
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
            Method::CONNECT => "CONNECT",
            Method::TRACE => "TRACE",
        }
    }

    /// Check if the method allows a request body
    pub fn allows_body(&self) -> bool {
        matches!(self, Method::POST | Method::PUT | Method::PATCH)
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Method {
    type Err = NetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "PATCH" => Ok(Method::PATCH),
            "HEAD" => Ok(Method::HEAD),
            "OPTIONS" => Ok(Method::OPTIONS),
            "CONNECT" => Ok(Method::CONNECT),
            "TRACE" => Ok(Method::TRACE),
            _ => Err(NetError::InvalidRequest(format!("Unknown method: {}", s))),
        }
    }
}

/// HTTP versions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    Http10,
    Http11,
}

impl Version {
    /// Returns the version as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Version {
    type Err = NetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "HTTP/1.0" => Ok(Version::Http10),
            "HTTP/1.1" => Ok(Version::Http11),
            _ => Err(NetError::InvalidRequest(format!(
                "Unsupported HTTP version: {}",
                s
            ))),
        }
    }
}

/// HTTP status codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    // 1xx Informational
    Continue = 100,
    SwitchingProtocols = 101,

    // 2xx Success
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,

    // 3xx Redirection
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // 4xx Client Errors
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PayloadTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    UnprocessableEntity = 422,
    TooManyRequests = 429,

    // 5xx Server Errors
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
}

impl StatusCode {
    /// Get the numeric status code
    pub fn code(&self) -> u16 {
        *self as u16
    }

    /// Get the reason phrase for this status
    pub fn reason(&self) -> &'static str {
        match self {
            StatusCode::Continue => "Continue",
            StatusCode::SwitchingProtocols => "Switching Protocols",
            StatusCode::Ok => "OK",
            StatusCode::Created => "Created",
            StatusCode::Accepted => "Accepted",
            StatusCode::NoContent => "No Content",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::Found => "Found",
            StatusCode::SeeOther => "See Other",
            StatusCode::NotModified => "Not Modified",
            StatusCode::TemporaryRedirect => "Temporary Redirect",
            StatusCode::PermanentRedirect => "Permanent Redirect",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::NotAcceptable => "Not Acceptable",
            StatusCode::RequestTimeout => "Request Timeout",
            StatusCode::Conflict => "Conflict",
            StatusCode::Gone => "Gone",
            StatusCode::LengthRequired => "Length Required",
            StatusCode::PayloadTooLarge => "Payload Too Large",
            StatusCode::UriTooLong => "URI Too Long",
            StatusCode::UnsupportedMediaType => "Unsupported Media Type",
            StatusCode::UnprocessableEntity => "Unprocessable Entity",
            StatusCode::TooManyRequests => "Too Many Requests",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
            StatusCode::GatewayTimeout => "Gateway Timeout",
        }
    }

    /// Check if this is a success status (2xx)
    pub fn is_success(&self) -> bool {
        let code = self.code();
        (200..300).contains(&code)
    }

    /// Check if this is a redirect status (3xx)
    pub fn is_redirect(&self) -> bool {
        let code = self.code();
        (300..400).contains(&code)
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        let code = self.code();
        (400..500).contains(&code)
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        let code = self.code();
        (500..600).contains(&code)
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code(), self.reason())
    }
}

/// HTTP headers collection
#[derive(Debug, Clone, Default)]
pub struct Headers {
    headers: Vec<(String, String)>,
}

impl Headers {
    /// Create a new empty headers collection
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    /// Add a header (allows duplicates)
    pub fn append(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers
            .push((name.into().to_lowercase(), value.into()));
    }

    /// Set a header (replaces existing)
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into().to_lowercase();
        self.headers.retain(|(n, _)| n != &name);
        self.headers.push((name, value.into()));
    }

    /// Get the first value for a header
    pub fn get(&self, name: &str) -> Option<&str> {
        let name = name.to_lowercase();
        self.headers
            .iter()
            .find(|(n, _)| n == &name)
            .map(|(_, v)| v.as_str())
    }

    /// Get all values for a header
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        let name = name.to_lowercase();
        self.headers
            .iter()
            .filter(|(n, _)| n == &name)
            .map(|(_, v)| v.as_str())
            .collect()
    }

    /// Remove a header
    pub fn remove(&mut self, name: &str) {
        let name = name.to_lowercase();
        self.headers.retain(|(n, _)| n != &name);
    }

    /// Check if a header exists
    pub fn contains(&self, name: &str) -> bool {
        let name = name.to_lowercase();
        self.headers.iter().any(|(n, _)| n == &name)
    }

    /// Iterate over all headers
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.iter().map(|(n, v)| (n.as_str(), v.as_str()))
    }

    /// Get the number of headers
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Check if headers are empty
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    /// Parse Content-Length header
    pub fn content_length(&self) -> Option<usize> {
        self.get("content-length")?.parse().ok()
    }

    /// Check if connection should be kept alive
    pub fn is_keep_alive(&self) -> bool {
        self.get("connection")
            .map(|v| v.to_lowercase() != "close")
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_parsing() {
        assert_eq!(Method::from_str("GET").unwrap(), Method::GET);
        assert_eq!(Method::from_str("post").unwrap(), Method::POST);
        assert!(Method::from_str("INVALID").is_err());
    }

    #[test]
    fn test_version_parsing() {
        assert_eq!(Version::from_str("HTTP/1.1").unwrap(), Version::Http11);
        assert_eq!(Version::from_str("HTTP/1.0").unwrap(), Version::Http10);
        assert!(Version::from_str("HTTP/2.0").is_err());
    }

    #[test]
    fn test_status_code() {
        assert_eq!(StatusCode::Ok.code(), 200);
        assert!(StatusCode::Ok.is_success());
        assert!(StatusCode::NotFound.is_client_error());
        assert!(StatusCode::InternalServerError.is_server_error());
    }

    #[test]
    fn test_headers() {
        let mut headers = Headers::new();
        headers.set("Content-Type", "application/json");
        headers.append("Accept", "text/html");
        headers.append("Accept", "application/json");

        assert_eq!(headers.get("content-type"), Some("application/json"));
        assert_eq!(headers.get_all("accept").len(), 2);
        assert!(headers.contains("accept"));
    }
}
