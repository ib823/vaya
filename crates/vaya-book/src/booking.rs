//! Booking types and state machine

use ring::rand::{SecureRandom, SystemRandom};
use time::OffsetDateTime;
use vaya_common::{CurrencyCode, MinorUnits};
use vaya_search::FlightOffer;

use crate::passenger::Passenger;
use crate::payment::PaymentRecord;
use crate::{BookError, BookResult};

/// Booking status (state machine)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookingStatus {
    /// Initial state - booking created but not confirmed
    Pending,
    /// Offer confirmed with provider, awaiting payment
    Confirmed,
    /// Payment received, awaiting ticketing
    PaymentReceived,
    /// Ticketing in progress
    Ticketing,
    /// Fully ticketed
    Ticketed,
    /// Booking cancelled
    Cancelled,
    /// Booking expired (timeout)
    Expired,
    /// Refund in progress
    RefundPending,
    /// Fully refunded
    Refunded,
    /// Booking failed (error state)
    Failed,
}

impl BookingStatus {
    /// Get status as string
    pub fn as_str(&self) -> &'static str {
        match self {
            BookingStatus::Pending => "PENDING",
            BookingStatus::Confirmed => "CONFIRMED",
            BookingStatus::PaymentReceived => "PAYMENT_RECEIVED",
            BookingStatus::Ticketing => "TICKETING",
            BookingStatus::Ticketed => "TICKETED",
            BookingStatus::Cancelled => "CANCELLED",
            BookingStatus::Expired => "EXPIRED",
            BookingStatus::RefundPending => "REFUND_PENDING",
            BookingStatus::Refunded => "REFUNDED",
            BookingStatus::Failed => "FAILED",
        }
    }

    /// Check if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            BookingStatus::Ticketed
                | BookingStatus::Cancelled
                | BookingStatus::Expired
                | BookingStatus::Refunded
                | BookingStatus::Failed
        )
    }

    /// Check if booking can be cancelled from this state
    pub fn can_cancel(&self) -> bool {
        matches!(
            self,
            BookingStatus::Pending
                | BookingStatus::Confirmed
                | BookingStatus::PaymentReceived
                | BookingStatus::Ticketed
        )
    }

    /// Check if booking can be ticketed from this state
    pub fn can_ticket(&self) -> bool {
        matches!(self, BookingStatus::PaymentReceived)
    }

    /// Check if booking can receive payment
    pub fn can_pay(&self) -> bool {
        matches!(self, BookingStatus::Confirmed)
    }

    /// Validate state transition
    pub fn can_transition_to(&self, target: BookingStatus) -> bool {
        match (self, target) {
            // From Pending
            (BookingStatus::Pending, BookingStatus::Confirmed) => true,
            (BookingStatus::Pending, BookingStatus::Expired) => true,
            (BookingStatus::Pending, BookingStatus::Cancelled) => true,
            (BookingStatus::Pending, BookingStatus::Failed) => true,

            // From Confirmed
            (BookingStatus::Confirmed, BookingStatus::PaymentReceived) => true,
            (BookingStatus::Confirmed, BookingStatus::Expired) => true,
            (BookingStatus::Confirmed, BookingStatus::Cancelled) => true,
            (BookingStatus::Confirmed, BookingStatus::Failed) => true,

            // From PaymentReceived
            (BookingStatus::PaymentReceived, BookingStatus::Ticketing) => true,
            (BookingStatus::PaymentReceived, BookingStatus::Cancelled) => true,
            (BookingStatus::PaymentReceived, BookingStatus::RefundPending) => true,
            (BookingStatus::PaymentReceived, BookingStatus::Failed) => true,

            // From Ticketing
            (BookingStatus::Ticketing, BookingStatus::Ticketed) => true,
            (BookingStatus::Ticketing, BookingStatus::Failed) => true,

            // From Ticketed
            (BookingStatus::Ticketed, BookingStatus::Cancelled) => true,
            (BookingStatus::Ticketed, BookingStatus::RefundPending) => true,

            // From RefundPending
            (BookingStatus::RefundPending, BookingStatus::Refunded) => true,
            (BookingStatus::RefundPending, BookingStatus::Failed) => true,

            // All other transitions invalid
            _ => false,
        }
    }
}

/// A booking record
#[derive(Debug, Clone)]
pub struct Booking {
    /// Unique booking reference (PNR)
    pub pnr: String,
    /// User ID who made the booking
    pub user_id: String,
    /// Current status
    pub status: BookingStatus,
    /// Flight offer
    pub offer: FlightOffer,
    /// Passengers
    pub passengers: Vec<Passenger>,
    /// Payment records
    pub payments: Vec<PaymentRecord>,
    /// Total price
    pub total_price: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
    /// Confirmation deadline (Unix timestamp)
    pub confirm_deadline: Option<i64>,
    /// Payment deadline (Unix timestamp)
    pub payment_deadline: Option<i64>,
    /// Ticketing deadline (Unix timestamp)
    pub ticketing_deadline: Option<i64>,
    /// Provider booking reference
    pub provider_ref: Option<String>,
    /// Airline PNR (after ticketing)
    pub airline_pnr: Option<String>,
    /// Ticket numbers
    pub ticket_numbers: Vec<String>,
    /// Status history
    pub history: Vec<StatusChange>,
    /// Version for optimistic locking
    pub version: u32,
    /// Notes
    pub notes: Vec<BookingNote>,
}

impl Booking {
    /// Create a new booking
    pub fn new(user_id: impl Into<String>, offer: FlightOffer, passengers: Vec<Passenger>) -> BookResult<Self> {
        let pnr = generate_pnr()?;
        let now = OffsetDateTime::now_utc().unix_timestamp();

        // Calculate total price
        let total_price = offer.price.total();
        let currency = offer.price.currency;

        let mut booking = Self {
            pnr: pnr.clone(),
            user_id: user_id.into(),
            status: BookingStatus::Pending,
            offer,
            passengers,
            payments: Vec::new(),
            total_price,
            currency,
            created_at: now,
            updated_at: now,
            confirm_deadline: Some(now + 900), // 15 minutes to confirm
            payment_deadline: None,
            ticketing_deadline: None,
            provider_ref: None,
            airline_pnr: None,
            ticket_numbers: Vec::new(),
            history: Vec::new(),
            version: 1,
            notes: Vec::new(),
        };

        // Record initial state
        booking.history.push(StatusChange {
            from: None,
            to: BookingStatus::Pending,
            timestamp: now,
            reason: "Booking created".into(),
            actor: booking.user_id.clone(),
        });

        Ok(booking)
    }

    /// Transition to a new status
    pub fn transition(&mut self, new_status: BookingStatus, reason: &str, actor: &str) -> BookResult<()> {
        if !self.status.can_transition_to(new_status) {
            return Err(BookError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: new_status.as_str().to_string(),
            });
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();

        self.history.push(StatusChange {
            from: Some(self.status),
            to: new_status,
            timestamp: now,
            reason: reason.to_string(),
            actor: actor.to_string(),
        });

        self.status = new_status;
        self.updated_at = now;
        self.version += 1;

        // Set deadlines based on new status
        match new_status {
            BookingStatus::Confirmed => {
                self.payment_deadline = Some(now + 86400); // 24 hours to pay
            }
            BookingStatus::PaymentReceived => {
                self.ticketing_deadline = Some(now + 3600); // 1 hour to ticket
            }
            _ => {}
        }

        Ok(())
    }

    /// Confirm booking (after provider confirmation)
    pub fn confirm(&mut self, provider_ref: &str, actor: &str) -> BookResult<()> {
        self.provider_ref = Some(provider_ref.to_string());
        self.transition(BookingStatus::Confirmed, "Provider confirmed", actor)
    }

    /// Mark payment received
    pub fn mark_paid(&mut self, payment: PaymentRecord, actor: &str) -> BookResult<()> {
        if !self.status.can_pay() {
            return Err(BookError::InvalidStateTransition {
                from: self.status.as_str().to_string(),
                to: "PAYMENT_RECEIVED".to_string(),
            });
        }

        self.payments.push(payment);
        self.transition(BookingStatus::PaymentReceived, "Payment received", actor)
    }

    /// Start ticketing process
    pub fn start_ticketing(&mut self, actor: &str) -> BookResult<()> {
        self.transition(BookingStatus::Ticketing, "Ticketing started", actor)
    }

    /// Mark as ticketed
    pub fn mark_ticketed(&mut self, airline_pnr: &str, tickets: Vec<String>, actor: &str) -> BookResult<()> {
        self.airline_pnr = Some(airline_pnr.to_string());
        self.ticket_numbers = tickets;
        self.transition(BookingStatus::Ticketed, "Ticketing complete", actor)
    }

    /// Cancel booking
    pub fn cancel(&mut self, reason: &str, actor: &str) -> BookResult<()> {
        if !self.status.can_cancel() {
            return Err(BookError::NotCancellable(format!(
                "Cannot cancel booking in {} status",
                self.status.as_str()
            )));
        }

        // If payment was received, initiate refund
        if self.has_payment() && self.status != BookingStatus::Pending && self.status != BookingStatus::Confirmed {
            self.transition(BookingStatus::RefundPending, reason, actor)?;
        } else {
            self.transition(BookingStatus::Cancelled, reason, actor)?;
        }

        Ok(())
    }

    /// Mark as refunded
    pub fn mark_refunded(&mut self, actor: &str) -> BookResult<()> {
        self.transition(BookingStatus::Refunded, "Refund complete", actor)
    }

    /// Check if booking is expired
    pub fn check_expiry(&mut self) -> bool {
        if self.status.is_terminal() {
            return false;
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();

        let is_expired = match self.status {
            BookingStatus::Pending => {
                self.confirm_deadline.map(|d| now > d).unwrap_or(false)
            }
            BookingStatus::Confirmed => {
                self.payment_deadline.map(|d| now > d).unwrap_or(false)
            }
            _ => false,
        };

        if is_expired {
            let _ = self.transition(BookingStatus::Expired, "Deadline exceeded", "SYSTEM");
        }

        is_expired
    }

    /// Check if booking has any payment
    pub fn has_payment(&self) -> bool {
        !self.payments.is_empty()
    }

    /// Get total paid amount
    pub fn total_paid(&self) -> MinorUnits {
        let sum: i64 = self.payments.iter().map(|p| p.amount.as_i64()).sum();
        MinorUnits::new(sum)
    }

    /// Add a note
    pub fn add_note(&mut self, content: &str, author: &str) {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.notes.push(BookingNote {
            content: content.to_string(),
            author: author.to_string(),
            timestamp: now,
        });
        self.updated_at = now;
    }

    /// Get time remaining until next deadline
    pub fn time_to_deadline(&self) -> Option<i64> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let deadline = match self.status {
            BookingStatus::Pending => self.confirm_deadline,
            BookingStatus::Confirmed => self.payment_deadline,
            BookingStatus::PaymentReceived => self.ticketing_deadline,
            _ => None,
        };

        deadline.map(|d| (d - now).max(0))
    }
}

/// Status change record
#[derive(Debug, Clone)]
pub struct StatusChange {
    /// Previous status
    pub from: Option<BookingStatus>,
    /// New status
    pub to: BookingStatus,
    /// Timestamp
    pub timestamp: i64,
    /// Reason for change
    pub reason: String,
    /// Actor (user/system) who made the change
    pub actor: String,
}

/// Booking note
#[derive(Debug, Clone)]
pub struct BookingNote {
    /// Note content
    pub content: String,
    /// Author
    pub author: String,
    /// Timestamp
    pub timestamp: i64,
}

/// Generate a PNR (6 alphanumeric characters)
fn generate_pnr() -> BookResult<String> {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Avoid confusing chars (0, O, 1, I)
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 6];

    rng.fill(&mut bytes)
        .map_err(|_| BookError::Internal("Failed to generate PNR".into()))?;

    let pnr: String = bytes
        .iter()
        .map(|b| CHARS[(*b as usize) % CHARS.len()] as char)
        .collect();

    Ok(pnr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vaya_search::{FlightLeg, PriceBreakdown};

    fn mock_offer() -> FlightOffer {
        FlightOffer {
            id: "offer-1".into(),
            outbound: FlightLeg {
                segments: vec![],
                total_duration_minutes: 120,
            },
            inbound: None,
            price: PriceBreakdown {
                base_fare: MinorUnits::new(10000),
                taxes: MinorUnits::new(2000),
                surcharges: MinorUnits::new(500),
                currency: CurrencyCode::SGD,
            },
            price_per_pax: vec![],
            expires_at: None,
            provider: "test".into(),
            refundable: true,
            changeable: true,
            baggage: None,
            fare_rules: None,
        }
    }

    #[test]
    fn test_booking_creation() {
        let offer = mock_offer();
        let booking = Booking::new("user-123", offer, vec![]).unwrap();

        assert_eq!(booking.pnr.len(), 6);
        assert_eq!(booking.status, BookingStatus::Pending);
        assert_eq!(booking.history.len(), 1);
    }

    #[test]
    fn test_status_transitions() {
        assert!(BookingStatus::Pending.can_transition_to(BookingStatus::Confirmed));
        assert!(BookingStatus::Confirmed.can_transition_to(BookingStatus::PaymentReceived));
        assert!(!BookingStatus::Pending.can_transition_to(BookingStatus::Ticketed));
        assert!(!BookingStatus::Ticketed.can_transition_to(BookingStatus::Pending));
    }

    #[test]
    fn test_booking_lifecycle() {
        let offer = mock_offer();
        let mut booking = Booking::new("user-123", offer, vec![]).unwrap();

        // Confirm
        assert!(booking.confirm("PROV-123", "system").is_ok());
        assert_eq!(booking.status, BookingStatus::Confirmed);
        assert!(booking.payment_deadline.is_some());

        // Pay
        let payment = PaymentRecord {
            id: "pay-1".into(),
            amount: MinorUnits::new(12500),
            currency: CurrencyCode::SGD,
            method: crate::payment::PaymentMethod::Card,
            status: crate::payment::PaymentStatus::Completed,
            provider_ref: Some("stripe-123".into()),
            timestamp: 0,
        };
        assert!(booking.mark_paid(payment, "system").is_ok());
        assert_eq!(booking.status, BookingStatus::PaymentReceived);

        // Ticket
        assert!(booking.start_ticketing("system").is_ok());
        assert_eq!(booking.status, BookingStatus::Ticketing);

        assert!(booking.mark_ticketed("ABC123", vec!["TKT001".into()], "system").is_ok());
        assert_eq!(booking.status, BookingStatus::Ticketed);
        assert!(booking.status.is_terminal());
    }

    #[test]
    fn test_cancellation() {
        let offer = mock_offer();
        let mut booking = Booking::new("user-123", offer, vec![]).unwrap();

        assert!(booking.cancel("User requested", "user-123").is_ok());
        assert_eq!(booking.status, BookingStatus::Cancelled);
    }

    #[test]
    fn test_pnr_generation() {
        let pnr = generate_pnr().unwrap();
        assert_eq!(pnr.len(), 6);
        assert!(pnr.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_terminal_states() {
        assert!(BookingStatus::Ticketed.is_terminal());
        assert!(BookingStatus::Cancelled.is_terminal());
        assert!(BookingStatus::Expired.is_terminal());
        assert!(!BookingStatus::Pending.is_terminal());
        assert!(!BookingStatus::Confirmed.is_terminal());
    }
}
