//! Oracle/Prediction handlers (4 handlers)

use crate::{ApiError, ApiResult, Request, Response};

/// POST /oracle/predict - Get price prediction
pub fn predict_handler(req: &Request) -> ApiResult<Response> {
    if req.body.is_empty() {
        return Err(ApiError::bad_request("Missing request body"));
    }
    // TODO: Implement price prediction using ML models
    Ok(Response::ok().with_body(br#"{"prediction_id":"pred_123","direction":"down","confidence":0.85,"expected_change":-50.00,"recommendation":"wait"}"#.to_vec()))
}

/// GET /oracle/explain/{id} - Explain a prediction
pub fn explain_prediction_handler(req: &Request) -> ApiResult<Response> {
    let _id = req.param("id").ok_or(ApiError::bad_request("Missing prediction ID"))?;
    // TODO: Implement prediction explanation (feature importance, etc.)
    Ok(Response::ok().with_body(br#"{"prediction_id":"pred_123","factors":[{"name":"seasonal_demand","weight":0.3},{"name":"days_until_departure","weight":0.25}]}"#.to_vec()))
}

/// GET /oracle/history - Get prediction history
pub fn get_prediction_history_handler(req: &Request) -> ApiResult<Response> {
    let _user_id = req.user_id.as_ref().ok_or(ApiError::unauthorized("Authentication required"))?;
    // TODO: Implement prediction history retrieval
    Ok(Response::ok().with_body(br#"{"predictions":[],"total":0}"#.to_vec()))
}

/// GET /oracle/accuracy - Get oracle accuracy metrics
pub fn get_oracle_accuracy_handler(_req: &Request) -> ApiResult<Response> {
    // TODO: Implement accuracy metrics retrieval
    Ok(Response::ok().with_body(br#"{"accuracy":0.82,"mape":8.5,"samples":10000,"last_updated":"2026-01-09T00:00:00Z"}"#.to_vec()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_handler() {
        let mut req = Request::new("POST", "/oracle/predict");
        req.body = br#"{"route":"SIN-BKK","date":"2026-02-01"}"#.to_vec();
        let resp = predict_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_get_oracle_accuracy_handler() {
        let req = Request::new("GET", "/oracle/accuracy");
        let resp = get_oracle_accuracy_handler(&req).unwrap();
        assert_eq!(resp.status, 200);
    }
}
