//! Pool (group buying) handlers

use vaya_api::{ApiError, ApiResult, JsonSerialize, Request, Response};

/// Create a new pool
pub fn create_pool(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let _body = req
        .body_string()
        .ok_or(ApiError::BadRequest("Missing request body".into()))?;

    // TODO: Parse and create pool
    let pool = PoolResponse {
        id: generate_pool_id(),
        status: "forming".into(),
        members: 1,
        target_size: 10,
        current_price_cents: 0,
        created_at: current_timestamp(),
    };

    let mut response = Response::created();
    response.set_json_body(&pool);
    Ok(response)
}

/// List pools
pub fn list_pools(req: &Request) -> ApiResult<Response> {
    let _status = req.query("status").cloned();
    let _page = req
        .query("page")
        .and_then(|p| p.parse::<u32>().ok())
        .unwrap_or(1);

    // TODO: Fetch pools from database
    let response = PoolsListResponse {
        pools: vec![],
        total: 0,
        page: 1,
        page_size: 20,
    };

    let mut resp = Response::ok();
    resp.set_json_body(&response);
    Ok(resp)
}

/// Get pool details
pub fn get_pool(req: &Request) -> ApiResult<Response> {
    let pool_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing pool ID".into()))?;

    // TODO: Fetch pool from database
    Err(ApiError::NotFound(format!("Pool {} not found", pool_id)))
}

/// Join a pool
pub fn join_pool(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let pool_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing pool ID".into()))?;

    // TODO: Join pool
    Err(ApiError::NotFound(format!("Pool {} not found", pool_id)))
}

/// Leave a pool
pub fn leave_pool(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let pool_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing pool ID".into()))?;

    // TODO: Leave pool
    Err(ApiError::NotFound(format!("Pool {} not found", pool_id)))
}

/// Contribute to pool
pub fn contribute(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let pool_id = req
        .param("id")
        .ok_or(ApiError::BadRequest("Missing pool ID".into()))?;

    // TODO: Process contribution
    Err(ApiError::NotFound(format!("Pool {} not found", pool_id)))
}

/// Pool response
#[derive(Debug, Clone)]
pub struct PoolResponse {
    pub id: String,
    pub status: String,
    pub members: u32,
    pub target_size: u32,
    pub current_price_cents: i64,
    pub created_at: String,
}

impl JsonSerialize for PoolResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","status":"{}","members":{},"target_size":{},"current_price_cents":{},"created_at":"{}"}}"#,
            self.id,
            self.status,
            self.members,
            self.target_size,
            self.current_price_cents,
            self.created_at
        )
    }
}

/// Pools list response
#[derive(Debug, Clone)]
pub struct PoolsListResponse {
    pub pools: Vec<PoolResponse>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

impl JsonSerialize for PoolsListResponse {
    fn to_json(&self) -> String {
        let pools: Vec<String> = self.pools.iter().map(|p| p.to_json()).collect();
        format!(
            r#"{{"pools":[{}],"total":{},"page":{},"page_size":{}}}"#,
            pools.join(","),
            self.total,
            self.page,
            self.page_size
        )
    }
}

/// Generate pool ID
fn generate_pool_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("pool-{:x}", timestamp)
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
    fn test_create_pool_requires_auth() {
        let req = Request::new("POST", "/pools");
        let result = create_pool(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_join_pool_requires_auth() {
        let mut req = Request::new("POST", "/pools/123/join");
        req.path_params.insert("id".into(), "123".into());
        let result = join_pool(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_pool_response_json() {
        let pool = PoolResponse {
            id: "pool-123".into(),
            status: "forming".into(),
            members: 5,
            target_size: 20,
            current_price_cents: 50000,
            created_at: "2026-01-15T10:00:00Z".into(),
        };
        let json = pool.to_json();
        assert!(json.contains(r#""members":5"#));
        assert!(json.contains(r#""status":"forming""#));
    }
}
