//! HTTP response building and serialization

use crate::http::{Headers, StatusCode, Version};

/// An HTTP response
#[derive(Debug, Clone)]
pub struct Response {
    /// HTTP version
    version: Version,
    /// Status code
    status: StatusCode,
    /// Response headers
    headers: Headers,
    /// Response body
    body: Vec<u8>,
}

impl Response {
    /// Create a new response with the given status
    pub fn new(status: StatusCode) -> Self {
        Self {
            version: Version::Http11,
            status,
            headers: Headers::new(),
            body: Vec::new(),
        }
    }

    /// Create an OK (200) response
    pub fn ok() -> Self {
        Self::new(StatusCode::Ok)
    }

    /// Create a Created (201) response
    pub fn created() -> Self {
        Self::new(StatusCode::Created)
    }

    /// Create a No Content (204) response
    pub fn no_content() -> Self {
        Self::new(StatusCode::NoContent)
    }

    /// Create a Bad Request (400) response
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BadRequest)
    }

    /// Create an Unauthorized (401) response
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::Unauthorized)
    }

    /// Create a Forbidden (403) response
    pub fn forbidden() -> Self {
        Self::new(StatusCode::Forbidden)
    }

    /// Create a Not Found (404) response
    pub fn not_found() -> Self {
        Self::new(StatusCode::NotFound)
    }

    /// Create an Internal Server Error (500) response
    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::InternalServerError)
    }

    /// Get the status code
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Set the status code
    pub fn set_status(&mut self, status: StatusCode) {
        self.status = status;
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

    /// Set a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.set(name, value);
        self
    }

    /// Set the body
    pub fn body_bytes(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set text body
    pub fn text(self, body: impl Into<String>) -> Self {
        self.header("Content-Type", "text/plain; charset=utf-8")
            .body_bytes(body.into().into_bytes())
    }

    /// Set HTML body
    pub fn html(self, body: impl Into<String>) -> Self {
        self.header("Content-Type", "text/html; charset=utf-8")
            .body_bytes(body.into().into_bytes())
    }

    /// Set JSON body
    pub fn json(self, body: impl Into<String>) -> Self {
        self.header("Content-Type", "application/json")
            .body_bytes(body.into().into_bytes())
    }

    /// Set a redirect response
    pub fn redirect(location: &str, permanent: bool) -> Self {
        let status = if permanent {
            StatusCode::PermanentRedirect
        } else {
            StatusCode::TemporaryRedirect
        };
        Self::new(status).header("Location", location)
    }

    /// Serialize the response to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Status line
        result.extend_from_slice(
            format!(
                "{} {} {}\r\n",
                self.version,
                self.status.code(),
                self.status.reason()
            )
            .as_bytes(),
        );

        // Headers
        for (name, value) in self.headers.iter() {
            result.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }

        // Content-Length if not already set
        if self.headers.get("content-length").is_none() {
            result.extend_from_slice(format!("Content-Length: {}\r\n", self.body.len()).as_bytes());
        }

        // End of headers
        result.extend_from_slice(b"\r\n");

        // Body
        result.extend_from_slice(&self.body);

        result
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::ok()
    }
}

/// Builder for responses
pub struct ResponseBuilder {
    response: Response,
}

impl ResponseBuilder {
    /// Create a new response builder
    pub fn new(status: StatusCode) -> Self {
        Self {
            response: Response::new(status),
        }
    }

    /// Set a header
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.response.headers.set(name, value);
        self
    }

    /// Set the body
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.response.body = body.into();
        self
    }

    /// Build the response
    pub fn build(self) -> Response {
        self.response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_response() {
        let res = Response::ok().text("Hello, World!");
        let bytes = res.to_bytes();
        let text = String::from_utf8_lossy(&bytes);

        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("content-type: text/plain"));
        assert!(text.ends_with("Hello, World!"));
    }

    #[test]
    fn test_json_response() {
        let res = Response::ok().json("{\"status\":\"ok\"}");
        let bytes = res.to_bytes();
        let text = String::from_utf8_lossy(&bytes);

        assert!(text.contains("content-type: application/json"));
        assert!(text.ends_with("{\"status\":\"ok\"}"));
    }

    #[test]
    fn test_redirect() {
        let res = Response::redirect("/new-location", false);
        assert_eq!(res.status(), StatusCode::TemporaryRedirect);
        assert_eq!(res.headers().get("location"), Some("/new-location"));
    }
}
