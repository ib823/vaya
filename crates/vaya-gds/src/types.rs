//! GDS types - Built on vaya-common types

use serde::{Deserialize, Serialize};
use vaya_common::{AirlineCode, CurrencyCode, Date, IataCode, MinorUnits, Price, Timestamp};

/// Cabin class for flights
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CabinClass {
    /// Economy class
    #[default]
    Economy,
    /// Premium economy
    PremiumEconomy,
    /// Business class
    Business,
    /// First class
    First,
}

impl CabinClass {
    /// Convert to Amadeus API format
    #[must_use]
    pub const fn to_amadeus_code(&self) -> &'static str {
        match self {
            Self::Economy => "ECONOMY",
            Self::PremiumEconomy => "PREMIUM_ECONOMY",
            Self::Business => "BUSINESS",
            Self::First => "FIRST",
        }
    }

    /// Display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Economy => "Economy",
            Self::PremiumEconomy => "Premium Economy",
            Self::Business => "Business",
            Self::First => "First",
        }
    }
}

/// Flight search request
#[derive(Debug, Clone)]
pub struct FlightSearchRequest {
    /// Origin airport
    pub origin: IataCode,
    /// Destination airport
    pub destination: IataCode,
    /// Departure date
    pub departure_date: Date,
    /// Return date (for round trip)
    pub return_date: Option<Date>,
    /// Number of adults (12+)
    pub adults: u8,
    /// Number of children (2-11)
    pub children: u8,
    /// Number of infants (0-2)
    pub infants: u8,
    /// Cabin class preference
    pub cabin_class: CabinClass,
    /// Direct flights only
    pub direct_only: bool,
    /// Maximum number of results
    pub max_results: u32,
    /// Preferred currency for prices
    pub currency: CurrencyCode,
}

impl Default for FlightSearchRequest {
    fn default() -> Self {
        Self {
            origin: IataCode::default(),
            destination: IataCode::default(),
            departure_date: Date::today(),
            return_date: None,
            adults: 1,
            children: 0,
            infants: 0,
            cabin_class: CabinClass::Economy,
            direct_only: false,
            max_results: 50,
            currency: CurrencyCode::MYR,
        }
    }
}

impl FlightSearchRequest {
    /// Create a new one-way search
    #[must_use]
    pub fn one_way(origin: IataCode, destination: IataCode, departure_date: Date) -> Self {
        Self {
            origin,
            destination,
            departure_date,
            ..Default::default()
        }
    }

    /// Create a round-trip search
    #[must_use]
    pub fn round_trip(
        origin: IataCode,
        destination: IataCode,
        departure_date: Date,
        return_date: Date,
    ) -> Self {
        Self {
            origin,
            destination,
            departure_date,
            return_date: Some(return_date),
            ..Default::default()
        }
    }

    /// Set number of passengers
    #[must_use]
    pub const fn with_passengers(mut self, adults: u8, children: u8, infants: u8) -> Self {
        self.adults = adults;
        self.children = children;
        self.infants = infants;
        self
    }

    /// Set cabin class
    #[must_use]
    pub const fn with_cabin(mut self, cabin: CabinClass) -> Self {
        self.cabin_class = cabin;
        self
    }

    /// Set direct only
    #[must_use]
    pub const fn direct_only(mut self, direct: bool) -> Self {
        self.direct_only = direct;
        self
    }

    /// Total passenger count
    #[must_use]
    pub const fn total_passengers(&self) -> u8 {
        self.adults + self.children + self.infants
    }

    /// Is round trip?
    #[must_use]
    pub const fn is_round_trip(&self) -> bool {
        self.return_date.is_some()
    }

    /// Generate cache key for this request
    #[must_use]
    pub fn cache_key(&self) -> String {
        format!(
            "search:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.origin,
            self.destination,
            self.departure_date,
            self.return_date.map_or("OW".to_string(), |d| d.to_string()),
            self.adults,
            self.children,
            self.infants,
            self.cabin_class.to_amadeus_code(),
            if self.direct_only { "D" } else { "C" }
        )
    }
}

/// Flight point (departure or arrival)
#[derive(Debug, Clone)]
pub struct FlightPoint {
    /// Airport code
    pub airport: IataCode,
    /// Terminal (if known)
    pub terminal: Option<String>,
    /// Date and time (UTC)
    pub datetime: Timestamp,
}

impl FlightPoint {
    /// Create new flight point
    #[must_use]
    pub fn new(airport: IataCode, datetime: Timestamp) -> Self {
        Self {
            airport,
            terminal: None,
            datetime,
        }
    }

    /// With terminal
    #[must_use]
    pub fn with_terminal(mut self, terminal: impl Into<String>) -> Self {
        self.terminal = Some(terminal.into());
        self
    }
}

/// Flight segment (one leg of a journey)
#[derive(Debug, Clone)]
pub struct FlightSegment {
    /// Departure info
    pub departure: FlightPoint,
    /// Arrival info
    pub arrival: FlightPoint,
    /// Operating airline
    pub airline: AirlineCode,
    /// Flight number
    pub flight_number: String,
    /// Duration in minutes
    pub duration_minutes: u32,
    /// Aircraft type (if known)
    pub aircraft: Option<String>,
    /// Cabin class
    pub cabin_class: CabinClass,
    /// Booking class code (e.g., "Y", "B", "M")
    pub booking_class: Option<String>,
    /// Number of stops
    pub stops: u8,
}

impl FlightSegment {
    /// Get flight designator (e.g., "MH123")
    #[must_use]
    pub fn designator(&self) -> String {
        format!("{}{}", self.airline, self.flight_number)
    }

    /// Get duration as hours and minutes string
    #[must_use]
    pub fn duration_display(&self) -> String {
        let hours = self.duration_minutes / 60;
        let minutes = self.duration_minutes % 60;
        if hours > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{minutes}m")
        }
    }

    /// Is direct flight?
    #[must_use]
    pub const fn is_direct(&self) -> bool {
        self.stops == 0
    }
}

/// Itinerary (outbound or return journey)
#[derive(Debug, Clone)]
pub struct Itinerary {
    /// Segments in this itinerary
    pub segments: Vec<FlightSegment>,
    /// Total duration in minutes
    pub total_duration_minutes: u32,
}

impl Itinerary {
    /// Get total number of stops
    #[must_use]
    pub fn total_stops(&self) -> usize {
        if self.segments.is_empty() {
            0
        } else {
            self.segments.len() - 1
                + self
                    .segments
                    .iter()
                    .map(|s| s.stops as usize)
                    .sum::<usize>()
        }
    }

    /// Is direct itinerary?
    #[must_use]
    pub fn is_direct(&self) -> bool {
        self.segments.len() == 1 && self.segments[0].is_direct()
    }

    /// Get departure point
    #[must_use]
    pub fn departure(&self) -> Option<&FlightPoint> {
        self.segments.first().map(|s| &s.departure)
    }

    /// Get arrival point
    #[must_use]
    pub fn arrival(&self) -> Option<&FlightPoint> {
        self.segments.last().map(|s| &s.arrival)
    }

    /// Get operating airlines
    #[must_use]
    pub fn airlines(&self) -> Vec<AirlineCode> {
        self.segments.iter().map(|s| s.airline).collect()
    }

    /// Duration display string
    #[must_use]
    pub fn duration_display(&self) -> String {
        let hours = self.total_duration_minutes / 60;
        let minutes = self.total_duration_minutes % 60;
        format!("{hours}h {minutes}m")
    }
}

/// Price breakdown
#[derive(Debug, Clone)]
pub struct PriceBreakdown {
    /// Base fare
    pub base: Price,
    /// Taxes
    pub taxes: Price,
    /// Fees and surcharges
    pub fees: Price,
    /// Total price
    pub total: Price,
    /// Price per adult
    pub per_adult: Price,
    /// Price per child (if applicable)
    pub per_child: Option<Price>,
    /// Price per infant (if applicable)
    pub per_infant: Option<Price>,
}

impl PriceBreakdown {
    /// Create simple breakdown (base + taxes = total)
    #[must_use]
    pub fn simple(base: Price, taxes: Price) -> Self {
        let total = base.add(&taxes).unwrap_or(base);
        Self {
            base,
            taxes,
            fees: Price::new(MinorUnits::ZERO, base.currency),
            total,
            per_adult: total,
            per_child: None,
            per_infant: None,
        }
    }
}

/// Flight offer from GDS
#[derive(Debug, Clone)]
pub struct FlightOffer {
    /// Unique offer ID (from GDS)
    pub id: String,
    /// Outbound itinerary
    pub outbound: Itinerary,
    /// Return itinerary (if round trip)
    pub return_itinerary: Option<Itinerary>,
    /// Price breakdown
    pub price: PriceBreakdown,
    /// Validating/ticketing airline
    pub validating_airline: AirlineCode,
    /// Available seats (if known)
    pub available_seats: Option<u32>,
    /// Offer creation timestamp
    pub created_at: Timestamp,
    /// Offer expiry timestamp
    pub expires_at: Option<Timestamp>,
    /// Is instant ticketing available
    pub instant_ticketing: bool,
    /// Fare rules (brief summary)
    pub fare_rules: Option<FareRules>,
}

impl FlightOffer {
    /// Check if offer is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| exp.is_past())
    }

    /// Check if offer is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Is round trip?
    #[must_use]
    pub const fn is_round_trip(&self) -> bool {
        self.return_itinerary.is_some()
    }

    /// Total stops (outbound + return)
    #[must_use]
    pub fn total_stops(&self) -> usize {
        self.outbound.total_stops()
            + self
                .return_itinerary
                .as_ref()
                .map_or(0, Itinerary::total_stops)
    }

    /// Is direct flight(s)?
    #[must_use]
    pub fn is_direct(&self) -> bool {
        self.outbound.is_direct()
            && self
                .return_itinerary
                .as_ref()
                .is_none_or(Itinerary::is_direct)
    }

    /// Get all airlines involved (unique)
    #[must_use]
    pub fn airlines(&self) -> Vec<AirlineCode> {
        let mut airlines = self.outbound.airlines();
        if let Some(ret) = &self.return_itinerary {
            airlines.extend(ret.airlines());
        }
        // Deduplicate without sorting (AirlineCode doesn't implement Ord)
        let mut seen = std::collections::HashSet::new();
        airlines.retain(|a| seen.insert(*a));
        airlines
    }
}

/// Brief fare rules
#[derive(Debug, Clone)]
pub struct FareRules {
    /// Is refundable
    pub refundable: bool,
    /// Is changeable
    pub changeable: bool,
    /// Change fee (if applicable)
    pub change_fee: Option<Price>,
    /// Cancellation fee (if applicable)
    pub cancellation_fee: Option<Price>,
    /// Baggage allowance
    pub baggage: Option<BaggageAllowance>,
}

/// Baggage allowance
#[derive(Debug, Clone)]
pub struct BaggageAllowance {
    /// Number of checked bags
    pub checked_bags: u8,
    /// Weight per bag (kg)
    pub weight_kg: Option<u32>,
    /// Carry-on allowed
    pub carry_on: bool,
}

/// Gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    /// Male
    Male,
    /// Female
    Female,
}

impl Gender {
    /// Get Amadeus code
    #[must_use]
    pub const fn amadeus_code(&self) -> &'static str {
        match self {
            Self::Male => "MALE",
            Self::Female => "FEMALE",
        }
    }
}

/// Passenger type for booking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassengerType {
    /// Adult (12+)
    Adult,
    /// Child (2-11)
    Child,
    /// Infant (0-2)
    Infant,
}

impl PassengerType {
    /// Amadeus code
    #[must_use]
    pub const fn amadeus_code(&self) -> &'static str {
        match self {
            Self::Adult => "ADT",
            Self::Child => "CHD",
            Self::Infant => "INF",
        }
    }
}

/// Passenger details for booking
#[derive(Debug, Clone)]
pub struct PassengerDetails {
    /// Passenger type
    pub passenger_type: PassengerType,
    /// Title
    pub title: String,
    /// First name (as in passport)
    pub first_name: String,
    /// Last name (as in passport)
    pub last_name: String,
    /// Date of birth
    pub date_of_birth: Date,
    /// Gender
    pub gender: Gender,
    /// Nationality (ISO 3166-1 alpha-2)
    pub nationality: Option<String>,
    /// Passport number
    pub passport_number: Option<String>,
    /// Passport expiry
    pub passport_expiry: Option<Date>,
    /// Passport issuing country
    pub passport_country: Option<String>,
    /// Email (for e-ticket)
    pub email: Option<String>,
    /// Phone
    pub phone: Option<String>,
}

impl PassengerDetails {
    /// Create adult passenger
    #[must_use]
    pub fn adult(first_name: impl Into<String>, last_name: impl Into<String>, dob: Date) -> Self {
        Self {
            passenger_type: PassengerType::Adult,
            title: "MR".to_string(),
            first_name: first_name.into(),
            last_name: last_name.into(),
            date_of_birth: dob,
            gender: Gender::Male,
            nationality: None,
            passport_number: None,
            passport_expiry: None,
            passport_country: None,
            email: None,
            phone: None,
        }
    }

    /// Full name
    #[must_use]
    pub fn full_name(&self) -> String {
        format!("{} {} {}", self.title, self.first_name, self.last_name)
    }

    /// Check if passport is valid for travel date
    #[must_use]
    pub fn is_passport_valid_for(&self, travel_date: Date) -> bool {
        self.passport_expiry.is_some_and(|exp| {
            // Passport should be valid at least 6 months after travel
            exp > travel_date.add_days(180)
        })
    }
}

/// Contact details for booking
#[derive(Debug, Clone)]
pub struct ContactDetails {
    /// Email
    pub email: String,
    /// Phone (with country code)
    pub phone: String,
    /// Emergency contact name
    pub emergency_name: Option<String>,
    /// Emergency contact phone
    pub emergency_phone: Option<String>,
}

impl ContactDetails {
    /// Create new contact
    #[must_use]
    pub fn new(email: impl Into<String>, phone: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            phone: phone.into(),
            emergency_name: None,
            emergency_phone: None,
        }
    }
}

/// Booking status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BookingStatus {
    /// Booking confirmed, awaiting payment
    Confirmed,
    /// Payment received
    Paid,
    /// Ticket issued
    Ticketed,
    /// Cancelled
    Cancelled,
    /// Pending (processing)
    Pending,
    /// Failed
    Failed,
}

impl BookingStatus {
    /// Is active booking?
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Confirmed | Self::Paid | Self::Ticketed)
    }

    /// Is terminal state?
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Cancelled | Self::Failed)
    }
}

/// Booking confirmation from GDS
#[derive(Debug, Clone)]
pub struct BookingConfirmation {
    /// PNR (Passenger Name Record) - airline reference
    pub pnr: String,
    /// Our booking reference
    pub booking_reference: String,
    /// Status
    pub status: BookingStatus,
    /// Created timestamp
    pub created_at: Timestamp,
    /// Ticketing deadline (if applicable)
    pub ticketing_deadline: Option<Timestamp>,
    /// Passengers in this booking
    pub passengers: Vec<String>,
    /// Flight offer that was booked
    pub offer_id: String,
}

impl BookingConfirmation {
    /// Check if ticketing is needed soon
    #[must_use]
    pub fn ticketing_urgent(&self) -> bool {
        self.ticketing_deadline.is_some_and(|deadline| {
            let hours_remaining = (deadline.as_unix() - Timestamp::now().as_unix()) / 3600;
            hours_remaining < 24
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cabin_class() {
        assert_eq!(CabinClass::Economy.to_amadeus_code(), "ECONOMY");
        assert_eq!(CabinClass::Business.display_name(), "Business");
    }

    #[test]
    fn test_flight_search_request() {
        let req = FlightSearchRequest::one_way(IataCode::KUL, IataCode::NRT, Date::today())
            .with_passengers(2, 1, 0)
            .with_cabin(CabinClass::Business)
            .direct_only(true);

        assert_eq!(req.total_passengers(), 3);
        assert!(!req.is_round_trip());
        assert!(req.direct_only);
    }

    #[test]
    fn test_cache_key() {
        let req = FlightSearchRequest::one_way(IataCode::KUL, IataCode::NRT, Date::today());
        let key = req.cache_key();
        assert!(key.starts_with("search:"));
        assert!(key.contains("KUL"));
        assert!(key.contains("NRT"));
    }

    #[test]
    fn test_flight_segment() {
        let segment = FlightSegment {
            departure: FlightPoint::new(IataCode::KUL, Timestamp::now()),
            arrival: FlightPoint::new(IataCode::NRT, Timestamp::now().add_hours(7)),
            airline: AirlineCode::MH,
            flight_number: "88".to_string(),
            duration_minutes: 420,
            aircraft: Some("A350".to_string()),
            cabin_class: CabinClass::Economy,
            booking_class: Some("Y".to_string()),
            stops: 0,
        };

        assert_eq!(segment.designator(), "MH88");
        assert_eq!(segment.duration_display(), "7h 0m");
        assert!(segment.is_direct());
    }

    #[test]
    fn test_passenger_passport_valid() {
        let mut pax = PassengerDetails::adult("John", "Doe", Date::new(1990, 1, 1));
        pax.passport_expiry = Some(Date::new(2028, 1, 1));

        let travel_date = Date::new(2025, 6, 1);
        assert!(pax.is_passport_valid_for(travel_date));

        let late_travel = Date::new(2027, 8, 1);
        assert!(!pax.is_passport_valid_for(late_travel));
    }
}
