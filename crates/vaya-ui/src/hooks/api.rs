//! API Integration
//!
//! HTTP client functions for communicating with the VAYA backend.

use gloo_net::http::Request;

use crate::hooks::config;
use crate::types::{ApiError, Flight, OraclePrediction, SearchRequest};

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// Search for flights
pub async fn search_flights(request: &SearchRequest) -> ApiResult<Vec<Flight>> {
    let url = format!(
        "{}/search?origin={}&destination={}&date={}&passengers={}&cabin={}",
        config::api_base(),
        request.origin,
        request.destination,
        request.departure_date,
        request.passengers,
        request.cabin_class
    );

    let response = Request::get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NETWORK_ERROR".to_string(),
            message: "Failed to connect to server".to_string(),
            details: Some(e.to_string()),
        })?;

    if response.ok() {
        response.json().await.map_err(|e| ApiError {
            code: "PARSE_ERROR".to_string(),
            message: "Failed to parse response".to_string(),
            details: Some(e.to_string()),
        })
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(ApiError {
            code: format!("HTTP_{}", status),
            message: format!("Request failed with status {}", status),
            details: Some(body),
        })
    }
}

/// Get Oracle prediction for a search
pub async fn get_oracle_prediction(search_id: &str) -> ApiResult<OraclePrediction> {
    let url = format!("{}/oracle/{}", config::api_base(), search_id);

    let response = Request::get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NETWORK_ERROR".to_string(),
            message: "Failed to connect to server".to_string(),
            details: Some(e.to_string()),
        })?;

    if response.ok() {
        response.json().await.map_err(|e| ApiError {
            code: "PARSE_ERROR".to_string(),
            message: "Failed to parse response".to_string(),
            details: Some(e.to_string()),
        })
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(ApiError {
            code: format!("HTTP_{}", status),
            message: format!("Request failed with status {}", status),
            details: Some(body),
        })
    }
}

/// Get flight details by ID
pub async fn get_flight(flight_id: &str) -> ApiResult<Flight> {
    let url = format!("{}/flights/{}", config::api_base(), flight_id);

    let response = Request::get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NETWORK_ERROR".to_string(),
            message: "Failed to connect to server".to_string(),
            details: Some(e.to_string()),
        })?;

    if response.ok() {
        response.json().await.map_err(|e| ApiError {
            code: "PARSE_ERROR".to_string(),
            message: "Failed to parse response".to_string(),
            details: Some(e.to_string()),
        })
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(ApiError {
            code: format!("HTTP_{}", status),
            message: format!("Request failed with status {}", status),
            details: Some(body),
        })
    }
}

/// Health check - verify API is reachable
pub async fn health_check() -> ApiResult<bool> {
    let url = format!("{}/health", config::api_base());

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NETWORK_ERROR".to_string(),
            message: "Failed to connect to server".to_string(),
            details: Some(e.to_string()),
        })?;

    Ok(response.ok())
}
