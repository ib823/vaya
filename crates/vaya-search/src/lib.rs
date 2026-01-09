//! Flight search engine and fare aggregation
//!
//! This crate provides:
//! - Flight search request/response types
//! - Search filtering and sorting
//! - Multi-provider aggregation
//! - Result caching
//!
//! # Example
//!
//! ```
//! use vaya_search::{SearchRequest, SearchEngine};
//! use vaya_common::IataCode;
//! use time::Date;
//!
//! let origin = IataCode::SIN;
//! let dest = IataCode::NRT;
//! let date = Date::from_calendar_date(2025, time::Month::January, 15).unwrap();
//!
//! let request = SearchRequest::one_way(origin, dest, date);
//! assert!(request.validate().is_ok());
//!
//! let engine = SearchEngine::new();
//! // Add providers and search...
//! ```

pub mod engine;
pub mod error;
pub mod request;
pub mod types;

pub use engine::{SearchEngine, SearchEngineConfig, SearchProvider, SearchResponse};
pub use error::{SearchError, SearchResult};
pub use request::{Alliance, SearchFilters, SearchRequest, SortBy, SortOrder};
pub use types::{
    BaggageAllowance, CabinClass, FlightLeg, FlightOffer, FlightSegment, PassengerType,
    Passengers, PriceBreakdown, TripType,
};
