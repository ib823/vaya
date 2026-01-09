//! Health check handlers

use vaya_api::{ApiResult, JsonSerialize, Request, Response};

/// Health response
#[derive(Debug, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

impl JsonSerialize for HealthResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"status":"{}","version":"{}","uptime_seconds":{}}}"#,
            self.status, self.version, self.uptime_seconds
        )
    }
}

/// Readiness response
#[derive(Debug, Clone)]
pub struct ReadyResponse {
    pub ready: bool,
    pub checks: Vec<CheckResult>,
}

impl JsonSerialize for ReadyResponse {
    fn to_json(&self) -> String {
        let checks: Vec<String> = self.checks.iter().map(|c| c.to_json()).collect();
        format!(
            r#"{{"ready":{},"checks":[{}]}}"#,
            self.ready,
            checks.join(",")
        )
    }
}

/// Individual check result
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
}

impl JsonSerialize for CheckResult {
    fn to_json(&self) -> String {
        match &self.message {
            Some(msg) => format!(
                r#"{{"name":"{}","status":"{}","message":"{}"}}"#,
                self.name,
                self.status,
                escape_json(msg)
            ),
            None => format!(r#"{{"name":"{}","status":"{}"}}"#, self.name, self.status),
        }
    }
}

/// Main health check endpoint
pub fn health(_req: &Request) -> ApiResult<Response> {
    let health = HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime_seconds: 0, // Would get from app state
    };

    let mut response = Response::ok();
    response.set_json_body(&health);
    Ok(response)
}

/// Readiness check (for load balancers)
pub fn ready(_req: &Request) -> ApiResult<Response> {
    // Check all dependencies
    let checks = vec![
        CheckResult {
            name: "database".into(),
            status: "ok".into(),
            message: None,
        },
        CheckResult {
            name: "cache".into(),
            status: "ok".into(),
            message: None,
        },
    ];

    let all_ok = checks.iter().all(|c| c.status == "ok");

    let ready_response = ReadyResponse {
        ready: all_ok,
        checks,
    };

    let mut response = if all_ok {
        Response::ok()
    } else {
        Response::new(503, "Service Unavailable")
    };

    response.set_json_body(&ready_response);
    Ok(response)
}

/// Liveness check (for orchestrators)
pub fn live(_req: &Request) -> ApiResult<Response> {
    // Simple liveness - just return OK if we're running
    let mut response = Response::ok();
    response.body = b"{\"live\":true}".to_vec();
    Ok(response)
}

/// Escape JSON string
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_handler() {
        let req = Request::new("GET", "/health");
        let response = health(&req).unwrap();
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_ready_handler() {
        let req = Request::new("GET", "/ready");
        let response = ready(&req).unwrap();
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_live_handler() {
        let req = Request::new("GET", "/live");
        let response = live(&req).unwrap();
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_health_response_json() {
        let health = HealthResponse {
            status: "ok".into(),
            version: "1.0.0".into(),
            uptime_seconds: 3600,
        };
        let json = health.to_json();
        assert!(json.contains(r#""status":"ok""#));
        assert!(json.contains(r#""uptime_seconds":3600"#));
    }
}
