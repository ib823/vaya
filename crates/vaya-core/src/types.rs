//! Core business types

use vaya_common::{AirlineCode, CurrencyCode, IataCode, Price, Timestamp};

/// Passenger type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassengerType {
    /// Adult (12+ years)
    Adult,
    /// Child (2-11 years)
    Child,
    /// Infant (0-2 years)
    Infant,
}

impl PassengerType {
    /// Get passenger type code
    pub fn code(&self) -> &'static str {
        match self {
            Self::Adult => "ADT",
            Self::Child => "CHD",
            Self::Infant => "INF",
        }
    }
}

/// Passenger count for search
#[derive(Debug, Clone, Copy)]
pub struct PassengerCount {
    /// Number of adults
    pub adults: u8,
    /// Number of children
    pub children: u8,
    /// Number of infants
    pub infants: u8,
}

impl PassengerCount {
    /// Create new passenger count with adults only
    pub fn adults(count: u8) -> Self {
        Self {
            adults: count,
            children: 0,
            infants: 0,
        }
    }

    /// Total number of passengers
    pub fn total(&self) -> u8 {
        self.adults + self.children + self.infants
    }

    /// Validate passenger counts
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.adults == 0 {
            return Err("At least one adult is required");
        }
        if self.infants > self.adults {
            return Err("Number of infants cannot exceed adults");
        }
        if self.total() > 9 {
            return Err("Maximum 9 passengers per booking");
        }
        Ok(())
    }
}

impl Default for PassengerCount {
    fn default() -> Self {
        Self::adults(1)
    }
}

/// Cabin class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CabinClass {
    /// Economy class
    Economy,
    /// Premium economy
    PremiumEconomy,
    /// Business class
    Business,
    /// First class
    First,
}

impl CabinClass {
    /// Get cabin class code
    pub fn code(&self) -> &'static str {
        match self {
            Self::Economy => "Y",
            Self::PremiumEconomy => "W",
            Self::Business => "C",
            Self::First => "F",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Economy => "Economy",
            Self::PremiumEconomy => "Premium Economy",
            Self::Business => "Business",
            Self::First => "First",
        }
    }
}

impl Default for CabinClass {
    fn default() -> Self {
        Self::Economy
    }
}

/// Trip type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TripType {
    /// One-way trip
    OneWay,
    /// Round trip
    RoundTrip,
    /// Multi-city
    MultiCity,
}

impl Default for TripType {
    fn default() -> Self {
        Self::RoundTrip
    }
}

/// Search request
#[derive(Debug, Clone)]
pub struct SearchRequest {
    /// Origin airport code
    pub origin: IataCode,
    /// Destination airport code
    pub destination: IataCode,
    /// Departure date (YYYY-MM-DD)
    pub departure_date: String,
    /// Return date (YYYY-MM-DD) for round trips
    pub return_date: Option<String>,
    /// Trip type
    pub trip_type: TripType,
    /// Passengers
    pub passengers: PassengerCount,
    /// Preferred cabin class
    pub cabin_class: CabinClass,
    /// Currency for pricing
    pub currency: CurrencyCode,
    /// Direct flights only
    pub direct_only: bool,
    /// Flexible dates (+/- 3 days)
    pub flexible_dates: bool,
    /// Maximum number of results
    pub max_results: Option<u16>,
}

impl SearchRequest {
    /// Create a new one-way search request
    pub fn one_way(
        origin: IataCode,
        destination: IataCode,
        departure_date: &str,
    ) -> Self {
        Self {
            origin,
            destination,
            departure_date: departure_date.to_string(),
            return_date: None,
            trip_type: TripType::OneWay,
            passengers: PassengerCount::default(),
            cabin_class: CabinClass::default(),
            currency: CurrencyCode::MYR,
            direct_only: false,
            flexible_dates: false,
            max_results: Some(50),
        }
    }

    /// Create a new round-trip search request
    pub fn round_trip(
        origin: IataCode,
        destination: IataCode,
        departure_date: &str,
        return_date: &str,
    ) -> Self {
        Self {
            origin,
            destination,
            departure_date: departure_date.to_string(),
            return_date: Some(return_date.to_string()),
            trip_type: TripType::RoundTrip,
            passengers: PassengerCount::default(),
            cabin_class: CabinClass::default(),
            currency: CurrencyCode::MYR,
            direct_only: false,
            flexible_dates: false,
            max_results: Some(50),
        }
    }

    /// Set passengers
    pub fn with_passengers(mut self, passengers: PassengerCount) -> Self {
        self.passengers = passengers;
        self
    }

    /// Set cabin class
    pub fn with_cabin(mut self, cabin: CabinClass) -> Self {
        self.cabin_class = cabin;
        self
    }

    /// Set currency
    pub fn with_currency(mut self, currency: CurrencyCode) -> Self {
        self.currency = currency;
        self
    }

    /// Set direct only
    pub fn direct_only(mut self) -> Self {
        self.direct_only = true;
        self
    }

    /// Validate the search request
    pub fn validate(&self) -> Result<(), String> {
        if self.origin == self.destination {
            return Err("Origin and destination must be different".to_string());
        }

        self.passengers
            .validate()
            .map_err(|e| e.to_string())?;

        if self.trip_type == TripType::RoundTrip && self.return_date.is_none() {
            return Err("Return date required for round-trip".to_string());
        }

        Ok(())
    }
}

/// Search result flight offer
#[derive(Debug, Clone)]
pub struct FlightOffer {
    /// Unique offer ID
    pub id: String,
    /// Operating airlines
    pub airlines: Vec<AirlineCode>,
    /// Outbound segments
    pub outbound: FlightJourney,
    /// Return segments (for round trips)
    pub inbound: Option<FlightJourney>,
    /// Total price
    pub price: Price,
    /// Price per passenger type
    pub price_breakdown: Vec<PricePerPassenger>,
    /// Fare conditions
    pub fare_conditions: FareConditions,
    /// Cabin class
    pub cabin_class: CabinClass,
    /// Seats remaining (if available)
    pub seats_remaining: Option<u8>,
    /// Is refundable
    pub refundable: bool,
    /// Baggage included
    pub baggage_included: BaggageAllowance,
    /// Offer expiry timestamp
    pub expires_at: Timestamp,
    /// Source GDS
    pub source: String,
}

/// Flight journey (one direction)
#[derive(Debug, Clone)]
pub struct FlightJourney {
    /// Segments in order
    pub segments: Vec<FlightSegment>,
    /// Total duration in minutes
    pub duration_minutes: u32,
    /// Number of stops
    pub stops: u8,
}

impl FlightJourney {
    /// Get departure time of first segment
    pub fn departure_time(&self) -> Option<&str> {
        self.segments.first().map(|s| s.departure_time.as_str())
    }

    /// Get arrival time of last segment
    pub fn arrival_time(&self) -> Option<&str> {
        self.segments.last().map(|s| s.arrival_time.as_str())
    }
}

/// Single flight segment
#[derive(Debug, Clone)]
pub struct FlightSegment {
    /// Segment ID
    pub id: String,
    /// Marketing carrier
    pub airline: AirlineCode,
    /// Flight number
    pub flight_number: String,
    /// Operating carrier (if different)
    pub operating_carrier: Option<AirlineCode>,
    /// Departure airport
    pub origin: IataCode,
    /// Departure time (ISO 8601)
    pub departure_time: String,
    /// Departure terminal
    pub departure_terminal: Option<String>,
    /// Arrival airport
    pub destination: IataCode,
    /// Arrival time (ISO 8601)
    pub arrival_time: String,
    /// Arrival terminal
    pub arrival_terminal: Option<String>,
    /// Duration in minutes
    pub duration_minutes: u32,
    /// Aircraft type
    pub aircraft: Option<String>,
    /// Cabin class
    pub cabin_class: CabinClass,
    /// Booking class
    pub booking_class: String,
}

/// Price per passenger type
#[derive(Debug, Clone)]
pub struct PricePerPassenger {
    /// Passenger type
    pub passenger_type: PassengerType,
    /// Number of passengers
    pub count: u8,
    /// Price per passenger
    pub price_per_passenger: Price,
    /// Total for this type
    pub total: Price,
}

/// Fare conditions
#[derive(Debug, Clone)]
pub struct FareConditions {
    /// Cancellation policy
    pub cancellation: String,
    /// Change policy
    pub changes: String,
    /// Refund policy
    pub refund: String,
    /// Fare family name
    pub fare_family: Option<String>,
}

/// Baggage allowance
#[derive(Debug, Clone)]
pub struct BaggageAllowance {
    /// Cabin baggage
    pub cabin: String,
    /// Checked baggage
    pub checked: String,
    /// Extra baggage cost
    pub extra_cost: Option<Price>,
}

/// Booking status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookingStatus {
    /// Pending payment
    PendingPayment,
    /// Payment processing
    PaymentProcessing,
    /// Confirmed
    Confirmed,
    /// Ticketed
    Ticketed,
    /// Cancelled
    Cancelled,
    /// Completed (flight taken)
    Completed,
    /// Refund pending
    RefundPending,
    /// Refunded
    Refunded,
}

impl BookingStatus {
    /// Check if booking can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::PendingPayment | Self::Confirmed | Self::Ticketed)
    }

    /// Check if booking can be modified
    pub fn can_modify(&self) -> bool {
        matches!(self, Self::Confirmed | Self::Ticketed)
    }

    /// Is terminal status
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Cancelled | Self::Completed | Self::Refunded)
    }
}

/// Passenger details for booking
#[derive(Debug, Clone)]
pub struct PassengerDetails {
    /// Passenger type
    pub passenger_type: PassengerType,
    /// Title (Mr, Mrs, Ms, etc.)
    pub title: String,
    /// First name
    pub first_name: String,
    /// Last name
    pub last_name: String,
    /// Date of birth (YYYY-MM-DD)
    pub date_of_birth: String,
    /// Gender
    pub gender: Gender,
    /// Nationality (country code)
    pub nationality: String,
    /// Passport number (optional)
    pub passport_number: Option<String>,
    /// Passport expiry (YYYY-MM-DD)
    pub passport_expiry: Option<String>,
    /// Email
    pub email: Option<String>,
    /// Phone
    pub phone: Option<String>,
    /// Frequent flyer number
    pub frequent_flyer: Option<FrequentFlyer>,
    /// Special requests
    pub special_requests: Vec<String>,
}

/// Gender
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    /// Male
    Male,
    /// Female
    Female,
    /// Other/Unspecified
    Other,
}

/// Frequent flyer info
#[derive(Debug, Clone)]
pub struct FrequentFlyer {
    /// Airline
    pub airline: AirlineCode,
    /// Number
    pub number: String,
}

/// Contact details for booking
#[derive(Debug, Clone)]
pub struct ContactDetails {
    /// Email
    pub email: String,
    /// Phone
    pub phone: String,
    /// Emergency contact name
    pub emergency_contact_name: Option<String>,
    /// Emergency contact phone
    pub emergency_contact_phone: Option<String>,
}

/// Booking request
#[derive(Debug, Clone)]
pub struct BookingRequest {
    /// Offer ID to book
    pub offer_id: String,
    /// User ID
    pub user_id: String,
    /// Passengers
    pub passengers: Vec<PassengerDetails>,
    /// Contact details
    pub contact: ContactDetails,
    /// Special remarks
    pub remarks: Option<String>,
}

/// Booking confirmation
#[derive(Debug, Clone)]
pub struct Booking {
    /// Booking ID
    pub id: String,
    /// PNR/Record locator
    pub pnr: String,
    /// User ID
    pub user_id: String,
    /// Status
    pub status: BookingStatus,
    /// Flight details
    pub flights: FlightOffer,
    /// Passengers
    pub passengers: Vec<PassengerDetails>,
    /// Contact
    pub contact: ContactDetails,
    /// Total price
    pub total_price: Price,
    /// Payment ID
    pub payment_id: Option<String>,
    /// Created at
    pub created_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
    /// Payment deadline
    pub payment_deadline: Option<Timestamp>,
    /// Ticket numbers (after ticketing)
    pub ticket_numbers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passenger_count_validation() {
        let valid = PassengerCount {
            adults: 2,
            children: 1,
            infants: 1,
        };
        assert!(valid.validate().is_ok());
        assert_eq!(valid.total(), 4);

        let no_adults = PassengerCount {
            adults: 0,
            children: 1,
            infants: 0,
        };
        assert!(no_adults.validate().is_err());

        let too_many_infants = PassengerCount {
            adults: 1,
            children: 0,
            infants: 2,
        };
        assert!(too_many_infants.validate().is_err());
    }

    #[test]
    fn test_search_request() {
        let search = SearchRequest::round_trip(
            IataCode::KUL,
            IataCode::SIN,
            "2025-06-15",
            "2025-06-20",
        )
        .with_passengers(PassengerCount::adults(2))
        .with_cabin(CabinClass::Business);

        assert!(search.validate().is_ok());
        assert_eq!(search.passengers.adults, 2);
        assert_eq!(search.cabin_class, CabinClass::Business);
    }

    #[test]
    fn test_booking_status() {
        assert!(BookingStatus::Confirmed.can_cancel());
        assert!(BookingStatus::Confirmed.can_modify());
        assert!(!BookingStatus::Cancelled.can_cancel());
        assert!(BookingStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_cabin_class() {
        assert_eq!(CabinClass::Business.code(), "C");
        assert_eq!(CabinClass::Economy.display_name(), "Economy");
    }
}
