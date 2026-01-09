//! GDS Provider trait for abstraction

use async_trait::async_trait;

use crate::error::GdsResult;
use crate::types::{
    BookingConfirmation, ContactDetails, FlightOffer, FlightSearchRequest, PassengerDetails,
};

/// GDS Provider trait - implement for each GDS system
///
/// This trait abstracts the GDS operations, allowing:
/// - Multiple GDS providers (Amadeus, Travelport, Sabre)
/// - Easy testing with mock implementations
/// - Fallback between providers
#[async_trait]
pub trait GdsProvider: Send + Sync {
    /// Search for available flights
    ///
    /// Returns a list of flight offers matching the search criteria.
    /// Results are typically cached for 5 minutes.
    async fn search_flights(&self, request: &FlightSearchRequest) -> GdsResult<Vec<FlightOffer>>;

    /// Get current pricing for an offer
    ///
    /// Verifies the offer is still available and returns updated pricing.
    /// Should be called before booking to ensure price hasn't changed.
    async fn price_offer(&self, offer_id: &str) -> GdsResult<FlightOffer>;

    /// Create a booking (PNR)
    ///
    /// Creates a booking with the GDS. The booking will be in "Confirmed"
    /// status and may have a ticketing deadline.
    async fn create_booking(
        &self,
        offer_id: &str,
        passengers: &[PassengerDetails],
        contact: &ContactDetails,
    ) -> GdsResult<BookingConfirmation>;

    /// Issue ticket for a booking
    ///
    /// Issues the ticket after payment is confirmed.
    async fn issue_ticket(&self, pnr: &str) -> GdsResult<BookingConfirmation>;

    /// Cancel a booking
    ///
    /// Cancels the booking. May incur fees depending on fare rules.
    async fn cancel_booking(&self, pnr: &str) -> GdsResult<()>;

    /// Get booking status
    ///
    /// Retrieves the current status of a booking.
    async fn get_booking(&self, pnr: &str) -> GdsResult<BookingConfirmation>;

    /// Get available airports
    ///
    /// Returns airports matching the search query.
    async fn search_airports(&self, query: &str) -> GdsResult<Vec<AirportInfo>>;

    /// Check provider health
    ///
    /// Returns true if the provider is operational.
    async fn health_check(&self) -> bool;

    /// Provider name
    fn provider_name(&self) -> &'static str;
}

/// Airport information
#[derive(Debug, Clone)]
pub struct AirportInfo {
    /// IATA code
    pub iata_code: String,
    /// Airport name
    pub name: String,
    /// City name
    pub city: String,
    /// Country name
    pub country: String,
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: String,
}

/// Mock GDS provider for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use vaya_common::{AirlineCode, CurrencyCode, Date, IataCode, MinorUnits, Price, Timestamp};

    use crate::{
        BaggageAllowance, BookingStatus, FareRules, FlightPoint, FlightSegment, Itinerary,
        PriceBreakdown,
    };

    /// Mock GDS provider for testing
    pub struct MockGdsProvider {
        /// Should search return empty results
        pub return_empty: AtomicBool,
        /// Should operations fail
        pub should_fail: AtomicBool,
    }

    impl MockGdsProvider {
        /// Create new mock provider
        #[must_use]
        pub fn new() -> Self {
            Self {
                return_empty: AtomicBool::new(false),
                should_fail: AtomicBool::new(false),
            }
        }

        /// Set to return empty results
        pub fn set_empty(&self, empty: bool) {
            self.return_empty.store(empty, Ordering::SeqCst);
        }

        /// Set to fail operations
        pub fn set_fail(&self, fail: bool) {
            self.should_fail.store(fail, Ordering::SeqCst);
        }
    }

    impl Default for MockGdsProvider {
        fn default() -> Self {
            Self::new()
        }
    }

    #[async_trait]
    impl GdsProvider for MockGdsProvider {
        async fn search_flights(
            &self,
            request: &FlightSearchRequest,
        ) -> GdsResult<Vec<FlightOffer>> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(crate::error::GdsError::ServiceUnavailable(
                    "Mock failure".to_string(),
                ));
            }

            if self.return_empty.load(Ordering::SeqCst) {
                return Ok(Vec::new());
            }

            // Generate mock offers
            let offers = vec![
                create_mock_offer("OFFER1", request, 50000),
                create_mock_offer("OFFER2", request, 55000),
                create_mock_offer("OFFER3", request, 48000),
            ];

            Ok(offers)
        }

        async fn price_offer(&self, offer_id: &str) -> GdsResult<FlightOffer> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(crate::error::GdsError::ServiceUnavailable(
                    "Mock failure".to_string(),
                ));
            }

            // Return mock offer with updated price
            let request = FlightSearchRequest::one_way(IataCode::KUL, IataCode::NRT, Date::today());
            Ok(create_mock_offer(offer_id, &request, 51000))
        }

        async fn create_booking(
            &self,
            offer_id: &str,
            passengers: &[PassengerDetails],
            _contact: &ContactDetails,
        ) -> GdsResult<BookingConfirmation> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(crate::error::GdsError::BookingFailed {
                    code: "MOCK_ERROR".to_string(),
                    message: "Mock booking failure".to_string(),
                });
            }

            Ok(BookingConfirmation {
                pnr: "ABC123".to_string(),
                booking_reference: format!("VAY{}", &offer_id[..6]),
                status: BookingStatus::Confirmed,
                created_at: Timestamp::now(),
                ticketing_deadline: Some(Timestamp::now().add_hours(24)),
                passengers: passengers.iter().map(|p| p.full_name()).collect(),
                offer_id: offer_id.to_string(),
            })
        }

        async fn issue_ticket(&self, pnr: &str) -> GdsResult<BookingConfirmation> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(crate::error::GdsError::TicketingFailed(
                    "Mock ticketing failure".to_string(),
                ));
            }

            Ok(BookingConfirmation {
                pnr: pnr.to_string(),
                booking_reference: format!("VAY{pnr}"),
                status: BookingStatus::Ticketed,
                created_at: Timestamp::now(),
                ticketing_deadline: None,
                passengers: vec!["Test Passenger".to_string()],
                offer_id: "OFFER1".to_string(),
            })
        }

        async fn cancel_booking(&self, _pnr: &str) -> GdsResult<()> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(crate::error::GdsError::CancellationFailed(
                    "Mock cancellation failure".to_string(),
                ));
            }
            Ok(())
        }

        async fn get_booking(&self, pnr: &str) -> GdsResult<BookingConfirmation> {
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(crate::error::GdsError::NotFound {
                    resource: "booking".to_string(),
                    id: pnr.to_string(),
                });
            }

            Ok(BookingConfirmation {
                pnr: pnr.to_string(),
                booking_reference: format!("VAY{pnr}"),
                status: BookingStatus::Confirmed,
                created_at: Timestamp::now(),
                ticketing_deadline: Some(Timestamp::now().add_hours(24)),
                passengers: vec!["Test Passenger".to_string()],
                offer_id: "OFFER1".to_string(),
            })
        }

        async fn search_airports(&self, query: &str) -> GdsResult<Vec<AirportInfo>> {
            let airports = vec![
                AirportInfo {
                    iata_code: "KUL".to_string(),
                    name: "Kuala Lumpur International Airport".to_string(),
                    city: "Kuala Lumpur".to_string(),
                    country: "Malaysia".to_string(),
                    country_code: "MY".to_string(),
                },
                AirportInfo {
                    iata_code: "SIN".to_string(),
                    name: "Singapore Changi Airport".to_string(),
                    city: "Singapore".to_string(),
                    country: "Singapore".to_string(),
                    country_code: "SG".to_string(),
                },
            ];

            let query_upper = query.to_uppercase();
            Ok(airports
                .into_iter()
                .filter(|a| {
                    a.iata_code.contains(&query_upper)
                        || a.city.to_uppercase().contains(&query_upper)
                })
                .collect())
        }

        async fn health_check(&self) -> bool {
            !self.should_fail.load(Ordering::SeqCst)
        }

        fn provider_name(&self) -> &'static str {
            "MockGDS"
        }
    }

    fn create_mock_offer(id: &str, request: &FlightSearchRequest, price_cents: i64) -> FlightOffer {
        let departure_time = request.departure_date.to_timestamp();
        let arrival_time = departure_time.add_hours(7);

        let segment = FlightSegment {
            departure: FlightPoint::new(request.origin, departure_time),
            arrival: FlightPoint::new(request.destination, arrival_time),
            airline: AirlineCode::MH,
            flight_number: "88".to_string(),
            duration_minutes: 420,
            aircraft: Some("A350".to_string()),
            cabin_class: request.cabin_class,
            booking_class: Some("Y".to_string()),
            stops: 0,
        };

        let base_price = Price::new(MinorUnits::new(price_cents), CurrencyCode::MYR);
        let taxes = Price::new(MinorUnits::new(price_cents / 10), CurrencyCode::MYR);

        FlightOffer {
            id: id.to_string(),
            outbound: Itinerary {
                segments: vec![segment],
                total_duration_minutes: 420,
            },
            return_itinerary: None,
            price: PriceBreakdown::simple(base_price, taxes),
            validating_airline: AirlineCode::MH,
            available_seats: Some(9),
            created_at: Timestamp::now(),
            expires_at: Some(Timestamp::now().add_mins(30)),
            instant_ticketing: true,
            fare_rules: Some(FareRules {
                refundable: false,
                changeable: true,
                change_fee: Some(Price::myr(15000)),
                cancellation_fee: None,
                baggage: Some(BaggageAllowance {
                    checked_bags: 1,
                    weight_kg: Some(23),
                    carry_on: true,
                }),
            }),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_mock_search() {
            let provider = MockGdsProvider::new();
            let request = FlightSearchRequest::one_way(IataCode::KUL, IataCode::NRT, Date::today());

            let offers = provider.search_flights(&request).await.unwrap();
            assert_eq!(offers.len(), 3);
        }

        #[tokio::test]
        async fn test_mock_empty() {
            let provider = MockGdsProvider::new();
            provider.set_empty(true);

            let request = FlightSearchRequest::one_way(IataCode::KUL, IataCode::NRT, Date::today());
            let offers = provider.search_flights(&request).await.unwrap();
            assert!(offers.is_empty());
        }

        #[tokio::test]
        async fn test_mock_failure() {
            let provider = MockGdsProvider::new();
            provider.set_fail(true);

            let request = FlightSearchRequest::one_way(IataCode::KUL, IataCode::NRT, Date::today());
            assert!(provider.search_flights(&request).await.is_err());
        }

        #[tokio::test]
        async fn test_mock_booking_flow() {
            let provider = MockGdsProvider::new();

            // Search
            let request = FlightSearchRequest::one_way(IataCode::KUL, IataCode::NRT, Date::today());
            let offers = provider.search_flights(&request).await.unwrap();
            let offer = &offers[0];

            // Price
            let priced = provider.price_offer(&offer.id).await.unwrap();
            assert_eq!(priced.id, offer.id);

            // Book
            let passenger = PassengerDetails::adult("John", "Doe", Date::new(1990, 1, 1));
            let contact = ContactDetails::new("john@example.com", "+60123456789");
            let booking = provider
                .create_booking(&offer.id, &[passenger], &contact)
                .await
                .unwrap();
            assert_eq!(booking.status, BookingStatus::Confirmed);

            // Issue ticket
            let ticketed = provider.issue_ticket(&booking.pnr).await.unwrap();
            assert_eq!(ticketed.status, BookingStatus::Ticketed);
        }
    }
}
