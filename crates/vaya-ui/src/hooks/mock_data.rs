//! Mock Data Layer
//!
//! Provides realistic mock data for testing the complete booking flow.

use crate::types::*;

/// Generate mock flights for KUL-NRT route
pub fn mock_flights() -> Vec<Flight> {
    vec![
        Flight {
            id: "FL001".to_string(),
            airline: "MH".to_string(),
            airline_name: "Malaysia Airlines".to_string(),
            flight_number: "MH52".to_string(),
            origin: "KUL".to_string(),
            destination: "NRT".to_string(),
            departure_time: "08:30".to_string(),
            arrival_time: "16:45".to_string(),
            duration_minutes: 435,
            stops: 0,
            price: Price::myr(184700),
            cabin_class: "Economy".to_string(),
        },
        Flight {
            id: "FL002".to_string(),
            airline: "D7".to_string(),
            airline_name: "AirAsia X".to_string(),
            flight_number: "D7536".to_string(),
            origin: "KUL".to_string(),
            destination: "NRT".to_string(),
            departure_time: "23:55".to_string(),
            arrival_time: "08:10+1".to_string(),
            duration_minutes: 455,
            stops: 0,
            price: Price::myr(129900),
            cabin_class: "Economy".to_string(),
        },
        Flight {
            id: "FL003".to_string(),
            airline: "SQ".to_string(),
            airline_name: "Singapore Airlines".to_string(),
            flight_number: "SQ12".to_string(),
            origin: "KUL".to_string(),
            destination: "NRT".to_string(),
            departure_time: "14:20".to_string(),
            arrival_time: "23:35".to_string(),
            duration_minutes: 495,
            stops: 1,
            price: Price::myr(215600),
            cabin_class: "Economy".to_string(),
        },
        Flight {
            id: "FL004".to_string(),
            airline: "JL".to_string(),
            airline_name: "Japan Airlines".to_string(),
            flight_number: "JL724".to_string(),
            origin: "KUL".to_string(),
            destination: "NRT".to_string(),
            departure_time: "10:15".to_string(),
            arrival_time: "18:30".to_string(),
            duration_minutes: 435,
            stops: 0,
            price: Price::myr(245000),
            cabin_class: "Economy".to_string(),
        },
        Flight {
            id: "FL005".to_string(),
            airline: "ID".to_string(),
            airline_name: "Batik Air".to_string(),
            flight_number: "ID8756".to_string(),
            origin: "KUL".to_string(),
            destination: "NRT".to_string(),
            departure_time: "19:45".to_string(),
            arrival_time: "05:20+1".to_string(),
            duration_minutes: 515,
            stops: 1,
            price: Price::myr(115600),
            cabin_class: "Economy".to_string(),
        },
    ]
}

/// Generate mock flights based on origin/destination
pub fn mock_flights_for_route(origin: &str, destination: &str) -> Vec<Flight> {
    let mut flights = mock_flights();
    // Update origin/destination for all flights
    for flight in &mut flights {
        flight.origin = origin.to_string();
        flight.destination = destination.to_string();
    }
    flights
}

/// Generate oracle prediction based on route hash for deterministic testing
pub fn mock_oracle_prediction(route_hash: u32) -> OraclePrediction {
    match route_hash % 4 {
        0 => OraclePrediction {
            id: format!("oracle-{}", route_hash),
            verdict: OracleVerdict::BookNow,
            confidence: 94,
            current_price: Price::myr(129900),
            predicted_price: Some(Price::myr(145600)),
            wait_days: None,
            price_trend: Some(PriceTrend::Rising),
            reasoning: vec![
                "Price is 12% below 90-day average".to_string(),
                "Historical data shows prices increase 89% of the time within 7 days".to_string(),
                "Current demand is moderate for this route".to_string(),
            ],
        },
        1 => OraclePrediction {
            id: format!("oracle-{}", route_hash),
            verdict: OracleVerdict::Wait,
            confidence: 78,
            current_price: Price::myr(184700),
            predicted_price: Some(Price::myr(152300)),
            wait_days: Some(12),
            price_trend: Some(PriceTrend::Falling),
            reasoning: vec![
                "Prices typically drop 2 weeks before departure for this route".to_string(),
                "Expected savings of RM324 (17%)".to_string(),
                "Low season approaching".to_string(),
            ],
        },
        2 => OraclePrediction {
            id: format!("oracle-{}", route_hash),
            verdict: OracleVerdict::JoinPool,
            confidence: 85,
            current_price: Price::myr(215600),
            predicted_price: Some(Price::myr(178900)),
            wait_days: None,
            price_trend: Some(PriceTrend::Stable),
            reasoning: vec![
                "23 travelers want this route".to_string(),
                "Demand pool could unlock group discount of up to 17%".to_string(),
                "Pool closes in 3 days".to_string(),
            ],
        },
        _ => OraclePrediction {
            id: format!("oracle-{}", route_hash),
            verdict: OracleVerdict::Uncertain,
            confidence: 45,
            current_price: Price::myr(165000),
            predicted_price: None,
            wait_days: None,
            price_trend: None,
            reasoning: vec![
                "Limited historical data for this route".to_string(),
                "We recommend comparing options manually".to_string(),
            ],
        },
    }
}

/// Generate mock FPX banks list
pub fn mock_fpx_banks() -> Vec<FpxBank> {
    FpxBank::mock_banks()
}

/// Generate price lock for a flight
pub fn mock_price_lock(flight: &Flight, duration: PriceLockDuration) -> PriceLock {
    let (hours, fee) = match duration {
        PriceLockDuration::Hours24 => (24, 0),
        PriceLockDuration::Hours48 => (48, 1500), // RM15 in sen
        PriceLockDuration::Hours72 => (72, 2500), // RM25 in sen
    };

    PriceLock {
        id: format!("PL{}", js_sys::Date::now() as u64),
        flight_id: flight.id.clone(),
        locked_price: flight.price.amount,
        duration,
        expires_at: format_expiry_time(hours),
        fee,
    }
}

/// Generate a booking reference code
pub fn mock_booking_reference() -> String {
    let chars: Vec<char> = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars().collect();
    let now = js_sys::Date::now() as u64;
    format!("VY{}", (0..6).map(|i| chars[((now >> (i * 5)) % 32) as usize]).collect::<String>())
}

/// Format expiry time as ISO string
fn format_expiry_time(hours: u32) -> String {
    let now = js_sys::Date::now();
    let expiry = now + (hours as f64 * 60.0 * 60.0 * 1000.0);
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(expiry));
    date.to_iso_string().as_string().unwrap_or_default()
}

/// Simple hash function for route strings
pub fn route_hash(origin: &str, destination: &str) -> u32 {
    let mut hash: u32 = 0;
    for c in origin.chars().chain(destination.chars()) {
        hash = hash.wrapping_mul(31).wrapping_add(c as u32);
    }
    hash
}
