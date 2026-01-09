//! Payment handlers (6 handlers)

use crate::{ApiError, ApiResult, Request, Response};

/// POST /payments - Create a payment
pub fn create_payment_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement payment creation
    Ok(Response::created()
        .with_body(br#"{"payment_id":"payment_123","status":"pending","amount":299.99}"#.to_vec()))
}

/// GET /payments/{id} - Get payment details
pub fn get_payment_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing payment ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement payment retrieval
    Ok(Response::ok().with_body(
        br#"{"payment_id":"payment_123","status":"completed","amount":299.99,"currency":"USD"}"#
            .to_vec(),
    ))
}

/// GET /payments/methods - List payment methods
pub fn list_payment_methods_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement payment methods listing
    Ok(Response::ok().with_body(br#"{"methods":[],"total":0}"#.to_vec()))
}

/// POST /payments/methods - Add payment method
pub fn add_payment_method_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement payment method addition
    Ok(Response::created().with_body(
        br#"{"method_id":"method_123","type":"card","last4":"4242","added":true}"#.to_vec(),
    ))
}

/// DELETE /payments/methods/{id} - Remove payment method
pub fn remove_payment_method_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing method ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement payment method removal
    Ok(Response::ok().with_body(br#"{"method_id":"method_123","removed":true}"#.to_vec()))
}

/// POST /payments/{id}/refund - Process refund
pub fn process_refund_handler(req: &Request) -> ApiResult<Response> {
    let _id = req
        .param("id")
        .ok_or(ApiError::bad_request("Missing payment ID"))?;
    let _user_id = req
        .user_id
        .as_ref()
        .ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement refund processing
    Ok(Response::ok().with_body(br#"{"payment_id":"payment_123","refund_id":"refund_123","status":"processed","amount":299.99}"#.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_payment_handler() {
        let mut req = Request::new("POST", "/payments");
        req.user_id = Some("user_123".into());
        req.body = br#"{"booking_id":"booking_123","amount":299.99}"#.to_vec();
        let resp = create_payment_handler(&req).unwrap();
        assert_eq!(resp.status, 201);
    }

    #[test]
    fn test_list_payment_methods_handler() {
        let mut req = Request::new("GET", "/payments/methods");
        req.user_id = Some("user_123".into());
        let resp = list_payment_methods_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }
}
