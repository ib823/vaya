//! Search request types

use time::Date;
use vaya_common::{AirlineCode, IataCode};

use crate::types::{CabinClass, Passengers, TripType};
use crate::{SearchError, SearchResult};

/// A search request
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// Trip type
    pub trip_type: TripType,
    /// Origin airport(s)
    pub origins: Vec<IataCode>,
    /// Destination airport(s)
    pub destinations: Vec<IataCode>,
    /// Outbound date
    pub departure_date: Date,
    /// Return date (for round trips)
    pub return_date: Option<Date>,
    /// Passengers
    pub passengers: Passengers,
    /// Cabin class
    pub cabin: CabinClass,
    /// Filters
    pub filters: SearchFilters,
    /// Maximum results to return
    pub max_results: Option<usize>,
}

impl SearchRequest {
    /// Create a one-way search
    pub fn one_way(origin: IataCode, destination: IataCode, date: Date) -> Self {
        Self {
            trip_type: TripType::OneWay,
            origins: vec![origin],
            destinations: vec![destination],
            departure_date: date,
            return_date: None,
            passengers: Passengers::default(),
            cabin: CabinClass::Economy,
            filters: SearchFilters::default(),
            max_results: None,
        }
    }

    /// Create a round-trip search
    pub fn round_trip(
        origin: IataCode,
        destination: IataCode,
        departure: Date,
        return_date: Date,
    ) -> Self {
        Self {
            trip_type: TripType::RoundTrip,
            origins: vec![origin],
            destinations: vec![destination],
            departure_date: departure,
            return_date: Some(return_date),
            passengers: Passengers::default(),
            cabin: CabinClass::Economy,
            filters: SearchFilters::default(),
            max_results: None,
        }
    }

    /// Set passengers
    pub fn with_passengers(mut self, passengers: Passengers) -> Self {
        self.passengers = passengers;
        self
    }

    /// Set cabin class
    pub fn with_cabin(mut self, cabin: CabinClass) -> Self {
        self.cabin = cabin;
        self
    }

    /// Set filters
    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = filters;
        self
    }

    /// Set max results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    /// Validate the search request
    pub fn validate(&self) -> SearchResult<()> {
        // Check origins
        if self.origins.is_empty() {
            return Err(SearchError::InvalidParams("No origin specified".into()));
        }

        // Check destinations
        if self.destinations.is_empty() {
            return Err(SearchError::InvalidParams(
                "No destination specified".into(),
            ));
        }

        // Check same origin and destination
        if self.origins.len() == 1
            && self.destinations.len() == 1
            && self.origins[0].as_str() == self.destinations[0].as_str()
        {
            return Err(SearchError::InvalidRoute(
                "Origin and destination cannot be the same".into(),
            ));
        }

        // Check passengers
        if !self.passengers.validate() {
            return Err(SearchError::InvalidParams("Invalid passenger count".into()));
        }

        // Check return date for round trips
        if self.trip_type == TripType::RoundTrip {
            match self.return_date {
                None => {
                    return Err(SearchError::InvalidParams(
                        "Return date required for round trip".into(),
                    ));
                }
                Some(ret) if ret < self.departure_date => {
                    return Err(SearchError::InvalidDateRange);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Generate cache key for this request
    pub fn cache_key(&self) -> String {
        let origins: Vec<&str> = self.origins.iter().map(|a: &IataCode| a.as_str()).collect();
        let dests: Vec<&str> = self
            .destinations
            .iter()
            .map(|a: &IataCode| a.as_str())
            .collect();

        format!(
            "search:{}:{}:{}:{}:{}:{}:{}",
            origins.join(","),
            dests.join(","),
            self.departure_date,
            self.return_date.map(|d| d.to_string()).unwrap_or_default(),
            self.cabin.code(),
            self.passengers.adults,
            self.passengers.children + self.passengers.infants
        )
    }
}

/// Search filters
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    /// Maximum stops (None = any)
    pub max_stops: Option<u8>,
    /// Preferred airlines
    pub airlines: Vec<AirlineCode>,
    /// Excluded airlines
    pub exclude_airlines: Vec<AirlineCode>,
    /// Maximum price
    pub max_price: Option<i64>,
    /// Minimum departure time
    pub min_departure_time: Option<time::Time>,
    /// Maximum departure time
    pub max_departure_time: Option<time::Time>,
    /// Minimum arrival time
    pub min_arrival_time: Option<time::Time>,
    /// Maximum arrival time
    pub max_arrival_time: Option<time::Time>,
    /// Maximum duration in minutes
    pub max_duration: Option<u16>,
    /// Require refundable fares
    pub refundable_only: bool,
    /// Require flexible fares
    pub flexible_only: bool,
    /// Include nearby airports
    pub include_nearby: bool,
    /// Preferred connection airports
    pub via_airports: Vec<IataCode>,
    /// Alliance filter
    pub alliance: Option<Alliance>,
}

/// Airline alliance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alliance {
    StarAlliance,
    Oneworld,
    SkyTeam,
}

impl Alliance {
    /// Get alliance name
    pub fn name(&self) -> &'static str {
        match self {
            Alliance::StarAlliance => "Star Alliance",
            Alliance::Oneworld => "oneworld",
            Alliance::SkyTeam => "SkyTeam",
        }
    }
}

impl SearchFilters {
    /// Create filters for direct flights only
    pub fn direct() -> Self {
        Self {
            max_stops: Some(0),
            ..Default::default()
        }
    }

    /// Set maximum stops
    pub fn max_stops(mut self, stops: u8) -> Self {
        self.max_stops = Some(stops);
        self
    }

    /// Add preferred airline
    pub fn airline(mut self, airline: AirlineCode) -> Self {
        self.airlines.push(airline);
        self
    }

    /// Set maximum price
    pub fn max_price(mut self, price: i64) -> Self {
        self.max_price = Some(price);
        self
    }

    /// Set refundable only
    pub fn refundable(mut self) -> Self {
        self.refundable_only = true;
        self
    }

    /// Check if a number of stops passes the filter
    pub fn passes_stops(&self, stops: usize) -> bool {
        match self.max_stops {
            Some(max) => stops <= max as usize,
            None => true,
        }
    }

    /// Check if a price passes the filter
    pub fn passes_price(&self, price: i64) -> bool {
        match self.max_price {
            Some(max) => price <= max,
            None => true,
        }
    }

    /// Check if an airline passes the filter
    pub fn passes_airline(&self, airline: &AirlineCode) -> bool {
        // Check exclusion first
        if self
            .exclude_airlines
            .iter()
            .any(|a| a.as_str() == airline.as_str())
        {
            return false;
        }

        // If no preferred airlines, all pass
        if self.airlines.is_empty() {
            return true;
        }

        // Check if in preferred list
        self.airlines.iter().any(|a| a.as_str() == airline.as_str())
    }

    /// Check if duration passes filter
    pub fn passes_duration(&self, minutes: u16) -> bool {
        match self.max_duration {
            Some(max) => minutes <= max,
            None => true,
        }
    }
}

/// Sort options for results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    #[default]
    Price,
    Duration,
    Departure,
    Arrival,
    Stops,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Ascending,
    Descending,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_way_request() {
        let origin = IataCode::SIN;
        let dest = IataCode::NRT;
        let date = Date::from_calendar_date(2025, time::Month::January, 15).unwrap();

        let request = SearchRequest::one_way(origin, dest, date);

        assert_eq!(request.trip_type, TripType::OneWay);
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_round_trip_request() {
        let origin = IataCode::SIN;
        let dest = IataCode::NRT;
        let dep = Date::from_calendar_date(2025, time::Month::January, 15).unwrap();
        let ret = Date::from_calendar_date(2025, time::Month::January, 22).unwrap();

        let request = SearchRequest::round_trip(origin, dest, dep, ret);

        assert_eq!(request.trip_type, TripType::RoundTrip);
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_invalid_return_date() {
        let origin = IataCode::SIN;
        let dest = IataCode::NRT;
        let dep = Date::from_calendar_date(2025, time::Month::January, 22).unwrap();
        let ret = Date::from_calendar_date(2025, time::Month::January, 15).unwrap();

        let request = SearchRequest::round_trip(origin, dest, dep, ret);

        assert!(request.validate().is_err());
    }

    #[test]
    fn test_filters_stops() {
        let filters = SearchFilters::direct();
        assert!(filters.passes_stops(0));
        assert!(!filters.passes_stops(1));

        let filters2 = SearchFilters::default().max_stops(1);
        assert!(filters2.passes_stops(0));
        assert!(filters2.passes_stops(1));
        assert!(!filters2.passes_stops(2));
    }

    #[test]
    fn test_filters_price() {
        let filters = SearchFilters::default().max_price(50000);
        assert!(filters.passes_price(49999));
        assert!(filters.passes_price(50000));
        assert!(!filters.passes_price(50001));
    }

    #[test]
    fn test_cache_key() {
        let origin = IataCode::SIN;
        let dest = IataCode::NRT;
        let date = Date::from_calendar_date(2025, time::Month::January, 15).unwrap();

        let request = SearchRequest::one_way(origin, dest, date);
        let key = request.cache_key();

        assert!(key.contains("SIN"));
        assert!(key.contains("NRT"));
    }
}
