//! Oracle (pricing insights) handlers

use vaya_api::{ApiError, ApiResult, JsonSerialize, Request, Response};

/// Get price prediction
pub fn get_prediction(req: &Request) -> ApiResult<Response> {
    let origin = req
        .query("origin")
        .ok_or(ApiError::BadRequest("Missing origin parameter".into()))?;

    let destination = req
        .query("destination")
        .ok_or(ApiError::BadRequest("Missing destination parameter".into()))?;

    let date = req
        .query("date")
        .ok_or(ApiError::BadRequest("Missing date parameter".into()))?;

    // Validate airport codes
    if origin.len() != 3 || destination.len() != 3 {
        return Err(ApiError::BadRequest("Airport codes must be 3 characters".into()));
    }

    // TODO: Get actual prediction from oracle
    let prediction = PredictionResponse {
        origin: origin.clone(),
        destination: destination.clone(),
        date: date.clone(),
        predicted_price_cents: 25000,
        confidence: 0.85,
        trend: "stable".into(),
        recommendation: "book_now".into(),
        price_range_low_cents: 22000,
        price_range_high_cents: 30000,
    };

    let mut response = Response::ok();
    response.set_json_body(&prediction);
    Ok(response)
}

/// Get pricing insights
pub fn get_insights(req: &Request) -> ApiResult<Response> {
    let origin = req
        .query("origin")
        .ok_or(ApiError::BadRequest("Missing origin parameter".into()))?;

    let destination = req
        .query("destination")
        .ok_or(ApiError::BadRequest("Missing destination parameter".into()))?;

    // TODO: Get actual insights
    let insights = InsightsResponse {
        origin: origin.clone(),
        destination: destination.clone(),
        cheapest_day: "Tuesday".into(),
        most_expensive_day: "Friday".into(),
        average_price_cents: 28000,
        price_volatility: "medium".into(),
        best_advance_days: 21,
        season: "peak".into(),
    };

    let mut response = Response::ok();
    response.set_json_body(&insights);
    Ok(response)
}

/// Get best booking time
pub fn get_best_time(req: &Request) -> ApiResult<Response> {
    let origin = req
        .query("origin")
        .ok_or(ApiError::BadRequest("Missing origin parameter".into()))?;

    let destination = req
        .query("destination")
        .ok_or(ApiError::BadRequest("Missing destination parameter".into()))?;

    let departure_date = req
        .query("departure_date")
        .ok_or(ApiError::BadRequest("Missing departure_date parameter".into()))?;

    // TODO: Calculate best booking time
    let best_time = BestTimeResponse {
        origin: origin.clone(),
        destination: destination.clone(),
        departure_date: departure_date.clone(),
        recommended_booking_date: calculate_booking_date(departure_date),
        expected_price_cents: 24000,
        savings_percent: 15,
        confidence: 0.78,
    };

    let mut response = Response::ok();
    response.set_json_body(&best_time);
    Ok(response)
}

/// Prediction response
#[derive(Debug, Clone)]
pub struct PredictionResponse {
    pub origin: String,
    pub destination: String,
    pub date: String,
    pub predicted_price_cents: i64,
    pub confidence: f64,
    pub trend: String,
    pub recommendation: String,
    pub price_range_low_cents: i64,
    pub price_range_high_cents: i64,
}

impl JsonSerialize for PredictionResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"origin":"{}","destination":"{}","date":"{}","predicted_price_cents":{},"confidence":{},"trend":"{}","recommendation":"{}","price_range_low_cents":{},"price_range_high_cents":{}}}"#,
            self.origin, self.destination, self.date, self.predicted_price_cents,
            self.confidence, self.trend, self.recommendation,
            self.price_range_low_cents, self.price_range_high_cents
        )
    }
}

/// Insights response
#[derive(Debug, Clone)]
pub struct InsightsResponse {
    pub origin: String,
    pub destination: String,
    pub cheapest_day: String,
    pub most_expensive_day: String,
    pub average_price_cents: i64,
    pub price_volatility: String,
    pub best_advance_days: u32,
    pub season: String,
}

impl JsonSerialize for InsightsResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"origin":"{}","destination":"{}","cheapest_day":"{}","most_expensive_day":"{}","average_price_cents":{},"price_volatility":"{}","best_advance_days":{},"season":"{}"}}"#,
            self.origin, self.destination, self.cheapest_day, self.most_expensive_day,
            self.average_price_cents, self.price_volatility, self.best_advance_days, self.season
        )
    }
}

/// Best time response
#[derive(Debug, Clone)]
pub struct BestTimeResponse {
    pub origin: String,
    pub destination: String,
    pub departure_date: String,
    pub recommended_booking_date: String,
    pub expected_price_cents: i64,
    pub savings_percent: u32,
    pub confidence: f64,
}

impl JsonSerialize for BestTimeResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"origin":"{}","destination":"{}","departure_date":"{}","recommended_booking_date":"{}","expected_price_cents":{},"savings_percent":{},"confidence":{}}}"#,
            self.origin, self.destination, self.departure_date,
            self.recommended_booking_date, self.expected_price_cents,
            self.savings_percent, self.confidence
        )
    }
}

/// Calculate recommended booking date (21 days before departure)
fn calculate_booking_date(departure_date: &str) -> String {
    // Simple implementation - would use proper date math in production
    // For now, just return a placeholder
    if departure_date.len() >= 10 {
        // Parse YYYY-MM-DD and subtract 21 days (simplified)
        departure_date[..10].to_string()
    } else {
        departure_date.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction_missing_params() {
        let req = Request::new("GET", "/oracle/predict");
        let result = get_prediction(&req);
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_prediction_invalid_airport() {
        let mut req = Request::new("GET", "/oracle/predict");
        req.query_params.insert("origin".into(), "INVALID".into());
        req.query_params.insert("destination".into(), "BKK".into());
        req.query_params.insert("date".into(), "2026-02-15".into());
        let result = get_prediction(&req);
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_insights_missing_params() {
        let req = Request::new("GET", "/oracle/insights");
        let result = get_insights(&req);
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_best_time_missing_params() {
        let req = Request::new("GET", "/oracle/best-time");
        let result = get_best_time(&req);
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_prediction_response_json() {
        let prediction = PredictionResponse {
            origin: "SIN".into(),
            destination: "BKK".into(),
            date: "2026-02-15".into(),
            predicted_price_cents: 25000,
            confidence: 0.85,
            trend: "stable".into(),
            recommendation: "book_now".into(),
            price_range_low_cents: 22000,
            price_range_high_cents: 30000,
        };
        let json = prediction.to_json();
        assert!(json.contains(r#""origin":"SIN""#));
        assert!(json.contains(r#""predicted_price_cents":25000"#));
    }

    #[test]
    fn test_insights_response_json() {
        let insights = InsightsResponse {
            origin: "SIN".into(),
            destination: "BKK".into(),
            cheapest_day: "Tuesday".into(),
            most_expensive_day: "Friday".into(),
            average_price_cents: 28000,
            price_volatility: "medium".into(),
            best_advance_days: 21,
            season: "peak".into(),
        };
        let json = insights.to_json();
        assert!(json.contains(r#""cheapest_day":"Tuesday""#));
        assert!(json.contains(r#""best_advance_days":21"#));
    }
}
