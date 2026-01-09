//! Traveler handlers (5 handlers)

use crate::{ApiError, ApiResult, Request, Response};

/// POST /travelers - Create a traveler profile
pub fn create_traveler_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement traveler creation
    Ok(Response::created()
        .with_body(br#"{"traveler_id":"traveler_123","name":"John Doe","created":true}"#.to_vec()))
}

/// GET /travelers - List user's travelers
pub fn list_travelers_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement traveler listing
    Ok(Response::ok().with_body(br#"{"travelers":[],"total":0}"#.to_vec()))
}

/// GET /travelers/{id} - Get traveler details
pub fn get_traveler_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing traveler ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement traveler retrieval
    Ok(Response::ok().with_body(br#"{"traveler_id":"traveler_123","first_name":"John","last_name":"Doe","date_of_birth":"1990-01-01"}"#.to_vec()))
}

/// PUT /travelers/{id} - Update traveler
pub fn update_traveler_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing traveler ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement traveler update
    Ok(Response::ok().with_body(br#"{"traveler_id":"traveler_123","updated":true}"#.to_vec()))
}

/// DELETE /travelers/{id} - Delete traveler
pub fn delete_traveler_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing traveler ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement traveler deletion
    Ok(Response::ok().with_body(br#"{"traveler_id":"traveler_123","deleted":true}"#.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_traveler_handler() {
        let mut req = Request::new("POST", "/travelers");
        req.user_id = Some("user_123".into());
        req.body = br#"{"first_name":"John","last_name":"Doe"}"#.to_vec();
        let resp = create_traveler_handler(&req).unwrap();
        assert_eq!(resp.status, 201);
    }

    #[test]
    fn test_list_travelers_handler() {
        let mut req = Request::new("GET", "/travelers");
        req.user_id = Some("user_123".into());
        let resp = list_travelers_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }
}
