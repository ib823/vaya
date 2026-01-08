//! HTTP request routing

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::http::Method;
use crate::{NetError, NetResult, Request, Response};

/// Handler function type
pub type Handler = Arc<
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = NetResult<Response>> + Send>> + Send + Sync,
>;

/// A route entry
#[derive(Clone)]
struct Route {
    /// HTTP method
    method: Method,
    /// Path pattern (supports :param syntax)
    pattern: String,
    /// Path segments for matching
    segments: Vec<PathSegment>,
    /// Handler function
    handler: Handler,
}

/// A path segment for pattern matching
#[derive(Clone, Debug)]
enum PathSegment {
    /// Literal segment (e.g., "api")
    Literal(String),
    /// Parameter segment (e.g., ":id")
    Param(String),
    /// Wildcard segment (e.g., "*")
    Wildcard,
}

impl Route {
    fn new(method: Method, pattern: &str, handler: Handler) -> Self {
        let segments = Self::parse_pattern(pattern);
        Self {
            method,
            pattern: pattern.to_string(),
            segments,
            handler,
        }
    }

    fn parse_pattern(pattern: &str) -> Vec<PathSegment> {
        pattern
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

    fn matches(&self, method: Method, path: &str) -> Option<HashMap<String, String>> {
        if self.method != method {
            return None;
        }

        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Check if wildcard at end
        let has_wildcard = matches!(self.segments.last(), Some(PathSegment::Wildcard));

        if !has_wildcard && path_segments.len() != self.segments.len() {
            return None;
        }

        if has_wildcard && path_segments.len() < self.segments.len() - 1 {
            return None;
        }

        let mut params = HashMap::new();

        for (i, segment) in self.segments.iter().enumerate() {
            match segment {
                PathSegment::Literal(s) => {
                    if path_segments.get(i) != Some(&s.as_str()) {
                        return None;
                    }
                }
                PathSegment::Param(name) => {
                    if let Some(value) = path_segments.get(i) {
                        params.insert(name.clone(), (*value).to_string());
                    } else {
                        return None;
                    }
                }
                PathSegment::Wildcard => {
                    // Wildcard matches everything remaining
                    let remaining: Vec<&str> = path_segments[i..].to_vec();
                    params.insert("*".to_string(), remaining.join("/"));
                    break;
                }
            }
        }

        Some(params)
    }
}

/// HTTP request router
#[derive(Clone)]
pub struct Router {
    routes: Vec<Route>,
    not_found_handler: Option<Handler>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            not_found_handler: None,
        }
    }

    /// Add a GET route
    pub fn get<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NetResult<Response>> + Send + 'static,
    {
        self.route(Method::GET, path, handler)
    }

    /// Add a POST route
    pub fn post<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NetResult<Response>> + Send + 'static,
    {
        self.route(Method::POST, path, handler)
    }

    /// Add a PUT route
    pub fn put<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NetResult<Response>> + Send + 'static,
    {
        self.route(Method::PUT, path, handler)
    }

    /// Add a DELETE route
    pub fn delete<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NetResult<Response>> + Send + 'static,
    {
        self.route(Method::DELETE, path, handler)
    }

    /// Add a PATCH route
    pub fn patch<F, Fut>(&mut self, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NetResult<Response>> + Send + 'static,
    {
        self.route(Method::PATCH, path, handler)
    }

    /// Add a route with any method
    pub fn route<F, Fut>(&mut self, method: Method, path: &str, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NetResult<Response>> + Send + 'static,
    {
        let handler = Arc::new(move |req: Request| {
            let fut = handler(req);
            Box::pin(fut) as Pin<Box<dyn Future<Output = NetResult<Response>> + Send>>
        });

        self.routes.push(Route::new(method, path, handler));
        self
    }

    /// Set a custom 404 handler
    pub fn not_found<F, Fut>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = NetResult<Response>> + Send + 'static,
    {
        let handler = Arc::new(move |req: Request| {
            let fut = handler(req);
            Box::pin(fut) as Pin<Box<dyn Future<Output = NetResult<Response>> + Send>>
        });
        self.not_found_handler = Some(handler);
        self
    }

    /// Merge another router's routes under a prefix
    pub fn merge(&mut self, prefix: &str, other: Router) -> &mut Self {
        for mut route in other.routes {
            route.pattern = format!("{}{}", prefix, route.pattern);
            route.segments = Route::parse_pattern(&route.pattern);
            self.routes.push(route);
        }
        self
    }

    /// Route a request to the appropriate handler
    pub async fn handle(&self, mut request: Request) -> NetResult<Response> {
        let method = request.method();
        let path = request.path().to_string();

        // Find matching route
        for route in &self.routes {
            if let Some(params) = route.matches(method, &path) {
                request.set_params(params);
                return (route.handler)(request).await;
            }
        }

        // Check for method not allowed (path matches but method doesn't)
        let path_matches = self.routes.iter().any(|r| {
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            r.segments.len() == segments.len()
        });

        if path_matches {
            return Err(NetError::MethodNotAllowed);
        }

        // Custom 404 handler or default
        if let Some(handler) = &self.not_found_handler {
            return (handler)(request).await;
        }

        Err(NetError::NotFound)
    }

    /// Get the number of registered routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn hello_handler(_req: Request) -> NetResult<Response> {
        Ok(Response::ok().text("Hello!"))
    }

    async fn user_handler(req: Request) -> NetResult<Response> {
        let id = req.param("id").unwrap_or("unknown");
        Ok(Response::ok().text(format!("User: {}", id)))
    }

    #[tokio::test]
    async fn test_simple_route() {
        let mut router = Router::new();
        router.get("/hello", hello_handler);

        let req = Request::new(Method::GET, "/hello");
        let res = router.handle(req).await.unwrap();
        assert_eq!(res.status(), crate::StatusCode::Ok);
    }

    #[tokio::test]
    async fn test_param_route() {
        let mut router = Router::new();
        router.get("/users/:id", user_handler);

        let req = Request::new(Method::GET, "/users/123");
        let res = router.handle(req).await.unwrap();
        assert_eq!(res.status(), crate::StatusCode::Ok);
    }

    #[tokio::test]
    async fn test_not_found() {
        let router = Router::new();
        let req = Request::new(Method::GET, "/nonexistent");
        let res = router.handle(req).await;
        assert!(matches!(res, Err(NetError::NotFound)));
    }
}
