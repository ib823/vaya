//! Trip handlers (4 handlers)

use crate::{ApiError, ApiResult, Request, Response};

/// GET /trips - List user's trips
pub fn list_trips_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req.user_id.as_ref().ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement trip listing
    Ok(Response::ok().with_body(br#"{"trips":[],"total":0,"page":1}"#.to_vec()))
}

/// GET /trips/{id} - Get trip details
pub fn get_trip_handler(req: &Request) -> ApiResult<Response> {
    let _id = req.param("id").ok_or(ApiError::bad_request("Missing trip ID"))?;
    let _user_id = req.user_id.as_ref().ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement trip retrieval
    Ok(Response::ok().with_body(br#"{"trip_id":"trip_123","name":"Singapore to Bangkok","bookings":[],"status":"upcoming"}"#.to_vec()))
}

/// PUT /trips/{id} - Update trip
pub fn update_trip_handler(req: &Request) -> ApiResult<Response> {
    let _id = req.param("id").ok_or(ApiError::bad_request("Missing trip ID"))?;
    let _user_id = req.user_id.as_ref().ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement trip update
    Ok(Response::ok().with_body(br#"{"trip_id":"trip_123","updated":true}"#.to_vec()))
}

/// DELETE /trips/{id} - Delete trip
pub fn delete_trip_handler(req: &Request) -> ApiResult<Response> {
    let _id = req.param("id").ok_or(ApiError::bad_request("Missing trip ID"))?;
    let _user_id = req.user_id.as_ref().ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement trip deletion
    Ok(Response::ok().with_body(br#"{"trip_id":"trip_123","deleted":true}"#.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_trips_handler() {
        let mut req = Request::new("GET", "/trips");
        req.user_id = Some("user_123".into());
        let resp = list_trips_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_get_trip_handler() {
        let mut req = Request::new("GET", "/trips/trip_123");
        req.user_id = Some("user_123".into());
        req.path_params.insert("id".into(), "trip_123".into());
        let resp = get_trip_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }
}
