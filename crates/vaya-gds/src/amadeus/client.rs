//! Amadeus GDS Client implementation

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use vaya_common::{AirlineCode, CurrencyCode, IataCode, MinorUnits, Price, Timestamp};

use crate::cache::GdsCache;
use crate::error::{GdsError, GdsResult};
use crate::traits::{AirportInfo, GdsProvider};
use crate::types::{
    BaggageAllowance, BookingConfirmation, BookingStatus, CabinClass, ContactDetails, FareRules,
    FlightOffer, FlightPoint, FlightSearchRequest, FlightSegment, Itinerary, PassengerDetails,
    PriceBreakdown,
};
use crate::GdsConfig;

use super::auth::TokenManager;
use super::response::{
    AirportSearchResponse, AmadeusError, AmadeusFlightOffer, AmadeusItinerary, AmadeusSegment,
    ContactRequest, Dictionaries, FlightOffersResponse, FlightOrderRequest, FlightOrderResponse,
    Phone, TravelerContact, TravelerDocument, TravelerName, TravelerPricing, TravelerRequest,
};

/// Amadeus GDS client
pub struct AmadeusClient {
    /// HTTP client
    http_client: reqwest::Client,
    /// Token manager
    token_manager: Arc<TokenManager>,
    /// Response cache
    cache: GdsCache,
    /// Base URL
    base_url: String,
    /// Max retries
    max_retries: u32,
}

impl AmadeusClient {
    /// Create new Amadeus client
    pub fn new(config: &GdsConfig) -> GdsResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .build()
            .map_err(|e| GdsError::Configuration(format!("Failed to create HTTP client: {e}")))?;

        let token_manager = Arc::new(TokenManager::new(config, http_client.clone()));

        let cache = GdsCache::new()
            .with_search_ttl(Duration::from_secs(config.search_cache_ttl_secs))
            .with_pricing_ttl(Duration::from_secs(config.pricing_cache_ttl_secs));

        Ok(Self {
            http_client,
            token_manager,
            cache,
            base_url: config.amadeus_base_url.clone(),
            max_retries: config.max_retries,
        })
    }

    /// Build cache key for search request
    fn build_cache_key(request: &FlightSearchRequest) -> String {
        format!(
            "{}-{}-{}-{:?}-{}-{}-{:?}",
            request.origin,
            request.destination,
            request.departure_date,
            request.return_date,
            request.adults,
            request.children + request.infants,
            request.cabin_class,
        )
    }

    /// Make authenticated GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> GdsResult<T> {
        self.request_with_retry(reqwest::Method::GET, url, None::<()>)
            .await
    }

    /// Make authenticated POST request
    async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        url: &str,
        body: &B,
    ) -> GdsResult<T> {
        self.request_with_retry(reqwest::Method::POST, url, Some(body))
            .await
    }

    /// Execute request with retry logic
    async fn request_with_retry<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<B>,
    ) -> GdsResult<T> {
        let mut last_error = GdsError::ServiceUnavailable("No attempts made".to_string());

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
                debug!("Retry attempt {} after {:?}", attempt, delay);
            }

            match self.execute_request(method.clone(), url, &body).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if e.is_retryable() && attempt < self.max_retries {
                        warn!("Retryable error on attempt {}: {:?}", attempt + 1, e);
                        last_error = e;
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Err(last_error)
    }

    /// Execute a single request
    async fn execute_request<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: &Option<B>,
    ) -> GdsResult<T> {
        let token = self.token_manager.get_token().await?;

        let mut request = self
            .http_client
            .request(method, url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/json");

        if let Some(ref b) = body {
            request = request.json(b);
        }

        let response = request.send().await.map_err(GdsError::from)?;
        let status = response.status();

        if status.is_success() {
            let result: T = response
                .json()
                .await
                .map_err(|e| GdsError::InvalidResponse(format!("Failed to parse response: {e}")))?;
            return Ok(result);
        }

        // Handle errors
        let body_text = response.text().await.unwrap_or_default();

        if status.as_u16() == 401 {
            self.token_manager.invalidate();
            return Err(GdsError::AuthenticationFailed(
                "Token expired or invalid".to_string(),
            ));
        }

        if status.as_u16() == 429 {
            return Err(GdsError::RateLimited {
                retry_after_secs: 60,
            });
        }

        if status.as_u16() == 404 {
            return Err(GdsError::NotFound {
                resource: "resource".to_string(),
                id: url.to_string(),
            });
        }

        // Try to parse Amadeus error
        if let Ok(amadeus_error) = serde_json::from_str::<AmadeusError>(&body_text) {
            if let Some(error) = amadeus_error.errors.first() {
                return Err(GdsError::ServiceUnavailable(format!(
                    "{}: {}",
                    error.title.as_deref().unwrap_or("Error"),
                    error.detail.as_deref().unwrap_or("Unknown error")
                )));
            }
        }

        Err(GdsError::ServiceUnavailable(format!(
            "HTTP {status}: {body_text}"
        )))
    }

    /// Convert Amadeus flight offer to internal type
    fn convert_offer(
        &self,
        amadeus_offer: &AmadeusFlightOffer,
        _dictionaries: &Option<Dictionaries>,
    ) -> GdsResult<FlightOffer> {
        let outbound = self.convert_itinerary(&amadeus_offer.itineraries[0])?;

        let return_itinerary = if amadeus_offer.itineraries.len() > 1 {
            Some(self.convert_itinerary(&amadeus_offer.itineraries[1])?)
        } else {
            None
        };

        // Parse price
        let total_cents: i64 = amadeus_offer
            .price
            .total
            .parse::<f64>()
            .map(|v| (v * 100.0) as i64)
            .unwrap_or(0);

        let base_cents: i64 = amadeus_offer
            .price
            .base
            .as_ref()
            .and_then(|b| b.parse::<f64>().ok())
            .map_or(total_cents, |v| (v * 100.0) as i64);

        let currency = CurrencyCode::new(&amadeus_offer.price.currency);

        let base_price = Price::new(MinorUnits::new(base_cents), currency);
        let taxes = Price::new(MinorUnits::new(total_cents - base_cents), currency);

        // Get validating airline
        let validating_airline = amadeus_offer
            .validating_airline_codes
            .as_ref()
            .and_then(|codes| codes.first())
            .map_or(AirlineCode::MH, |code| AirlineCode::new(code));

        // Extract fare rules from traveler pricing
        let fare_rules = self.extract_fare_rules(&amadeus_offer.traveler_pricings);

        Ok(FlightOffer {
            id: amadeus_offer.id.clone(),
            outbound,
            return_itinerary,
            price: PriceBreakdown::simple(base_price, taxes),
            validating_airline,
            available_seats: amadeus_offer.number_of_bookable_seats,
            created_at: Timestamp::now(),
            expires_at: Some(Timestamp::now().add_mins(30)),
            instant_ticketing: amadeus_offer.instant_ticketing_required.unwrap_or(false),
            fare_rules,
        })
    }

    /// Convert Amadeus itinerary
    fn convert_itinerary(&self, itinerary: &AmadeusItinerary) -> GdsResult<Itinerary> {
        let segments: Vec<FlightSegment> = itinerary
            .segments
            .iter()
            .map(|s| self.convert_segment(s))
            .collect::<GdsResult<Vec<_>>>()?;

        let total_duration = self.parse_duration(&itinerary.duration);

        Ok(Itinerary {
            segments,
            total_duration_minutes: total_duration,
        })
    }

    /// Convert Amadeus segment
    fn convert_segment(&self, segment: &AmadeusSegment) -> GdsResult<FlightSegment> {
        let departure_time = self.parse_iso_datetime(&segment.departure.at);
        let arrival_time = self.parse_iso_datetime(&segment.arrival.at);

        let mut departure =
            FlightPoint::new(IataCode::new(&segment.departure.iata_code), departure_time);
        if let Some(ref term) = segment.departure.terminal {
            departure = departure.with_terminal(term.clone());
        }

        let mut arrival = FlightPoint::new(IataCode::new(&segment.arrival.iata_code), arrival_time);
        if let Some(ref term) = segment.arrival.terminal {
            arrival = arrival.with_terminal(term.clone());
        }

        let duration = self.parse_duration(&segment.duration);

        let airline = AirlineCode::new(&segment.carrier_code);

        Ok(FlightSegment {
            departure,
            arrival,
            airline,
            flight_number: segment.number.clone(),
            duration_minutes: duration,
            aircraft: segment.aircraft.as_ref().map(|a| a.code.clone()),
            cabin_class: CabinClass::Economy,
            booking_class: None,
            stops: segment.number_of_stops.unwrap_or(0) as u8,
        })
    }

    /// Parse ISO 8601 datetime string to Timestamp
    fn parse_iso_datetime(&self, datetime: &str) -> Timestamp {
        // Format: 2025-01-15T10:30:00
        // Simple parsing - extract year, month, day, hour, minute, second
        let parts: Vec<&str> = datetime.split('T').collect();
        if parts.len() != 2 {
            return Timestamp::now();
        }

        let date_parts: Vec<&str> = parts[0].split('-').collect();
        let time_parts: Vec<&str> = parts[1].split(':').collect();

        if date_parts.len() < 3 || time_parts.len() < 2 {
            return Timestamp::now();
        }

        let year: i64 = date_parts[0].parse().unwrap_or(2025);
        let month: i64 = date_parts[1].parse().unwrap_or(1);
        let day: i64 = date_parts[2].parse().unwrap_or(1);
        let hour: i64 = time_parts[0].parse().unwrap_or(0);
        let minute: i64 = time_parts[1].parse().unwrap_or(0);
        let second: i64 = time_parts
            .get(2)
            .and_then(|s| s.split('+').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Calculate unix timestamp (simplified - not accounting for leap years properly)
        let days_since_epoch =
            (year - 1970) * 365 + (year - 1969) / 4 - (year - 1901) / 100 + (year - 1601) / 400;
        let month_days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        let day_of_year = month_days.get((month - 1) as usize).copied().unwrap_or(0) + day - 1;
        let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let leap_adjustment = i64::from(is_leap && month > 2);

        let total_days = days_since_epoch + day_of_year + leap_adjustment;
        let total_seconds = total_days * 86400 + hour * 3600 + minute * 60 + second;

        Timestamp::from_unix(total_seconds)
    }

    /// Parse ISO 8601 duration (e.g., PT7H30M)
    fn parse_duration(&self, duration: &Option<String>) -> u32 {
        let Some(d) = duration else {
            return 0;
        };

        let mut total_minutes = 0u32;
        let mut current_num = String::new();

        for c in d.chars() {
            if c.is_ascii_digit() {
                current_num.push(c);
            } else if c == 'H' {
                if let Ok(hours) = current_num.parse::<u32>() {
                    total_minutes += hours * 60;
                }
                current_num.clear();
            } else if c == 'M' {
                if let Ok(mins) = current_num.parse::<u32>() {
                    total_minutes += mins;
                }
                current_num.clear();
            }
        }

        total_minutes
    }

    /// Extract fare rules from traveler pricing
    fn extract_fare_rules(
        &self,
        traveler_pricings: &Option<Vec<TravelerPricing>>,
    ) -> Option<FareRules> {
        let pricings = traveler_pricings.as_ref()?;
        let pricing = pricings.first()?;
        let details = pricing.fare_details_by_segment.as_ref()?;
        let detail = details.first()?;

        let baggage = detail
            .included_checked_bags
            .as_ref()
            .map(|bags| BaggageAllowance {
                checked_bags: bags.quantity.unwrap_or(1) as u8,
                weight_kg: bags.weight,
                carry_on: true,
            });

        Some(FareRules {
            refundable: false, // Would need fare rules API
            changeable: true,
            change_fee: None,
            cancellation_fee: None,
            baggage,
        })
    }

    /// Format date as ISO string (YYYY-MM-DD)
    fn format_date(&self, date: &vaya_common::Date) -> String {
        format!("{date}")
    }

    /// Build search request body
    fn build_search_request(&self, request: &FlightSearchRequest) -> serde_json::Value {
        let mut origin_destinations = vec![serde_json::json!({
            "id": "1",
            "originLocationCode": request.origin.as_str(),
            "destinationLocationCode": request.destination.as_str(),
            "departureDateTimeRange": {
                "date": self.format_date(&request.departure_date)
            }
        })];

        if let Some(return_date) = &request.return_date {
            origin_destinations.push(serde_json::json!({
                "id": "2",
                "originLocationCode": request.destination.as_str(),
                "destinationLocationCode": request.origin.as_str(),
                "departureDateTimeRange": {
                    "date": self.format_date(return_date)
                }
            }));
        }

        let mut travelers = Vec::new();
        let mut traveler_id = 1;

        for _ in 0..request.adults {
            travelers.push(serde_json::json!({
                "id": traveler_id.to_string(),
                "travelerType": "ADULT"
            }));
            traveler_id += 1;
        }

        for _ in 0..request.children {
            travelers.push(serde_json::json!({
                "id": traveler_id.to_string(),
                "travelerType": "CHILD"
            }));
            traveler_id += 1;
        }

        for _ in 0..request.infants {
            travelers.push(serde_json::json!({
                "id": traveler_id.to_string(),
                "travelerType": "SEATED_INFANT"
            }));
            traveler_id += 1;
        }

        let cabin_code = match request.cabin_class {
            CabinClass::Economy => "ECONOMY",
            CabinClass::PremiumEconomy => "PREMIUM_ECONOMY",
            CabinClass::Business => "BUSINESS",
            CabinClass::First => "FIRST",
        };

        serde_json::json!({
            "currencyCode": "MYR",
            "originDestinations": origin_destinations,
            "travelers": travelers,
            "sources": ["GDS"],
            "searchCriteria": {
                "maxFlightOffers": request.max_results,
                "flightFilters": {
                    "cabinRestrictions": [{
                        "cabin": cabin_code,
                        "coverage": "MOST_SEGMENTS",
                        "originDestinationIds": ["1"]
                    }]
                }
            }
        })
    }
}

#[async_trait]
impl GdsProvider for AmadeusClient {
    async fn search_flights(&self, request: &FlightSearchRequest) -> GdsResult<Vec<FlightOffer>> {
        let cache_key = Self::build_cache_key(request);

        // Check cache
        if let Some(cached) = self.cache.get_search(&cache_key) {
            debug!("Cache hit for search: {}", cache_key);
            return Ok(cached);
        }

        debug!("Cache miss for search: {}", cache_key);

        let url = format!("{}/v2/shopping/flight-offers", self.base_url);
        let body = self.build_search_request(request);

        let response: FlightOffersResponse = self.post(&url, &body).await?;

        let offers: Vec<FlightOffer> = response
            .data
            .iter()
            .filter_map(|o| self.convert_offer(o, &response.dictionaries).ok())
            .collect();

        info!(
            "Found {} flight offers for {} -> {}",
            offers.len(),
            request.origin,
            request.destination
        );

        // Cache results
        self.cache.put_search(&cache_key, offers.clone());

        Ok(offers)
    }

    async fn price_offer(&self, offer_id: &str) -> GdsResult<FlightOffer> {
        // Check pricing cache
        if let Some(cached) = self.cache.get_pricing(offer_id) {
            debug!("Cache hit for pricing: {}", offer_id);
            return Ok(cached);
        }

        // In a real implementation, we would call the flight-offers-pricing endpoint
        // For now, return an error indicating the offer needs to be re-searched
        Err(GdsError::NotFound {
            resource: "offer".to_string(),
            id: offer_id.to_string(),
        })
    }

    async fn create_booking(
        &self,
        offer_id: &str,
        passengers: &[PassengerDetails],
        contact: &ContactDetails,
    ) -> GdsResult<BookingConfirmation> {
        // Build booking request
        let travelers: Vec<TravelerRequest> = passengers
            .iter()
            .enumerate()
            .map(|(i, p)| TravelerRequest {
                id: (i + 1).to_string(),
                date_of_birth: format!("{}", p.date_of_birth),
                gender: p.gender.amadeus_code().to_string(),
                name: TravelerName {
                    first_name: p.first_name.clone(),
                    last_name: p.last_name.clone(),
                },
                documents: p.passport_number.as_ref().map(|num| {
                    vec![TravelerDocument {
                        document_type: "PASSPORT".to_string(),
                        birth_place: None,
                        issuance_location: None,
                        issuance_date: None,
                        number: num.clone(),
                        expiry_date: p
                            .passport_expiry
                            .as_ref()
                            .map(|d| format!("{d}"))
                            .unwrap_or_default(),
                        issuance_country: p.nationality.clone().unwrap_or_else(|| "MY".to_string()),
                        validity_country: None,
                        nationality: p.nationality.clone().unwrap_or_else(|| "MY".to_string()),
                        holder: true,
                    }]
                }),
                contact: Some(TravelerContact {
                    email_address: Some(contact.email.clone()),
                    phones: Some(vec![Phone {
                        device_type: "MOBILE".to_string(),
                        country_calling_code: "60".to_string(),
                        number: contact.phone.clone(),
                    }]),
                }),
            })
            .collect();

        let contact_request = ContactRequest {
            address_eename: None,
            purpose: "STANDARD".to_string(),
            phones: vec![Phone {
                device_type: "MOBILE".to_string(),
                country_calling_code: "60".to_string(),
                number: contact.phone.clone(),
            }],
            email_address: contact.email.clone(),
        };

        // We would need to retrieve the original flight offer here
        // For now, create a placeholder booking request
        let booking_request = FlightOrderRequest {
            request_type: "flight-order".to_string(),
            flight_offers: vec![serde_json::json!({"id": offer_id})],
            travelers,
            remarks: None,
            contacts: vec![contact_request],
        };

        let url = format!("{}/v1/booking/flight-orders", self.base_url);
        let response: FlightOrderResponse = self.post(&url, &booking_request).await?;

        let pnr = response
            .data
            .associated_records
            .as_ref()
            .and_then(|records| records.first())
            .map_or_else(|| response.data.id.clone(), |r| r.reference.clone());

        let ticketing_deadline = response
            .data
            .ticketing_agreement
            .as_ref()
            .and_then(|t| t.date_time.as_ref())
            .map(|dt| self.parse_iso_datetime(dt));

        Ok(BookingConfirmation {
            pnr,
            booking_reference: response.data.id,
            status: BookingStatus::Confirmed,
            created_at: Timestamp::now(),
            ticketing_deadline,
            passengers: passengers
                .iter()
                .map(super::super::types::PassengerDetails::full_name)
                .collect(),
            offer_id: offer_id.to_string(),
        })
    }

    async fn issue_ticket(&self, _pnr: &str) -> GdsResult<BookingConfirmation> {
        // Ticketing would require payment integration
        // Return current booking with updated status
        Err(GdsError::TicketingFailed(
            "Ticketing requires payment confirmation".to_string(),
        ))
    }

    async fn cancel_booking(&self, pnr: &str) -> GdsResult<()> {
        let url = format!("{}/v1/booking/flight-orders/{}", self.base_url, pnr);

        // DELETE request
        let token = self.token_manager.get_token().await?;
        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(GdsError::from)?;

        if response.status().is_success() {
            info!("Cancelled booking: {}", pnr);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(GdsError::CancellationFailed(format!(
                "HTTP {status}: {body}"
            )))
        }
    }

    async fn get_booking(&self, pnr: &str) -> GdsResult<BookingConfirmation> {
        let url = format!("{}/v1/booking/flight-orders/{}", self.base_url, pnr);
        let response: FlightOrderResponse = self.get(&url).await?;

        let ticketing_deadline = response
            .data
            .ticketing_agreement
            .as_ref()
            .and_then(|t| t.date_time.as_ref())
            .map(|dt| self.parse_iso_datetime(dt));

        let passengers = response
            .data
            .travelers
            .as_ref()
            .map(|travelers| {
                travelers
                    .iter()
                    .filter_map(|t| {
                        t.get("name").and_then(|n| {
                            let first = n.get("firstName")?.as_str()?;
                            let last = n.get("lastName")?.as_str()?;
                            Some(format!("{first} {last}"))
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(BookingConfirmation {
            pnr: pnr.to_string(),
            booking_reference: response.data.id,
            status: BookingStatus::Confirmed,
            created_at: Timestamp::now(),
            ticketing_deadline,
            passengers,
            offer_id: response
                .data
                .flight_offers
                .first()
                .and_then(|o| o.get("id")?.as_str())
                .map(String::from)
                .unwrap_or_default(),
        })
    }

    async fn search_airports(&self, query: &str) -> GdsResult<Vec<AirportInfo>> {
        let url = format!(
            "{}/v1/reference-data/locations?subType=AIRPORT&keyword={}&page[limit]=10",
            self.base_url, query
        );

        let response: AirportSearchResponse = self.get(&url).await?;

        let airports = response
            .data
            .into_iter()
            .map(|a| AirportInfo {
                iata_code: a.iata_code,
                name: a.name,
                city: a
                    .address
                    .as_ref()
                    .and_then(|addr| addr.city_name.clone())
                    .unwrap_or_default(),
                country: a
                    .address
                    .as_ref()
                    .and_then(|addr| addr.country_name.clone())
                    .unwrap_or_default(),
                country_code: a
                    .address
                    .as_ref()
                    .and_then(|addr| addr.country_code.clone())
                    .unwrap_or_default(),
            })
            .collect();

        Ok(airports)
    }

    async fn health_check(&self) -> bool {
        match self.token_manager.get_token().await {
            Ok(_) => true,
            Err(e) => {
                warn!("Health check failed: {:?}", e);
                false
            }
        }
    }

    fn provider_name(&self) -> &'static str {
        "Amadeus"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        let config = GdsConfig::default();
        let client = AmadeusClient {
            http_client: reqwest::Client::new(),
            token_manager: Arc::new(TokenManager::new(&config, reqwest::Client::new())),
            cache: GdsCache::new(),
            base_url: config.amadeus_base_url.clone(),
            max_retries: 3,
        };

        assert_eq!(client.parse_duration(&Some("PT7H30M".to_string())), 450);
        assert_eq!(client.parse_duration(&Some("PT2H".to_string())), 120);
        assert_eq!(client.parse_duration(&Some("PT45M".to_string())), 45);
        assert_eq!(client.parse_duration(&None), 0);
    }

    #[test]
    fn test_cache_key_building() {
        use vaya_common::Date;

        let request = FlightSearchRequest::one_way(IataCode::KUL, IataCode::NRT, Date::today());

        let key = AmadeusClient::build_cache_key(&request);
        assert!(key.contains("KUL"));
        assert!(key.contains("NRT"));
    }
}
