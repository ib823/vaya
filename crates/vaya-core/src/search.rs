//! Flight search service

use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info};

use vaya_cache::Cache;
use vaya_common::{Date, Timestamp};
use vaya_gds::{FlightSearchRequest, GdsProvider};
use vaya_oracle::LSTMPredictor;

use crate::error::{CoreError, CoreResult};
use crate::types::*;

/// Parse date string (YYYY-MM-DD) into Date
fn parse_date(s: &str) -> Option<Date> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let year: i16 = parts[0].parse().ok()?;
    let month: u8 = parts[1].parse().ok()?;
    let day: u8 = parts[2].parse().ok()?;
    let date = Date::new(year, month, day);
    if date.is_valid() {
        Some(date)
    } else {
        None
    }
}

/// Flight search service
pub struct SearchService<G: GdsProvider + Send + Sync> {
    /// GDS provider
    gds: Arc<G>,
    /// Cache for search results
    cache: Arc<Cache<String, Vec<FlightOffer>>>,
    /// Price predictor
    predictor: LSTMPredictor,
    /// Search timeout
    timeout: Duration,
    /// Maximum results
    max_results: usize,
}

impl<G: GdsProvider + Send + Sync> SearchService<G> {
    /// Create new search service
    pub fn new(gds: Arc<G>, cache: Arc<Cache<String, Vec<FlightOffer>>>) -> Self {
        Self {
            gds,
            cache,
            predictor: LSTMPredictor::new(),
            timeout: Duration::from_secs(30),
            max_results: 100,
        }
    }

    /// Set search timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set max results
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Search for flights
    pub async fn search(&self, request: &SearchRequest) -> CoreResult<SearchResponse> {
        // Validate request
        request.validate().map_err(CoreError::InvalidSearchParams)?;

        info!(
            "Searching flights: {} -> {} on {}",
            request.origin, request.destination, request.departure_date
        );

        // Check cache first
        let cache_key = self.build_cache_key(request);
        if let Some(cached) = self.cache.get(&cache_key) {
            debug!("Cache hit for search: {}", cache_key);
            return Ok(SearchResponse {
                offers: cached,
                search_id: cache_key,
                cached: true,
                price_insight: None,
            });
        }

        // Build GDS search params
        let gds_params = self.build_gds_params(request)?;

        // Execute search with timeout
        let search_result =
            tokio::time::timeout(self.timeout, self.gds.search_flights(&gds_params))
                .await
                .map_err(|_| CoreError::SearchTimeout)?
                .map_err(|e| CoreError::GdsError(e.to_string()))?;

        // Convert GDS results to our types
        let mut offers = self.convert_gds_offers(&search_result)?;

        // Filter and sort
        offers = self.filter_offers(offers, request);
        offers.sort_by(|a, b| a.price.amount.cmp(&b.price.amount));

        // Limit results
        if offers.len() > self.max_results {
            offers.truncate(self.max_results);
        }

        if offers.is_empty() {
            return Err(CoreError::NoFlightsFound {
                origin: request.origin.as_str().to_string(),
                destination: request.destination.as_str().to_string(),
            });
        }

        // Cache the results (5 minute TTL)
        self.cache.insert(
            cache_key.clone(),
            offers.clone(),
            Some(Duration::from_secs(300)),
        );

        // Calculate price insight
        let price_insight = self.calculate_insight(request, &offers);

        Ok(SearchResponse {
            offers,
            search_id: cache_key,
            cached: false,
            price_insight,
        })
    }

    /// Build cache key from search request
    fn build_cache_key(&self, request: &SearchRequest) -> String {
        format!(
            "search:{}:{}:{}:{}:{}:{}",
            request.origin,
            request.destination,
            request.departure_date,
            request.return_date.as_deref().unwrap_or(""),
            request.passengers.total(),
            request.cabin_class.code(),
        )
    }

    /// Build GDS search params
    fn build_gds_params(&self, request: &SearchRequest) -> CoreResult<FlightSearchRequest> {
        // Parse departure date (YYYY-MM-DD)
        let departure_date = parse_date(&request.departure_date).ok_or_else(|| {
            CoreError::InvalidSearchParams("Invalid departure date format".to_string())
        })?;

        // Parse return date if present
        let return_date = match &request.return_date {
            Some(d) => Some(parse_date(d).ok_or_else(|| {
                CoreError::InvalidSearchParams("Invalid return date format".to_string())
            })?),
            None => None,
        };

        // Map cabin class
        let cabin_class = match request.cabin_class {
            CabinClass::Economy => vaya_gds::CabinClass::Economy,
            CabinClass::PremiumEconomy => vaya_gds::CabinClass::PremiumEconomy,
            CabinClass::Business => vaya_gds::CabinClass::Business,
            CabinClass::First => vaya_gds::CabinClass::First,
        };

        Ok(FlightSearchRequest {
            origin: request.origin,
            destination: request.destination,
            departure_date,
            return_date,
            adults: request.passengers.adults,
            children: request.passengers.children,
            infants: request.passengers.infants,
            cabin_class,
            direct_only: request.direct_only,
            max_results: request.max_results.unwrap_or(50) as u32,
            currency: request.currency,
        })
    }

    /// Convert GDS offers to core types
    fn convert_gds_offers(
        &self,
        gds_offers: &[vaya_gds::FlightOffer],
    ) -> CoreResult<Vec<FlightOffer>> {
        let mut offers = Vec::with_capacity(gds_offers.len());

        for gds_offer in gds_offers {
            let offer = self.convert_single_offer(gds_offer)?;
            offers.push(offer);
        }

        Ok(offers)
    }

    /// Convert a single GDS offer
    fn convert_single_offer(&self, gds: &vaya_gds::FlightOffer) -> CoreResult<FlightOffer> {
        let outbound = self.convert_journey(&gds.outbound)?;
        let inbound = gds
            .return_itinerary
            .as_ref()
            .map(|r| self.convert_journey(r))
            .transpose()?;

        // Get cabin class from first segment
        let cabin_class = gds
            .outbound
            .segments
            .first()
            .map(|s| match s.cabin_class {
                vaya_gds::CabinClass::Economy => CabinClass::Economy,
                vaya_gds::CabinClass::PremiumEconomy => CabinClass::PremiumEconomy,
                vaya_gds::CabinClass::Business => CabinClass::Business,
                vaya_gds::CabinClass::First => CabinClass::First,
            })
            .unwrap_or(CabinClass::Economy);

        // Build fare conditions from gds fare rules
        let fare_conditions = gds
            .fare_rules
            .as_ref()
            .map(|rules| FareConditions {
                cancellation: if rules.refundable {
                    "Refundable".to_string()
                } else {
                    "Non-refundable".to_string()
                },
                changes: if rules.changeable {
                    "Changeable with fee".to_string()
                } else {
                    "Non-changeable".to_string()
                },
                refund: if rules.refundable {
                    "Refundable".to_string()
                } else {
                    "Non-refundable".to_string()
                },
                fare_family: None,
            })
            .unwrap_or(FareConditions {
                cancellation: "See fare rules".to_string(),
                changes: "See fare rules".to_string(),
                refund: "See fare rules".to_string(),
                fare_family: None,
            });

        // Build baggage allowance
        let baggage_included = gds
            .fare_rules
            .as_ref()
            .and_then(|r| r.baggage.as_ref())
            .map(|b| BaggageAllowance {
                cabin: if b.carry_on {
                    "7kg".to_string()
                } else {
                    "None".to_string()
                },
                checked: format!("{}x{}kg", b.checked_bags, b.weight_kg.unwrap_or(23)),
                extra_cost: None,
            })
            .unwrap_or(BaggageAllowance {
                cabin: "7kg".to_string(),
                checked: "1x23kg".to_string(),
                extra_cost: None,
            });

        let refundable = gds
            .fare_rules
            .as_ref()
            .map(|r| r.refundable)
            .unwrap_or(false);

        Ok(FlightOffer {
            id: gds.id.clone(),
            airlines: gds.airlines(),
            outbound,
            inbound,
            price: gds.price.total,
            price_breakdown: vec![], // Would convert from gds.price per passenger
            fare_conditions,
            cabin_class,
            seats_remaining: gds.available_seats.map(|s| s as u8),
            refundable,
            baggage_included,
            expires_at: gds
                .expires_at
                .unwrap_or_else(|| Timestamp::now().add_mins(30)),
            source: "amadeus".to_string(),
        })
    }

    /// Convert GDS journey (itinerary) to core journey
    fn convert_journey(&self, itinerary: &vaya_gds::Itinerary) -> CoreResult<FlightJourney> {
        let segments: Vec<FlightSegment> = itinerary
            .segments
            .iter()
            .map(|s| {
                let cabin_class = match s.cabin_class {
                    vaya_gds::CabinClass::Economy => CabinClass::Economy,
                    vaya_gds::CabinClass::PremiumEconomy => CabinClass::PremiumEconomy,
                    vaya_gds::CabinClass::Business => CabinClass::Business,
                    vaya_gds::CabinClass::First => CabinClass::First,
                };

                FlightSegment {
                    id: format!("{}_{}", s.airline, s.flight_number),
                    airline: s.airline,
                    flight_number: s.flight_number.clone(),
                    operating_carrier: None, // GDS segment doesn't have this field
                    origin: s.departure.airport,
                    departure_time: s.departure.datetime.to_string(),
                    departure_terminal: s.departure.terminal.clone(),
                    destination: s.arrival.airport,
                    arrival_time: s.arrival.datetime.to_string(),
                    arrival_terminal: s.arrival.terminal.clone(),
                    duration_minutes: s.duration_minutes,
                    aircraft: s.aircraft.clone(),
                    cabin_class,
                    booking_class: s.booking_class.clone().unwrap_or_else(|| "Y".to_string()),
                }
            })
            .collect();

        Ok(FlightJourney {
            duration_minutes: itinerary.total_duration_minutes,
            stops: itinerary.total_stops() as u8,
            segments,
        })
    }

    /// Filter offers based on request criteria
    fn filter_offers(&self, offers: Vec<FlightOffer>, request: &SearchRequest) -> Vec<FlightOffer> {
        offers
            .into_iter()
            .filter(|o| {
                // Filter direct only if requested
                if request.direct_only && o.outbound.stops > 0 {
                    return false;
                }
                true
            })
            .collect()
    }

    /// Calculate price insight
    fn calculate_insight(
        &self,
        _request: &SearchRequest,
        offers: &[FlightOffer],
    ) -> Option<SearchPriceInsight> {
        if offers.is_empty() {
            return None;
        }

        let prices: Vec<i64> = offers.iter().map(|o| o.price.amount.as_i64()).collect();
        let min_price = *prices.iter().min().unwrap_or(&0);
        let max_price = *prices.iter().max().unwrap_or(&0);
        let avg_price = prices.iter().sum::<i64>() / prices.len() as i64;

        Some(SearchPriceInsight {
            min_price,
            max_price,
            avg_price,
            currency: offers[0].price.currency,
            recommendation: if min_price <= avg_price {
                "Good prices available".to_string()
            } else {
                "Prices above average".to_string()
            },
        })
    }

    /// Get offer by ID
    pub async fn get_offer(&self, offer_id: &str) -> CoreResult<FlightOffer> {
        // Search all cached results for the offer
        // In production, would have a separate offer cache
        Err(CoreError::FareNotAvailable(format!(
            "Offer {} not found or expired",
            offer_id
        )))
    }
}

/// Search response
#[derive(Debug, Clone)]
pub struct SearchResponse {
    /// Flight offers
    pub offers: Vec<FlightOffer>,
    /// Search ID for reference
    pub search_id: String,
    /// Whether result was from cache
    pub cached: bool,
    /// Price insight
    pub price_insight: Option<SearchPriceInsight>,
}

/// Search price insight
#[derive(Debug, Clone)]
pub struct SearchPriceInsight {
    /// Minimum price found
    pub min_price: i64,
    /// Maximum price found
    pub max_price: i64,
    /// Average price
    pub avg_price: i64,
    /// Currency
    pub currency: vaya_common::CurrencyCode,
    /// Recommendation text
    pub recommendation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        // Would test cache key generation
    }
}
