//! Trip handlers (6 handlers)
//!
//! Endpoints for trip/itinerary management:
//! - GET /trips - List user's trips
//! - POST /trips - Create new trip
//! - GET /trips/{id} - Get trip details
//! - PUT /trips/{id} - Update trip
//! - DELETE /trips/{id} - Delete trip
//! - POST /trips/{id}/bookings - Add booking to trip

use crate::{ApiError, ApiResult, Request, Response};

/// GET /trips - List user's trips
pub fn list_trips_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement trip listing
    Ok(Response::ok().with_body(br#"{"trips":[],"total":0,"page":1}"#.to_vec()))
}

/// POST /trips - Create new trip
pub fn create_trip_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // Parse and validate trip data
    let body = String::from_utf8_lossy(&req.body);
    if !body.contains("\"name\"") {
        return Err(ApiError::bad_request("Missing required field: name"));
    }
    // TODO: Implement trip creation in database
    Ok(Response::created().with_body(br#"{"trip_id":"trip_new","name":"My Trip","status":"planning","created_at":"2026-01-09T00:00:00Z"}"#.to_vec()))
}

/// GET /trips/{id} - Get trip details
pub fn get_trip_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing trip ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement trip retrieval
    Ok(Response::ok().with_body(br#"{"trip_id":"trip_123","name":"Singapore to Bangkok","bookings":[],"status":"upcoming"}"#.to_vec()))
}

/// PUT /trips/{id} - Update trip
pub fn update_trip_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing trip ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement trip update
    Ok(Response::ok().with_body(br#"{"trip_id":"trip_123","updated":true}"#.to_vec()))
}

/// DELETE /trips/{id} - Delete trip
pub fn delete_trip_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing trip ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement trip deletion
    Ok(Response::ok().with_body(br#"{"trip_id":"trip_123","deleted":true}"#.to_vec()))
}

/// POST /trips/{id}/bookings - Add booking to trip
pub fn add_booking_to_trip_handler(req: &Request) -> ApiResult<Response> {
    let trip_id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing trip ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // Parse and validate booking reference
    let body = String::from_utf8_lossy(&req.body);
    if !body.contains("\"booking_id\"") {
        return Err(ApiError::bad_request("Missing required field: booking_id"));
    }
    // TODO: Implement adding booking to trip in database
    let response = format!(
        r#"{{"trip_id":"{}","booking_added":true,"updated_at":"2026-01-09T00:00:00Z"}}"#,
        trip_id
    );
    Ok(Response::ok().with_body(response.into_bytes()))
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

    #[test]
    fn test_create_trip_handler() {
        let mut req = Request::new("POST", "/trips");
        req.user_id = Some("user_123".into());
        req.body = br#"{"name":"My Holiday Trip"}"#.to_vec();
        let resp = create_trip_handler(&req).unwrap();
        assert_eq!(resp.status, 201);
    }

    #[test]
    fn test_add_booking_to_trip_handler() {
        let mut req = Request::new("POST", "/trips/trip_123/bookings");
        req.user_id = Some("user_123".into());
        req.path_params.insert("id".into(), "trip_123".into());
        req.body = br#"{"booking_id":"BK-123456"}"#.to_vec();
        let resp = add_booking_to_trip_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }
}
