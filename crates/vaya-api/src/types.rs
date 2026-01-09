//! API request and response types

use std::collections::HashMap;

/// HTTP Request
#[derive(Debug, Clone)]
pub struct Request {
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// Query string parameters
    pub query_params: HashMap<String, String>,
    /// Path parameters (extracted from route)
    pub path_params: HashMap<String, String>,
    /// Request headers
    pub headers: HashMap<String, String>,
    /// Request body
    pub body: Vec<u8>,
    /// Client IP
    pub client_ip: Option<String>,
    /// Request ID (for tracing)
    pub request_id: String,
    /// User ID (set by auth middleware)
    pub user_id: Option<String>,
    /// User roles (set by auth middleware)
    pub user_roles: Vec<String>,
}

impl Request {
    /// Create a new request
    pub fn new(method: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            query_params: HashMap::new(),
            path_params: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            client_ip: None,
            request_id: generate_request_id(),
            user_id: None,
            user_roles: Vec::new(),
        }
    }

    /// Get query parameter
    pub fn query(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }

    /// Get path parameter
    pub fn param(&self, key: &str) -> Option<&String> {
        self.path_params.get(key)
    }

    /// Get header (case-insensitive)
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(&key.to_lowercase())
    }

    /// Get body as string
    pub fn body_string(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }

    /// Check if request is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    /// Check if user has role
    pub fn has_role(&self, role: &str) -> bool {
        self.user_roles.iter().any(|r| r == role)
    }

    /// Get Content-Type header
    pub fn content_type(&self) -> Option<&String> {
        self.header("content-type")
    }

    /// Check if content type is JSON
    pub fn is_json(&self) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    /// Get Authorization token
    pub fn auth_token(&self) -> Option<&str> {
        self.header("authorization").and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(&auth[7..])
            } else {
                None
            }
        })
    }
}

/// HTTP Response
#[derive(Debug, Clone)]
pub struct Response {
    /// Status code
    pub status: u16,
    /// Status text
    pub status_text: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: Vec<u8>,
}

impl Response {
    /// Create a new response
    pub fn new(status: u16, status_text: impl Into<String>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "application/json".into());

        Self {
            status,
            status_text: status_text.into(),
            headers,
            body: Vec::new(),
        }
    }

    /// Create 200 OK response
    pub fn ok() -> Self {
        Self::new(200, "OK")
    }

    /// Create 201 Created response
    pub fn created() -> Self {
        Self::new(201, "Created")
    }

    /// Create 204 No Content response
    pub fn no_content() -> Self {
        Self::new(204, "No Content")
    }

    /// Create 400 Bad Request response
    pub fn bad_request(message: &str) -> Self {
        let mut resp = Self::new(400, "Bad Request");
        resp.set_json_body(&ErrorBody {
            error: "bad_request".into(),
            message: message.into(),
            code: 400,
        });
        resp
    }

    /// Create 401 Unauthorized response
    pub fn unauthorized(message: &str) -> Self {
        let mut resp = Self::new(401, "Unauthorized");
        resp.set_json_body(&ErrorBody {
            error: "unauthorized".into(),
            message: message.into(),
            code: 401,
        });
        resp
    }

    /// Create 403 Forbidden response
    pub fn forbidden(message: &str) -> Self {
        let mut resp = Self::new(403, "Forbidden");
        resp.set_json_body(&ErrorBody {
            error: "forbidden".into(),
            message: message.into(),
            code: 403,
        });
        resp
    }

    /// Create 404 Not Found response
    pub fn not_found(message: &str) -> Self {
        let mut resp = Self::new(404, "Not Found");
        resp.set_json_body(&ErrorBody {
            error: "not_found".into(),
            message: message.into(),
            code: 404,
        });
        resp
    }

    /// Create 429 Too Many Requests response
    pub fn too_many_requests(message: &str) -> Self {
        let mut resp = Self::new(429, "Too Many Requests");
        resp.set_json_body(&ErrorBody {
            error: "rate_limited".into(),
            message: message.into(),
            code: 429,
        });
        resp
    }

    /// Create 500 Internal Server Error response
    pub fn internal_error(message: &str) -> Self {
        let mut resp = Self::new(500, "Internal Server Error");
        resp.set_json_body(&ErrorBody {
            error: "internal_error".into(),
            message: message.into(),
            code: 500,
        });
        resp
    }

    /// Set response body
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    /// Set JSON body (manual serialization for zero-dep)
    pub fn set_json_body<T: JsonSerialize>(&mut self, body: &T) {
        self.body = body.to_json().into_bytes();
        self.headers.insert("content-type".into(), "application/json".into());
    }

    /// Set header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into().to_lowercase(), value.into());
        self
    }

    /// Get body as string
    pub fn body_string(&self) -> Option<String> {
        String::from_utf8(self.body.clone()).ok()
    }
}

/// Error response body
#[derive(Debug, Clone)]
pub struct ErrorBody {
    pub error: String,
    pub message: String,
    pub code: u16,
}

/// Trait for JSON serialization (manual, zero-dep)
pub trait JsonSerialize {
    fn to_json(&self) -> String;
}

impl JsonSerialize for ErrorBody {
    fn to_json(&self) -> String {
        format!(
            r#"{{"error":"{}","message":"{}","code":{}}}"#,
            escape_json(&self.error),
            escape_json(&self.message),
            self.code
        )
    }
}

/// Generic success response
#[derive(Debug, Clone)]
pub struct SuccessBody<T> {
    pub data: T,
}

impl<T: JsonSerialize> JsonSerialize for SuccessBody<T> {
    fn to_json(&self) -> String {
        format!(r#"{{"data":{}}}"#, self.data.to_json())
    }
}

/// Paginated response
#[derive(Debug, Clone)]
pub struct PaginatedBody<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

impl<T: JsonSerialize> JsonSerialize for PaginatedBody<T> {
    fn to_json(&self) -> String {
        let items: Vec<String> = self.data.iter().map(|d| d.to_json()).collect();
        format!(
            r#"{{"data":[{}],"total":{},"page":{},"page_size":{},"has_more":{}}}"#,
            items.join(","),
            self.total,
            self.page,
            self.page_size,
            self.has_more
        )
    }
}

/// Escape JSON string
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Generate unique request ID
fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    format!("req-{:x}", timestamp)
}

/// Parse query string into HashMap
pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();

    for pair in query.split('&') {
        if let Some(eq_pos) = pair.find('=') {
            let key = &pair[..eq_pos];
            let value = &pair[eq_pos + 1..];
            params.insert(
                url_decode(key),
                url_decode(value),
            );
        }
    }

    params
}

/// Basic URL decode
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = Request::new("GET", "/api/users");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/api/users");
        assert!(!req.is_authenticated());
    }

    #[test]
    fn test_request_params() {
        let mut req = Request::new("GET", "/api/users/123");
        req.path_params.insert("id".into(), "123".into());
        req.query_params.insert("page".into(), "1".into());

        assert_eq!(req.param("id"), Some(&"123".to_string()));
        assert_eq!(req.query("page"), Some(&"1".to_string()));
    }

    #[test]
    fn test_auth_token() {
        let mut req = Request::new("GET", "/api");
        req.headers.insert("authorization".into(), "Bearer abc123".into());

        assert_eq!(req.auth_token(), Some("abc123"));
    }

    #[test]
    fn test_response_ok() {
        let resp = Response::ok();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.status_text, "OK");
    }

    #[test]
    fn test_response_error() {
        let resp = Response::not_found("User not found");
        assert_eq!(resp.status, 404);
        assert!(resp.body_string().unwrap().contains("not_found"));
    }

    #[test]
    fn test_json_escape() {
        assert_eq!(escape_json(r#"Hello "World""#), r#"Hello \"World\""#);
        assert_eq!(escape_json("Line1\nLine2"), "Line1\\nLine2");
    }

    #[test]
    fn test_error_body_json() {
        let body = ErrorBody {
            error: "test".into(),
            message: "Test message".into(),
            code: 400,
        };

        let json = body.to_json();
        assert!(json.contains(r#""error":"test""#));
        assert!(json.contains(r#""code":400"#));
    }

    #[test]
    fn test_query_string_parsing() {
        let params = parse_query_string("name=John&age=30");
        assert_eq!(params.get("name"), Some(&"John".to_string()));
        assert_eq!(params.get("age"), Some(&"30".to_string()));
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("Hello%20World"), "Hello World");
        assert_eq!(url_decode("a+b"), "a b");
    }

    #[test]
    fn test_paginated_body() {
        let body = PaginatedBody {
            data: vec![ErrorBody { error: "e1".into(), message: "m1".into(), code: 1 }],
            total: 100,
            page: 1,
            page_size: 10,
            has_more: true,
        };

        let json = body.to_json();
        assert!(json.contains(r#""total":100"#));
        assert!(json.contains(r#""has_more":true"#));
    }
}
