//! Search handlers

use vaya_api::{ApiError, ApiResult, JsonSerialize, Request, Response};

/// Search flights
pub fn search_flights(req: &Request) -> ApiResult<Response> {
    // Parse request body for search criteria
    let body = req.body_string().ok_or(ApiError::BadRequest("Missing request body".into()))?;

    // TODO: Parse JSON body and perform search
    // For now, return mock response

    let response_body = SearchFlightsResponse {
        results: vec![],
        total: 0,
        search_id: generate_search_id(),
    };

    let mut response = Response::ok();
    response.set_json_body(&response_body);
    Ok(response)
}

/// Search airports
pub fn search_airports(req: &Request) -> ApiResult<Response> {
    let query = req.query("q").cloned().unwrap_or_default();

    if query.len() < 2 {
        return Err(ApiError::BadRequest("Query must be at least 2 characters".into()));
    }

    // TODO: Search airports
    let response_body = SearchAirportsResponse { airports: vec![] };

    let mut response = Response::ok();
    response.set_json_body(&response_body);
    Ok(response)
}

/// Search airlines
pub fn search_airlines(req: &Request) -> ApiResult<Response> {
    let query = req.query("q").cloned().unwrap_or_default();

    if query.is_empty() {
        return Err(ApiError::BadRequest("Query parameter 'q' is required".into()));
    }

    // TODO: Search airlines
    let response_body = SearchAirlinesResponse { airlines: vec![] };

    let mut response = Response::ok();
    response.set_json_body(&response_body);
    Ok(response)
}

/// Flight search response
#[derive(Debug, Clone)]
pub struct SearchFlightsResponse {
    pub results: Vec<FlightResult>,
    pub total: u64,
    pub search_id: String,
}

impl JsonSerialize for SearchFlightsResponse {
    fn to_json(&self) -> String {
        let results: Vec<String> = self.results.iter().map(|r| r.to_json()).collect();
        format!(
            r#"{{"results":[{}],"total":{},"search_id":"{}"}}"#,
            results.join(","),
            self.total,
            self.search_id
        )
    }
}

/// Individual flight result
#[derive(Debug, Clone)]
pub struct FlightResult {
    pub id: String,
    pub origin: String,
    pub destination: String,
    pub departure: String,
    pub arrival: String,
    pub price_cents: i64,
    pub currency: String,
    pub airline: String,
    pub stops: u8,
}

impl JsonSerialize for FlightResult {
    fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","origin":"{}","destination":"{}","departure":"{}","arrival":"{}","price_cents":{},"currency":"{}","airline":"{}","stops":{}}}"#,
            self.id, self.origin, self.destination, self.departure, self.arrival,
            self.price_cents, self.currency, self.airline, self.stops
        )
    }
}

/// Airport search response
#[derive(Debug, Clone)]
pub struct SearchAirportsResponse {
    pub airports: Vec<AirportResult>,
}

impl JsonSerialize for SearchAirportsResponse {
    fn to_json(&self) -> String {
        let airports: Vec<String> = self.airports.iter().map(|a| a.to_json()).collect();
        format!(r#"{{"airports":[{}]}}"#, airports.join(","))
    }
}

/// Airport result
#[derive(Debug, Clone)]
pub struct AirportResult {
    pub code: String,
    pub name: String,
    pub city: String,
    pub country: String,
}

impl JsonSerialize for AirportResult {
    fn to_json(&self) -> String {
        format!(
            r#"{{"code":"{}","name":"{}","city":"{}","country":"{}"}}"#,
            self.code, self.name, self.city, self.country
        )
    }
}

/// Airline search response
#[derive(Debug, Clone)]
pub struct SearchAirlinesResponse {
    pub airlines: Vec<AirlineResult>,
}

impl JsonSerialize for SearchAirlinesResponse {
    fn to_json(&self) -> String {
        let airlines: Vec<String> = self.airlines.iter().map(|a| a.to_json()).collect();
        format!(r#"{{"airlines":[{}]}}"#, airlines.join(","))
    }
}

/// Airline result
#[derive(Debug, Clone)]
pub struct AirlineResult {
    pub code: String,
    pub name: String,
}

impl JsonSerialize for AirlineResult {
    fn to_json(&self) -> String {
        format!(r#"{{"code":"{}","name":"{}"}}"#, self.code, self.name)
    }
}

/// Generate unique search ID
fn generate_search_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("srch-{:x}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_airports_short_query() {
        let mut req = Request::new("GET", "/search/airports");
        req.query_params.insert("q".into(), "a".into());
        let result = search_airports(&req);
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_search_airlines_missing_query() {
        let req = Request::new("GET", "/search/airlines");
        let result = search_airlines(&req);
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_flight_result_json() {
        let result = FlightResult {
            id: "FL001".into(),
            origin: "SIN".into(),
            destination: "BKK".into(),
            departure: "2026-01-15T08:00:00Z".into(),
            arrival: "2026-01-15T09:30:00Z".into(),
            price_cents: 15000,
            currency: "SGD".into(),
            airline: "SQ".into(),
            stops: 0,
        };
        let json = result.to_json();
        assert!(json.contains(r#""origin":"SIN""#));
        assert!(json.contains(r#""price_cents":15000"#));
    }
}
