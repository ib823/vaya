//! Core types for flight search

use time::{Date, Duration, Time};
use vaya_common::{AirlineCode, CurrencyCode, IataCode, MinorUnits};

/// Cabin class
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CabinClass {
    Economy,
    PremiumEconomy,
    Business,
    First,
}

impl CabinClass {
    /// Get cabin code
    pub fn code(&self) -> char {
        match self {
            CabinClass::Economy => 'Y',
            CabinClass::PremiumEconomy => 'W',
            CabinClass::Business => 'C',
            CabinClass::First => 'F',
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            CabinClass::Economy => "Economy",
            CabinClass::PremiumEconomy => "Premium Economy",
            CabinClass::Business => "Business",
            CabinClass::First => "First",
        }
    }
}

/// Trip type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TripType {
    OneWay,
    RoundTrip,
    MultiCity,
}

/// Passenger type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassengerType {
    Adult,
    Child,
    Infant,
}

/// Passenger count
#[derive(Debug, Clone, Copy)]
pub struct Passengers {
    pub adults: u8,
    pub children: u8,
    pub infants: u8,
}

impl Default for Passengers {
    fn default() -> Self {
        Self {
            adults: 1,
            children: 0,
            infants: 0,
        }
    }
}

impl Passengers {
    /// Create with just adults
    pub fn adults(count: u8) -> Self {
        Self {
            adults: count,
            ..Default::default()
        }
    }

    /// Total passenger count
    pub fn total(&self) -> u8 {
        self.adults + self.children + self.infants
    }

    /// Validate passenger counts
    pub fn validate(&self) -> bool {
        // At least one adult
        if self.adults == 0 {
            return false;
        }
        // Max 9 passengers
        if self.total() > 9 {
            return false;
        }
        // Infants can't exceed adults
        if self.infants > self.adults {
            return false;
        }
        true
    }
}

/// A flight segment
#[derive(Debug, Clone)]
pub struct FlightSegment {
    /// Operating airline
    pub airline: AirlineCode,
    /// Flight number
    pub flight_number: String,
    /// Marketing airline (if different from operating)
    pub marketing_airline: Option<AirlineCode>,
    /// Departure airport
    pub origin: IataCode,
    /// Arrival airport
    pub destination: IataCode,
    /// Departure date
    pub departure_date: Date,
    /// Departure time (local)
    pub departure_time: Time,
    /// Arrival date
    pub arrival_date: Date,
    /// Arrival time (local)
    pub arrival_time: Time,
    /// Duration in minutes
    pub duration_minutes: u16,
    /// Aircraft type
    pub aircraft: Option<String>,
    /// Cabin class
    pub cabin: CabinClass,
    /// Booking class (fare class letter)
    pub booking_class: char,
    /// Seats remaining
    pub seats_remaining: Option<u8>,
}

impl FlightSegment {
    /// Get flight designator (e.g., "SQ123")
    pub fn designator(&self) -> String {
        format!("{}{}", self.airline.as_str(), self.flight_number)
    }

    /// Check if this is a codeshare
    pub fn is_codeshare(&self) -> bool {
        self.marketing_airline.is_some()
    }

    /// Get duration as time::Duration
    pub fn duration(&self) -> Duration {
        Duration::minutes(self.duration_minutes as i64)
    }
}

/// A complete flight itinerary (one direction)
#[derive(Debug, Clone)]
pub struct FlightLeg {
    /// Segments in this leg
    pub segments: Vec<FlightSegment>,
    /// Total duration including layovers
    pub total_duration_minutes: u16,
}

impl FlightLeg {
    /// Number of stops
    pub fn stops(&self) -> usize {
        self.segments.len().saturating_sub(1)
    }

    /// Is this a direct flight?
    pub fn is_direct(&self) -> bool {
        self.segments.len() == 1
    }

    /// Origin airport
    pub fn origin(&self) -> Option<&IataCode> {
        self.segments.first().map(|s| &s.origin)
    }

    /// Destination airport
    pub fn destination(&self) -> Option<&IataCode> {
        self.segments.last().map(|s| &s.destination)
    }

    /// Departure date
    pub fn departure_date(&self) -> Option<Date> {
        self.segments.first().map(|s| s.departure_date)
    }

    /// Departure time
    pub fn departure_time(&self) -> Option<Time> {
        self.segments.first().map(|s| s.departure_time)
    }

    /// Arrival date
    pub fn arrival_date(&self) -> Option<Date> {
        self.segments.last().map(|s| s.arrival_date)
    }

    /// Arrival time
    pub fn arrival_time(&self) -> Option<Time> {
        self.segments.last().map(|s| s.arrival_time)
    }

    /// Get all connecting airports
    pub fn connections(&self) -> Vec<&IataCode> {
        if self.segments.len() <= 1 {
            return Vec::new();
        }
        self.segments[1..].iter().map(|s| &s.origin).collect()
    }
}

/// Price breakdown
#[derive(Debug, Clone)]
pub struct PriceBreakdown {
    /// Base fare
    pub base_fare: MinorUnits,
    /// Taxes
    pub taxes: MinorUnits,
    /// Surcharges
    pub surcharges: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
}

impl PriceBreakdown {
    /// Total price
    pub fn total(&self) -> MinorUnits {
        MinorUnits::new(self.base_fare.as_i64() + self.taxes.as_i64() + self.surcharges.as_i64())
    }
}

/// A complete flight offer
#[derive(Debug, Clone)]
pub struct FlightOffer {
    /// Unique offer ID
    pub id: String,
    /// Outbound leg
    pub outbound: FlightLeg,
    /// Return leg (for round trips)
    pub inbound: Option<FlightLeg>,
    /// Price breakdown
    pub price: PriceBreakdown,
    /// Price per passenger type
    pub price_per_pax: Vec<(PassengerType, MinorUnits)>,
    /// Offer expiry time (Unix timestamp)
    pub expires_at: Option<i64>,
    /// Provider/source
    pub provider: String,
    /// Refundable
    pub refundable: bool,
    /// Changeable
    pub changeable: bool,
    /// Baggage allowance
    pub baggage: Option<BaggageAllowance>,
    /// Fare rules summary
    pub fare_rules: Option<String>,
}

impl FlightOffer {
    /// Total duration for the whole trip
    pub fn total_duration_minutes(&self) -> u16 {
        let outbound = self.outbound.total_duration_minutes;
        let inbound = self
            .inbound
            .as_ref()
            .map(|i| i.total_duration_minutes)
            .unwrap_or(0);
        outbound + inbound
    }

    /// Check if offer is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            time::OffsetDateTime::now_utc().unix_timestamp() > expires
        } else {
            false
        }
    }

    /// Is this a round trip?
    pub fn is_round_trip(&self) -> bool {
        self.inbound.is_some()
    }
}

/// Baggage allowance
#[derive(Debug, Clone)]
pub struct BaggageAllowance {
    /// Carry-on bags
    pub carry_on: u8,
    /// Carry-on weight in kg
    pub carry_on_weight_kg: Option<u8>,
    /// Checked bags
    pub checked_bags: u8,
    /// Checked bag weight in kg
    pub checked_weight_kg: Option<u8>,
}

impl Default for BaggageAllowance {
    fn default() -> Self {
        Self {
            carry_on: 1,
            carry_on_weight_kg: Some(7),
            checked_bags: 1,
            checked_weight_kg: Some(23),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cabin_class() {
        assert_eq!(CabinClass::Economy.code(), 'Y');
        assert_eq!(CabinClass::Business.name(), "Business");
    }

    #[test]
    fn test_passengers() {
        let pax = Passengers::adults(2);
        assert_eq!(pax.total(), 2);
        assert!(pax.validate());

        let invalid = Passengers {
            adults: 1,
            children: 0,
            infants: 2,
        };
        assert!(!invalid.validate());
    }

    #[test]
    fn test_flight_leg_stops() {
        let leg = FlightLeg {
            segments: vec![],
            total_duration_minutes: 0,
        };
        // 0 segments means no flight, not direct
        assert!(!leg.is_direct());
        assert_eq!(leg.stops(), 0);

        // With segments would test actual flight logic
    }

    #[test]
    fn test_price_breakdown() {
        let price = PriceBreakdown {
            base_fare: MinorUnits::new(10000),
            taxes: MinorUnits::new(2000),
            surcharges: MinorUnits::new(500),
            currency: CurrencyCode::SGD,
        };
        assert_eq!(price.total().as_i64(), 12500);
    }
}
