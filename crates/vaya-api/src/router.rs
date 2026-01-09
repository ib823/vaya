//! HTTP Router with path matching and handler dispatch

use std::collections::HashMap;

use crate::{ApiError, ApiResult, Request, Response};

/// HTTP Method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    OPTIONS,
    HEAD,
}

impl Method {
    /// Parse method from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Method::GET),
            "POST" => Some(Method::POST),
            "PUT" => Some(Method::PUT),
            "PATCH" => Some(Method::PATCH),
            "DELETE" => Some(Method::DELETE),
            "OPTIONS" => Some(Method::OPTIONS),
            "HEAD" => Some(Method::HEAD),
            _ => None,
        }
    }

    /// Get method as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::PATCH => "PATCH",
            Method::DELETE => "DELETE",
            Method::OPTIONS => "OPTIONS",
            Method::HEAD => "HEAD",
        }
    }
}

/// Route handler function type
pub type Handler = fn(&Request) -> ApiResult<Response>;

/// A route definition
#[derive(Debug, Clone)]
pub struct Route {
    /// HTTP method
    pub method: Method,
    /// Path pattern (e.g., "/api/v1/flights/:id")
    pub pattern: String,
    /// Handler function name (for debugging)
    pub handler_name: String,
    /// Path segments for matching
    segments: Vec<PathSegment>,
}

#[derive(Debug, Clone)]
enum PathSegment {
    Literal(String),
    Param(String),
    Wildcard,
}

impl Route {
    /// Create a new route
    pub fn new(method: Method, pattern: impl Into<String>, handler_name: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let segments = Self::parse_pattern(&pattern);

        Self {
            method,
            pattern,
            handler_name: handler_name.into(),
            segments,
        }
    }

    /// Parse pattern into segments
    fn parse_pattern(pattern: &str) -> Vec<PathSegment> {
        pattern
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| {
                if s.starts_with(':') {
                    PathSegment::Param(s[1..].to_string())
                } else if s == "*" {
                    PathSegment::Wildcard
                } else {
                    PathSegment::Literal(s.to_string())
                }
            })
            .collect()
    }

    /// Match a path and extract parameters
    pub fn match_path(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_parts: Vec<&str> = path
            .trim_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        if path_parts.len() != self.segments.len() {
            // Check for wildcard at end
            if let Some(PathSegment::Wildcard) = self.segments.last() {
                if path_parts.len() >= self.segments.len() - 1 {
                    // Wildcard matches remaining
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        let mut params = HashMap::new();

        for (i, segment) in self.segments.iter().enumerate() {
            match segment {
                PathSegment::Literal(expected) => {
                    if i >= path_parts.len() || path_parts[i] != expected.as_str() {
                        return None;
                    }
                }
                PathSegment::Param(name) => {
                    if i < path_parts.len() {
                        params.insert(name.clone(), path_parts[i].to_string());
                    } else {
                        return None;
                    }
                }
                PathSegment::Wildcard => {
                    // Wildcard matches rest
                    break;
                }
            }
        }

        Some(params)
    }
}

/// Router for matching requests to handlers
#[derive(Debug, Default)]
pub struct Router {
    /// Registered routes
    routes: Vec<Route>,
    /// Handler functions by route index
    handlers: HashMap<usize, Handler>,
    /// Prefix for all routes
    prefix: String,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            handlers: HashMap::new(),
            prefix: String::new(),
        }
    }

    /// Create router with prefix
    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            routes: Vec::new(),
            handlers: HashMap::new(),
            prefix: prefix.into(),
        }
    }

    /// Add a GET route
    pub fn get(&mut self, pattern: &str, handler: Handler, name: &str) {
        self.add_route(Method::GET, pattern, handler, name);
    }

    /// Add a POST route
    pub fn post(&mut self, pattern: &str, handler: Handler, name: &str) {
        self.add_route(Method::POST, pattern, handler, name);
    }

    /// Add a PUT route
    pub fn put(&mut self, pattern: &str, handler: Handler, name: &str) {
        self.add_route(Method::PUT, pattern, handler, name);
    }

    /// Add a PATCH route
    pub fn patch(&mut self, pattern: &str, handler: Handler, name: &str) {
        self.add_route(Method::PATCH, pattern, handler, name);
    }

    /// Add a DELETE route
    pub fn delete(&mut self, pattern: &str, handler: Handler, name: &str) {
        self.add_route(Method::DELETE, pattern, handler, name);
    }

    /// Add a route with any method
    fn add_route(&mut self, method: Method, pattern: &str, handler: Handler, name: &str) {
        let full_pattern = format!("{}{}", self.prefix, pattern);
        let route = Route::new(method, full_pattern, name);
        let index = self.routes.len();
        self.routes.push(route);
        self.handlers.insert(index, handler);
    }

    /// Find matching route for request
    pub fn find(&self, method: Method, path: &str) -> Option<(&Route, HashMap<String, String>, Handler)> {
        for (i, route) in self.routes.iter().enumerate() {
            if route.method != method {
                continue;
            }

            if let Some(params) = route.match_path(path) {
                if let Some(handler) = self.handlers.get(&i) {
                    return Some((route, params, *handler));
                }
            }
        }
        None
    }

    /// Get all routes for documentation
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// Route the request and execute handler
    pub fn route(&self, request: &Request) -> ApiResult<Response> {
        let method = Method::from_str(&request.method)
            .ok_or_else(|| ApiError::MethodNotAllowed(request.method.clone()))?;

        match self.find(method, &request.path) {
            Some((route, params, handler)) => {
                // Create request with params
                let mut req = request.clone();
                req.path_params = params;

                tracing::debug!(
                    handler = %route.handler_name,
                    path = %request.path,
                    "Routing request"
                );

                handler(&req)
            }
            None => Err(ApiError::NotFound(format!("No route for {} {}", request.method, request.path))),
        }
    }

    /// Merge another router's routes (with optional prefix)
    pub fn merge(&mut self, other: Router, prefix: Option<&str>) {
        for (i, route) in other.routes.into_iter().enumerate() {
            let pattern = match prefix {
                Some(p) => format!("{}{}", p, route.pattern),
                None => route.pattern,
            };

            let new_route = Route::new(route.method, pattern, route.handler_name);
            let new_index = self.routes.len();
            self.routes.push(new_route);

            if let Some(handler) = other.handlers.get(&i) {
                self.handlers.insert(new_index, *handler);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handler(_req: &Request) -> ApiResult<Response> {
        Ok(Response::ok())
    }

    #[test]
    fn test_method_parsing() {
        assert_eq!(Method::from_str("GET"), Some(Method::GET));
        assert_eq!(Method::from_str("post"), Some(Method::POST));
        assert_eq!(Method::from_str("INVALID"), None);
    }

    #[test]
    fn test_route_matching() {
        let route = Route::new(Method::GET, "/api/users/:id", "get_user");

        let params = route.match_path("/api/users/123").unwrap();
        assert_eq!(params.get("id"), Some(&"123".to_string()));

        assert!(route.match_path("/api/users").is_none());
        assert!(route.match_path("/api/users/123/extra").is_none());
    }

    #[test]
    fn test_router_find() {
        let mut router = Router::new();
        router.get("/api/users/:id", test_handler, "get_user");
        router.post("/api/users", test_handler, "create_user");

        let (route, params, _) = router.find(Method::GET, "/api/users/42").unwrap();
        assert_eq!(route.handler_name, "get_user");
        assert_eq!(params.get("id"), Some(&"42".to_string()));

        let (route, _, _) = router.find(Method::POST, "/api/users").unwrap();
        assert_eq!(route.handler_name, "create_user");

        assert!(router.find(Method::DELETE, "/api/users/42").is_none());
    }

    #[test]
    fn test_router_prefix() {
        let mut router = Router::with_prefix("/api/v1");
        router.get("/users", test_handler, "list_users");

        let (route, _, _) = router.find(Method::GET, "/api/v1/users").unwrap();
        assert_eq!(route.handler_name, "list_users");
    }

    #[test]
    fn test_nested_params() {
        let route = Route::new(Method::GET, "/api/users/:user_id/bookings/:booking_id", "get_booking");

        let params = route.match_path("/api/users/123/bookings/456").unwrap();
        assert_eq!(params.get("user_id"), Some(&"123".to_string()));
        assert_eq!(params.get("booking_id"), Some(&"456".to_string()));
    }
}
