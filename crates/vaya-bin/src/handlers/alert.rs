//! Alert handlers

use vaya_api::{ApiError, ApiResult, JsonSerialize, Request, Response};

/// Create a new price alert
pub fn create_alert(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let _body = req.body_string().ok_or(ApiError::BadRequest("Missing request body".into()))?;

    // TODO: Parse and create alert
    let alert = AlertResponse {
        id: generate_alert_id(),
        origin: "SIN".into(),
        destination: "BKK".into(),
        target_price_cents: 15000,
        current_price_cents: 18000,
        status: "active".into(),
        created_at: current_timestamp(),
    };

    let mut response = Response::created();
    response.set_json_body(&alert);
    Ok(response)
}

/// List user's alerts
pub fn list_alerts(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let _page = req
        .query("page")
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(1);

    // TODO: Fetch alerts from database
    let response = AlertsListResponse {
        alerts: vec![],
        total: 0,
        page: 1,
        page_size: 20,
    };

    let mut resp = Response::ok();
    resp.set_json_body(&response);
    Ok(resp)
}

/// Get alert details
pub fn get_alert(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let alert_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing alert ID".into()))?;

    // TODO: Fetch alert from database
    Err(ApiError::NotFound(format!("Alert {} not found", alert_id)))
}

/// Delete an alert
pub fn delete_alert(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let alert_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing alert ID".into()))?;

    // TODO: Delete alert
    Err(ApiError::NotFound(format!("Alert {} not found", alert_id)))
}

/// Pause an alert
pub fn pause_alert(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let alert_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing alert ID".into()))?;

    // TODO: Pause alert
    Err(ApiError::NotFound(format!("Alert {} not found", alert_id)))
}

/// Resume an alert
pub fn resume_alert(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let alert_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing alert ID".into()))?;

    // TODO: Resume alert
    Err(ApiError::NotFound(format!("Alert {} not found", alert_id)))
}

/// Alert response
#[derive(Debug, Clone)]
pub struct AlertResponse {
    pub id: String,
    pub origin: String,
    pub destination: String,
    pub target_price_cents: i64,
    pub current_price_cents: i64,
    pub status: String,
    pub created_at: String,
}

impl JsonSerialize for AlertResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","origin":"{}","destination":"{}","target_price_cents":{},"current_price_cents":{},"status":"{}","created_at":"{}"}}"#,
            self.id, self.origin, self.destination, self.target_price_cents,
            self.current_price_cents, self.status, self.created_at
        )
    }
}

/// Alerts list response
#[derive(Debug, Clone)]
pub struct AlertsListResponse {
    pub alerts: Vec<AlertResponse>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

impl JsonSerialize for AlertsListResponse {
    fn to_json(&self) -> String {
        let alerts: Vec<String> = self.alerts.iter().map(|a| a.to_json()).collect();
        format!(
            r#"{{"alerts":[{}],"total":{},"page":{},"page_size":{}}}"#,
            alerts.join(","),
            self.total,
            self.page,
            self.page_size
        )
    }
}

/// Generate alert ID
fn generate_alert_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("alt-{:x}", timestamp)
}

/// Get current timestamp
fn current_timestamp() -> String {
    use time::OffsetDateTime;
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_alert_requires_auth() {
        let req = Request::new("POST", "/alerts");
        let result = create_alert(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_list_alerts_requires_auth() {
        let req = Request::new("GET", "/alerts");
        let result = list_alerts(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_alert_response_json() {
        let alert = AlertResponse {
            id: "alt-123".into(),
            origin: "SIN".into(),
            destination: "BKK".into(),
            target_price_cents: 15000,
            current_price_cents: 18000,
            status: "active".into(),
            created_at: "2026-01-15T10:00:00Z".into(),
        };
        let json = alert.to_json();
        assert!(json.contains(r#""origin":"SIN""#));
        assert!(json.contains(r#""target_price_cents":15000"#));
    }
}
