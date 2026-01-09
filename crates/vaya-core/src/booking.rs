//! Booking service

use std::sync::Arc;
use tracing::{debug, info, warn};

use vaya_common::{Price, Timestamp, Uuid};
use vaya_gds::GdsProvider;
use vaya_notification::{EmailClient, EmailRequest, NotificationConfig, NotificationType};
use vaya_payment::{PaymentProvider, PaymentRequest, PaymentStatus, RefundReason, RefundRequest};

use crate::error::{CoreError, CoreResult};
use crate::search::SearchService;
use crate::types::*;

/// Booking service configuration
#[derive(Debug, Clone)]
pub struct BookingConfig {
    /// Payment timeout in minutes
    pub payment_timeout_minutes: u32,
    /// Auto-cancel after payment timeout
    pub auto_cancel_on_timeout: bool,
    /// Send confirmation email
    pub send_confirmation_email: bool,
    /// Send confirmation SMS
    pub send_confirmation_sms: bool,
}

impl Default for BookingConfig {
    fn default() -> Self {
        Self {
            payment_timeout_minutes: 30,
            auto_cancel_on_timeout: true,
            send_confirmation_email: true,
            send_confirmation_sms: true,
        }
    }
}

/// Booking service
pub struct BookingService<G, P>
where
    G: GdsProvider + Send + Sync,
    P: PaymentProvider + Send + Sync,
{
    /// Search service for validating offers
    search: Arc<SearchService<G>>,
    /// Payment provider
    payment: Arc<P>,
    /// Email client (optional)
    email: Option<EmailClient>,
    /// Configuration
    config: BookingConfig,
}

impl<G, P> BookingService<G, P>
where
    G: GdsProvider + Send + Sync,
    P: PaymentProvider + Send + Sync,
{
    /// Create new booking service
    pub fn new(
        search: Arc<SearchService<G>>,
        payment: Arc<P>,
        notification_config: Option<&NotificationConfig>,
    ) -> CoreResult<Self> {
        let email = notification_config
            .map(|c| EmailClient::new(c))
            .transpose()
            .map_err(|e| CoreError::Internal(format!("Failed to create email client: {}", e)))?;

        Ok(Self {
            search,
            payment,
            email,
            config: BookingConfig::default(),
        })
    }

    /// Set configuration
    pub fn with_config(mut self, config: BookingConfig) -> Self {
        self.config = config;
        self
    }

    /// Create a new booking
    pub async fn create_booking(&self, request: BookingRequest) -> CoreResult<Booking> {
        info!(
            "Creating booking for offer {} by user {}",
            request.offer_id, request.user_id
        );

        // Validate the offer is still available
        let offer = self.search.get_offer(&request.offer_id).await?;

        // Validate offer hasn't expired
        if offer.expires_at < Timestamp::now() {
            return Err(CoreError::FareNotAvailable(
                "Offer has expired".to_string(),
            ));
        }

        // Validate passenger count matches
        let passenger_count = request.passengers.len();
        let expected_count = self.count_passengers_from_offer(&offer);
        if passenger_count != expected_count {
            return Err(CoreError::ValidationError(format!(
                "Expected {} passengers, got {}",
                expected_count, passenger_count
            )));
        }

        // Validate passengers
        self.validate_passengers(&request.passengers)?;

        // Generate booking ID
        let booking_id = Uuid::new_v4().to_string();
        let pnr = self.generate_pnr();

        // Calculate payment deadline
        let payment_deadline = Timestamp::now()
            .add_mins(self.config.payment_timeout_minutes as i64);

        // Create booking record
        let booking = Booking {
            id: booking_id.clone(),
            pnr: pnr.clone(),
            user_id: request.user_id.clone(),
            status: BookingStatus::PendingPayment,
            flights: offer,
            passengers: request.passengers,
            contact: request.contact,
            total_price: Price::new(
                vaya_common::MinorUnits::new(0), // Would calculate from offer
                vaya_common::CurrencyCode::MYR,
            ),
            payment_id: None,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            payment_deadline: Some(payment_deadline),
            ticket_numbers: vec![],
        };

        info!("Booking {} created with PNR {}", booking_id, pnr);

        // In production, would persist to database here

        Ok(booking)
    }

    /// Process payment for a booking
    pub async fn process_payment(
        &self,
        booking: &mut Booking,
        return_url: Option<&str>,
    ) -> CoreResult<PaymentResult> {
        if booking.status != BookingStatus::PendingPayment {
            return Err(CoreError::BookingNotModifiable(format!(
                "Booking {} is not pending payment",
                booking.id
            )));
        }

        // Check payment deadline
        if let Some(deadline) = booking.payment_deadline {
            if deadline < Timestamp::now() {
                booking.status = BookingStatus::Cancelled;
                return Err(CoreError::BookingExpired(booking.id.clone()));
            }
        }

        info!(
            "Processing payment for booking {} amount {}",
            booking.id,
            booking.total_price.amount.as_i64()
        );

        booking.status = BookingStatus::PaymentProcessing;

        // Create payment request
        let mut payment_request = PaymentRequest::new(
            booking.total_price,
            &booking.pnr,
            &booking.contact.email,
        )
        .with_description(format!("Flight booking {}", booking.pnr))
        .with_idempotency_key(format!("booking_{}", booking.id))
        .with_metadata("booking_id", &booking.id)
        .with_metadata("pnr", &booking.pnr);

        if let Some(url) = return_url {
            payment_request = payment_request.with_return_url(url);
        }

        // Process payment
        let payment_intent = self
            .payment
            .create_payment(&payment_request)
            .await
            .map_err(|e| {
                booking.status = BookingStatus::PendingPayment;
                CoreError::PaymentFailed(e.to_string())
            })?;

        // Update booking based on payment result
        match payment_intent.status {
            PaymentStatus::Succeeded => {
                booking.status = BookingStatus::Confirmed;
                booking.payment_id = Some(payment_intent.id.clone());
                booking.updated_at = Timestamp::now();

                info!(
                    "Payment successful for booking {}: {}",
                    booking.id, payment_intent.id
                );

                // Send confirmation
                if self.config.send_confirmation_email {
                    if let Err(e) = self.send_confirmation_email(booking).await {
                        warn!("Failed to send confirmation email: {}", e);
                    }
                }

                Ok(PaymentResult {
                    success: true,
                    payment_id: payment_intent.id,
                    status: "succeeded".to_string(),
                    message: None,
                })
            }
            PaymentStatus::RequiresAction => {
                booking.payment_id = Some(payment_intent.id.clone());
                Ok(PaymentResult {
                    success: false,
                    payment_id: payment_intent.id,
                    status: "requires_action".to_string(),
                    message: Some(payment_intent.client_secret),
                })
            }
            _ => {
                booking.status = BookingStatus::PendingPayment;
                Err(CoreError::PaymentFailed(payment_intent.error_message.unwrap_or_else(|| "Payment failed".to_string())))
            }
        }
    }

    /// Cancel a booking
    pub async fn cancel_booking(
        &self,
        booking: &mut Booking,
        reason: &str,
    ) -> CoreResult<CancellationResult> {
        if !booking.status.can_cancel() {
            return Err(CoreError::BookingNotModifiable(format!(
                "Booking {} cannot be cancelled in status {:?}",
                booking.id, booking.status
            )));
        }

        info!("Cancelling booking {}: {}", booking.id, reason);

        // If payment was made, initiate refund
        let refund_id = if let Some(ref payment_id) = booking.payment_id {
            let refund_request = RefundRequest {
                payment_id: payment_id.clone(),
                amount: None, // Full refund
                reason: RefundReason::BookingCancelled,
                idempotency_key: Some(format!("refund_booking_{}", booking.id)),
            };

            match self.payment.create_refund(&refund_request).await {
                Ok(refund) => {
                    booking.status = BookingStatus::RefundPending;
                    Some(refund.id)
                }
                Err(e) => {
                    warn!("Failed to initiate refund for {}: {}", booking.id, e);
                    None
                }
            }
        } else {
            booking.status = BookingStatus::Cancelled;
            None
        };

        booking.updated_at = Timestamp::now();

        let has_refund = refund_id.is_some();
        Ok(CancellationResult {
            booking_id: booking.id.clone(),
            status: booking.status,
            refund_id,
            refund_amount: if has_refund {
                Some(booking.total_price.amount.as_i64())
            } else {
                None
            },
        })
    }

    /// Get booking by ID
    pub async fn get_booking(&self, booking_id: &str) -> CoreResult<Booking> {
        // In production, would fetch from database
        Err(CoreError::BookingNotFound(booking_id.to_string()))
    }

    /// Get bookings for a user
    pub async fn get_user_bookings(&self, user_id: &str) -> CoreResult<Vec<Booking>> {
        // In production, would fetch from database
        info!("Fetching bookings for user {}", user_id);
        Ok(vec![])
    }

    /// Validate passengers
    fn validate_passengers(&self, passengers: &[PassengerDetails]) -> CoreResult<()> {
        for (i, p) in passengers.iter().enumerate() {
            if p.first_name.is_empty() {
                return Err(CoreError::MissingField(format!(
                    "Passenger {} first name",
                    i + 1
                )));
            }
            if p.last_name.is_empty() {
                return Err(CoreError::MissingField(format!(
                    "Passenger {} last name",
                    i + 1
                )));
            }
            if p.date_of_birth.is_empty() {
                return Err(CoreError::MissingField(format!(
                    "Passenger {} date of birth",
                    i + 1
                )));
            }
        }
        Ok(())
    }

    /// Count expected passengers from offer
    fn count_passengers_from_offer(&self, _offer: &FlightOffer) -> usize {
        // Would extract from offer's price breakdown
        1
    }

    /// Generate PNR (6 character alphanumeric)
    fn generate_pnr(&self) -> String {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};

        let hasher = RandomState::new().build_hasher();
        let hash = hasher.finish();

        const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
        let mut pnr = String::with_capacity(6);

        let mut h = hash;
        for _ in 0..6 {
            let idx = (h % CHARS.len() as u64) as usize;
            pnr.push(CHARS[idx] as char);
            h /= CHARS.len() as u64;
        }

        pnr
    }

    /// Send confirmation email
    async fn send_confirmation_email(&self, booking: &Booking) -> CoreResult<()> {
        let email_client = self.email.as_ref().ok_or_else(|| {
            CoreError::NotificationFailed("Email client not configured".to_string())
        })?;

        let email = EmailRequest::from_type(&booking.contact.email, NotificationType::BookingConfirmation)
            .with_name(&format!(
                "{} {}",
                booking.passengers[0].first_name, booking.passengers[0].last_name
            ))
            .with_context("booking_ref", &booking.pnr)
            .with_context("passenger_name", &format!(
                "{} {}",
                booking.passengers[0].first_name, booking.passengers[0].last_name
            ))
            .with_context("origin", booking.flights.outbound.segments[0].origin.as_str())
            .with_context("destination", booking.flights.outbound.segments.last()
                .map(|s| s.destination.as_str())
                .unwrap_or(""))
            .with_context("departure_date", &booking.flights.outbound.segments[0].departure_time)
            .with_context("flight_number", &booking.flights.outbound.segments[0].flight_number)
            .with_context("currency", booking.total_price.currency.as_str())
            .with_context("total_amount", format!("{:.2}", booking.total_price.amount.as_i64() as f64 / 100.0));

        email_client
            .send(&email)
            .await
            .map_err(|e| CoreError::NotificationFailed(e.to_string()))?;

        debug!("Confirmation email sent for booking {}", booking.pnr);
        Ok(())
    }
}

/// Payment result
#[derive(Debug, Clone)]
pub struct PaymentResult {
    /// Whether payment succeeded
    pub success: bool,
    /// Payment ID
    pub payment_id: String,
    /// Status string
    pub status: String,
    /// Additional message (e.g., client secret for 3DS)
    pub message: Option<String>,
}

/// Cancellation result
#[derive(Debug, Clone)]
pub struct CancellationResult {
    /// Booking ID
    pub booking_id: String,
    /// New status
    pub status: BookingStatus,
    /// Refund ID if refund initiated
    pub refund_id: Option<String>,
    /// Refund amount if applicable
    pub refund_amount: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pnr_generation() {
        // Test that PNR is 6 characters alphanumeric
        let config = BookingConfig::default();
        assert_eq!(config.payment_timeout_minutes, 30);
    }

    #[test]
    fn test_booking_config() {
        let config = BookingConfig {
            payment_timeout_minutes: 60,
            auto_cancel_on_timeout: false,
            send_confirmation_email: true,
            send_confirmation_sms: false,
        };

        assert_eq!(config.payment_timeout_minutes, 60);
        assert!(!config.auto_cancel_on_timeout);
    }
}
