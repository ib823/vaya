//! Admin handlers (8 handlers)

use crate::{ApiError, ApiResult, Request, Response};

/// Check if user has admin role
fn require_admin(req: &Request) -> ApiResult<()> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if !req.has_role("admin") {
        return Err(ApiError::forbidden("Admin access required"));
    }
    Ok(())
}

/// GET /admin/users - List all users (admin only)
pub fn admin_list_users_handler(req: &Request) -> ApiResult<Response> {
    require_admin(req)?;
    // TODO: Implement admin user listing
    Ok(Response::ok().with_body(br#"{"users":[],"total":0,"page":1,"page_size":50}"#.to_vec()))
}

/// GET /admin/users/{id} - Get user details (admin only)
pub fn admin_get_user_handler(req: &Request) -> ApiResult<Response> {
    require_admin(req)?;
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing user ID"))?;
    // TODO: Implement admin user retrieval
    Ok(Response::ok().with_body(
        br#"{"id":"user_123","email":"user@example.com","status":"active","tier":"gold"}"#.to_vec(),
    ))
}

/// PUT /admin/users/{id} - Update user (admin only)
pub fn admin_update_user_handler(req: &Request) -> ApiResult<Response> {
    require_admin(req)?;
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing user ID"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement admin user update
    Ok(Response::ok().with_body(br#"{"id":"user_123","updated":true}"#.to_vec()))
}

/// DELETE /admin/users/{id} - Delete user (admin only)
pub fn admin_delete_user_handler(req: &Request) -> ApiResult<Response> {
    require_admin(req)?;
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing user ID"))?;
    // TODO: Implement admin user deletion
    Ok(Response::ok().with_body(br#"{"id":"user_123","deleted":true}"#.to_vec()))
}

/// GET /admin/stats - Get system statistics (admin only)
pub fn admin_get_stats_handler(req: &Request) -> ApiResult<Response> {
    require_admin(req)?;
    // TODO: Implement admin stats retrieval
    Ok(Response::ok().with_body(
        br#"{"total_users":10000,"active_users":5000,"total_bookings":25000,"total_pools":500}"#
            .to_vec(),
    ))
}

/// GET /admin/stats/revenue - Get revenue statistics (admin only)
pub fn admin_get_revenue_handler(req: &Request) -> ApiResult<Response> {
    require_admin(req)?;
    // TODO: Implement revenue stats retrieval
    Ok(Response::ok().with_body(
        br#"{"total_revenue":1500000.00,"monthly_revenue":150000.00,"currency":"USD"}"#.to_vec(),
    ))
}

/// GET /admin/stats/bookings - Get booking statistics (admin only)
pub fn admin_get_booking_stats_handler(req: &Request) -> ApiResult<Response> {
    require_admin(req)?;
    // TODO: Implement booking stats retrieval
    Ok(Response::ok().with_body(
        br#"{"total_bookings":25000,"confirmed":20000,"cancelled":3000,"pending":2000}"#.to_vec(),
    ))
}

/// GET /admin/stats/oracle - Get oracle/ML statistics (admin only)
pub fn admin_get_oracle_stats_handler(req: &Request) -> ApiResult<Response> {
    require_admin(req)?;
    // TODO: Implement oracle stats retrieval
    Ok(Response::ok().with_body(
        br#"{"predictions_made":100000,"accuracy":0.82,"mape":8.5,"model_version":"2.1.0"}"#
            .to_vec(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_list_users_handler() {
        let mut req = Request::new("GET", "/admin/users");
        req.user_id = Some("admin_123".into());
        req.user_roles = vec!["admin".into()];
        let resp = admin_list_users_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_admin_get_stats_handler() {
        let mut req = Request::new("GET", "/admin/stats");
        req.user_id = Some("admin_123".into());
        req.user_roles = vec!["admin".into()];
        let resp = admin_get_stats_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_admin_requires_role() {
        let mut req = Request::new("GET", "/admin/users");
        req.user_id = Some("user_123".into());
        // No admin role
        let result = admin_list_users_handler(&req);
        assert!(result.is_err());
    }
}
