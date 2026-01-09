//! Search handlers (6 handlers)

use crate::{ApiError, ApiResult, Request, Response};

/// GET /search - Search for flights
pub fn search_flights_handler(req: &Request) -> ApiResult<Response> {
    let _origin = req.query("origin").ok_or(ApiError::bad_request("Missing origin"))?;
    let _dest = req.query("destination").ok_or(ApiError::bad_request("Missing destination"))?;
    // TODO: Implement flight search
    Ok(Response::ok().with_body(br#"{"search_id":"search_123","results":[],"total":0}"#.to_vec()))
}

/// GET /search/results/{id} - Get search results by ID
pub fn get_search_results_handler(req: &Request) -> ApiResult<Response> {
    let _id = req.param("id").ok_or(ApiError::bad_request("Missing search ID"))?;
    // TODO: Implement search results retrieval
    Ok(Response::ok().with_body(br#"{"search_id":"search_123","results":[],"expires_at":"2026-01-10T00:00:00Z"}"#.to_vec()))
}

/// POST /search/intent - Create a search intent for tracking
pub fn create_search_intent_handler(req: &Request) -> ApiResult<Response> {
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement search intent creation
    Ok(Response::created().with_body(br#"{"intent_id":"intent_123","created":true}"#.to_vec()))
}

/// GET /search/history - Get user's search history
pub fn get_search_history_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req.user_id.as_ref().ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement search history retrieval
    Ok(Response::ok().with_body(br#"{"searches":[],"total":0}"#.to_vec()))
}

/// GET /search/popular - Get popular routes
pub fn get_popular_routes_handler(_req: &Request) -> ApiResult<Response> {
    // TODO: Implement popular routes retrieval
    Ok(Response::ok().with_body(br#"{"routes":[{"origin":"SIN","destination":"BKK","searches":1000}]}"#.to_vec()))
}

/// GET /search/suggestions - Get search suggestions
pub fn get_search_suggestions_handler(req: &Request) -> ApiResult<Response> {
    let _query = req.query("q").ok_or(ApiError::bad_request("Missing query"))?;
    // TODO: Implement search suggestions
    Ok(Response::ok().with_body(br#"{"suggestions":[]}"#.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_flights_handler() {
        let mut req = Request::new("GET", "/search");
        req.query_params.insert("origin".into(), "SIN".into());
        req.query_params.insert("destination".into(), "BKK".into());
        let resp = search_flights_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_get_popular_routes_handler() {
        let req = Request::new("GET", "/search/popular");
        let resp = get_popular_routes_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }
}
