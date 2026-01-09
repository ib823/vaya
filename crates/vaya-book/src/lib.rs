//! vaya-book: Booking management, passenger records, and ticketing
//!
//! This crate provides comprehensive booking lifecycle management for flight reservations:
//!
//! - **Passenger management**: Full validation of passenger details, documents, contacts
//! - **Booking state machine**: Strict state transitions with audit history
//! - **Payment processing**: Card tokenization, multiple payment methods, refunds
//! - **Ticketing lifecycle**: From booking to ticket issuance
//!
//! # Security Considerations
//!
//! - Never stores raw card numbers - only tokenized references
//! - All passenger document data should be encrypted at rest
//! - PNR generation uses cryptographically secure random
//! - Optimistic locking prevents concurrent modification

mod booking;
mod error;
mod passenger;
mod payment;

pub use booking::{Booking, BookingNote, BookingStatus, StatusChange};
pub use error::{BookError, BookResult};
pub use passenger::{
    ContactDetails, CountryCode, DocumentType, FrequentFlyer, MealPreference, Passenger,
    SeatPreference, SpecialRequest, Title, TravelDocument,
};
pub use payment::{
    CardBrand, CardToken, PaymentMethod, PaymentRecord, PaymentRequest, PaymentStatus,
    RefundRecord, RefundStatus,
};

// Re-export PassengerType from vaya_search for convenience
pub use vaya_search::PassengerType;

/// Booking configuration
#[derive(Debug, Clone)]
pub struct BookingConfig {
    /// Confirmation deadline in seconds (default: 15 minutes)
    pub confirm_deadline_secs: i64,
    /// Payment deadline in seconds (default: 24 hours)
    pub payment_deadline_secs: i64,
    /// Ticketing deadline in seconds (default: 1 hour)
    pub ticketing_deadline_secs: i64,
    /// Maximum passengers per booking
    pub max_passengers: u8,
    /// Minimum advance booking days
    pub min_advance_days: u32,
    /// Maximum advance booking days
    pub max_advance_days: u32,
    /// Allow infant without adult
    pub allow_infant_only: bool,
    /// Maximum infants per adult
    pub max_infants_per_adult: u8,
}

impl Default for BookingConfig {
    fn default() -> Self {
        Self {
            confirm_deadline_secs: 900,       // 15 minutes
            payment_deadline_secs: 86400,     // 24 hours
            ticketing_deadline_secs: 3600,    // 1 hour
            max_passengers: 9,
            min_advance_days: 0,
            max_advance_days: 365,
            allow_infant_only: false,
            max_infants_per_adult: 1,
        }
    }
}

impl BookingConfig {
    /// Create a new booking configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set confirmation deadline
    pub fn with_confirm_deadline(mut self, secs: i64) -> Self {
        self.confirm_deadline_secs = secs;
        self
    }

    /// Set payment deadline
    pub fn with_payment_deadline(mut self, secs: i64) -> Self {
        self.payment_deadline_secs = secs;
        self
    }

    /// Set ticketing deadline
    pub fn with_ticketing_deadline(mut self, secs: i64) -> Self {
        self.ticketing_deadline_secs = secs;
        self
    }

    /// Set maximum passengers
    pub fn with_max_passengers(mut self, max: u8) -> Self {
        self.max_passengers = max;
        self
    }
}

/// Validate passenger composition for a booking
pub fn validate_passenger_composition(
    passengers: &[Passenger],
    config: &BookingConfig,
    departure_date: time::Date,
) -> BookResult<()> {
    if passengers.is_empty() {
        return Err(BookError::MissingField("passengers".into()));
    }

    if passengers.len() > config.max_passengers as usize {
        return Err(BookError::InvalidPassenger(format!(
            "Maximum {} passengers allowed",
            config.max_passengers
        )));
    }

    // Count passenger types
    let adults = passengers
        .iter()
        .filter(|p| p.pax_type == PassengerType::Adult)
        .count();
    let children = passengers
        .iter()
        .filter(|p| p.pax_type == PassengerType::Child)
        .count();
    let infants = passengers
        .iter()
        .filter(|p| p.pax_type == PassengerType::Infant)
        .count();

    // Infants require adults
    if infants > 0 && adults == 0 && !config.allow_infant_only {
        return Err(BookError::InvalidPassenger(
            "Infants must travel with an adult".into(),
        ));
    }

    // Check infant-to-adult ratio
    if infants > adults * config.max_infants_per_adult as usize {
        return Err(BookError::InvalidPassenger(format!(
            "Maximum {} infant(s) per adult",
            config.max_infants_per_adult
        )));
    }

    // Children under certain age may require adult (airline-specific, but general rule)
    if children > 0 && adults == 0 {
        // Allow unaccompanied minors but log it
        tracing::warn!(
            child_count = children,
            "Booking contains unaccompanied children"
        );
    }

    // Validate each passenger
    for (i, passenger) in passengers.iter().enumerate() {
        passenger.validate(departure_date).map_err(|e| {
            BookError::InvalidPassenger(format!("Passenger {}: {}", i + 1, e))
        })?;
    }

    Ok(())
}

/// Calculate pricing breakdown for passengers
pub fn calculate_passenger_pricing(
    passengers: &[Passenger],
    adult_price: vaya_common::MinorUnits,
    child_discount_pct: u8,
    infant_price: vaya_common::MinorUnits,
) -> vaya_common::MinorUnits {
    let mut total = 0i64;

    for passenger in passengers {
        let price = match passenger.pax_type {
            PassengerType::Adult => adult_price.as_i64(),
            PassengerType::Child => {
                let discount = (adult_price.as_i64() * child_discount_pct as i64) / 100;
                adult_price.as_i64() - discount
            }
            PassengerType::Infant => infant_price.as_i64(),
        };
        total += price;
    }

    vaya_common::MinorUnits::new(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Date;
    use vaya_common::Gender;

    fn make_adult(name: &str, dob: Date) -> Passenger {
        Passenger::adult(name, "TEST", dob, Gender::Male)
    }

    fn make_child(name: &str) -> Passenger {
        // 8 years old
        let dob = Date::from_calendar_date(2017, time::Month::January, 15).unwrap();
        let mut p = Passenger::adult(name, "TEST", dob, Gender::Male);
        p.pax_type = PassengerType::Child;
        p
    }

    fn make_infant(name: &str) -> Passenger {
        // 1 year old
        let dob = Date::from_calendar_date(2024, time::Month::January, 15).unwrap();
        let mut p = Passenger::adult(name, "TEST", dob, Gender::Male);
        p.pax_type = PassengerType::Infant;
        p
    }

    #[test]
    fn test_validate_composition_empty() {
        let config = BookingConfig::default();
        let dep = Date::from_calendar_date(2025, time::Month::June, 1).unwrap();
        let result = validate_passenger_composition(&[], &config, dep);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_composition_too_many() {
        let config = BookingConfig::default().with_max_passengers(2);
        let dob = Date::from_calendar_date(1990, time::Month::January, 15).unwrap();
        let dep = Date::from_calendar_date(2025, time::Month::June, 1).unwrap();
        let passengers = vec![
            make_adult("A", dob),
            make_adult("B", dob),
            make_adult("C", dob),
        ];
        let result = validate_passenger_composition(&passengers, &config, dep);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_composition_infant_without_adult() {
        let config = BookingConfig::default();
        let dep = Date::from_calendar_date(2025, time::Month::June, 1).unwrap();
        let passengers = vec![make_infant("BABY")];
        let result = validate_passenger_composition(&passengers, &config, dep);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_composition_too_many_infants() {
        let config = BookingConfig::default();
        let dob = Date::from_calendar_date(1990, time::Month::January, 15).unwrap();
        let dep = Date::from_calendar_date(2025, time::Month::June, 1).unwrap();
        let passengers = vec![
            make_adult("JOHN", dob),
            make_infant("ANNA"),
            make_infant("EMMA"),
        ];
        let result = validate_passenger_composition(&passengers, &config, dep);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_composition_valid_family() {
        let config = BookingConfig::default();
        let dob = Date::from_calendar_date(1990, time::Month::January, 15).unwrap();
        let dep = Date::from_calendar_date(2025, time::Month::June, 1).unwrap();
        let passengers = vec![
            make_adult("JOHN", dob),
            make_adult("JANE", dob),
            make_child("TOMMY"),
            make_infant("SARAH"),
        ];
        let result = validate_passenger_composition(&passengers, &config, dep);
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_pricing() {
        let dob = Date::from_calendar_date(1990, time::Month::January, 15).unwrap();
        let passengers = vec![
            make_adult("JOHN", dob),
            make_adult("JANE", dob),
            make_child("TOMMY"),
            make_infant("SARAH"),
        ];

        let adult_price = vaya_common::MinorUnits::new(10000); // $100
        let child_discount = 25; // 25% off
        let infant_price = vaya_common::MinorUnits::new(1000); // $10

        let total = calculate_passenger_pricing(&passengers, adult_price, child_discount, infant_price);

        // 2 adults @ $100 = $200
        // 1 child @ $75 (25% off) = $75
        // 1 infant @ $10 = $10
        // Total = $285 = 28500 cents
        assert_eq!(total.as_i64(), 28500);
    }

    #[test]
    fn test_booking_config_builder() {
        let config = BookingConfig::new()
            .with_confirm_deadline(600)
            .with_payment_deadline(43200)
            .with_max_passengers(4);

        assert_eq!(config.confirm_deadline_secs, 600);
        assert_eq!(config.payment_deadline_secs, 43200);
        assert_eq!(config.max_passengers, 4);
    }

    #[test]
    fn test_default_config() {
        let config = BookingConfig::default();
        assert_eq!(config.confirm_deadline_secs, 900);
        assert_eq!(config.payment_deadline_secs, 86400);
        assert_eq!(config.ticketing_deadline_secs, 3600);
        assert_eq!(config.max_passengers, 9);
        assert!(!config.allow_infant_only);
        assert_eq!(config.max_infants_per_adult, 1);
    }
}
