//! User handlers (11 handlers)

use crate::{ApiError, ApiResult, Request, Response};

/// GET /users/me - Get current user profile
pub fn get_current_user_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement user profile retrieval
    Ok(Response::ok().with_body(
        br#"{"id":"user_123","email":"user@example.com","name":"John Doe","tier":"gold"}"#.to_vec(),
    ))
}

/// PUT /users/me - Update current user profile
pub fn update_user_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement user profile update
    Ok(Response::ok().with_body(br#"{"id":"user_123","updated":true}"#.to_vec()))
}

/// GET /users/me/settings - Get user settings
pub fn get_settings_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement settings retrieval
    Ok(Response::ok().with_body(
        br#"{"notifications_enabled":true,"email_alerts":true,"currency":"USD","language":"en"}"#
            .to_vec(),
    ))
}

/// PUT /users/me/settings - Update user settings
pub fn update_settings_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement settings update
    Ok(Response::ok().with_body(br#"{"updated":true}"#.to_vec()))
}

/// GET /users/me/preferences - Get user travel preferences
pub fn get_preferences_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement preferences retrieval
    Ok(Response::ok().with_body(
        br#"{"preferred_airlines":[],"cabin_class":"economy","seat_preference":"window"}"#.to_vec(),
    ))
}

/// PUT /users/me/preferences - Update user travel preferences
pub fn update_preferences_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement preferences update
    Ok(Response::ok().with_body(br#"{"updated":true}"#.to_vec()))
}

/// GET /users/me/stats - Get user statistics
pub fn get_user_stats_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement user stats retrieval
    Ok(Response::ok().with_body(
        br#"{"total_bookings":5,"total_spent":1500.00,"savings":250.00,"pools_joined":3}"#.to_vec(),
    ))
}

/// POST /users/me/upgrade - Upgrade user tier
pub fn upgrade_tier_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement tier upgrade
    Ok(Response::ok().with_body(
        br#"{"previous_tier":"silver","new_tier":"gold","effective_date":"2026-01-09"}"#.to_vec(),
    ))
}

/// GET /users/me/saved-routes - List saved routes
pub fn list_saved_routes_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement saved routes retrieval from database
    Ok(Response::ok().with_body(br#"{"saved_routes":[],"total":0}"#.to_vec()))
}

/// POST /users/me/saved-routes - Save a route
pub fn save_route_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // Parse and validate route
    let body = String::from_utf8_lossy(&req.body);
    if !body.contains("\"origin\"") || !body.contains("\"destination\"") {
        return Err(ApiError::bad_request(
            "Missing required fields: origin, destination",
        ));
    }
    // TODO: Implement route saving to database
    Ok(Response::created().with_body(br#"{"id":"route_123","origin":"KUL","destination":"NRT","created_at":"2026-01-09T00:00:00Z"}"#.to_vec()))
}

/// DELETE /users/me/saved-routes/{id} - Delete a saved route
pub fn delete_saved_route_handler(req: &Request) -> ApiResult<Response> {
    let route_id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing route ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement route deletion from database
    let _ = route_id;
    Ok(Response::ok().with_body(br#"{"deleted":true}"#.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_user_handler() {
        let mut req = Request::new("GET", "/users/me");
        req.user_id = Some("user_123".into());
        let resp = get_current_user_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_get_settings_handler() {
        let mut req = Request::new("GET", "/users/me/settings");
        req.user_id = Some("user_123".into());
        let resp = get_settings_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_list_saved_routes_handler() {
        let mut req = Request::new("GET", "/users/me/saved-routes");
        req.user_id = Some("user_123".into());
        let resp = list_saved_routes_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_save_route_handler() {
        let mut req = Request::new("POST", "/users/me/saved-routes");
        req.user_id = Some("user_123".into());
        req.body = br#"{"origin":"KUL","destination":"NRT"}"#.to_vec();
        let resp = save_route_handler(&req).unwrap();
        assert_eq!(resp.status, 201);
    }

    #[test]
    fn test_delete_saved_route_handler() {
        let mut req = Request::new("DELETE", "/users/me/saved-routes/route_123");
        req.user_id = Some("user_123".into());
        req.path_params.insert("id".into(), "route_123".into());
        let resp = delete_saved_route_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }
}
