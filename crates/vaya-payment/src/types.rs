//! Payment types

use vaya_common::{CurrencyCode, MinorUnits, Price, Timestamp};

/// Payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentStatus {
    /// Payment created, awaiting processing
    Pending,
    /// Processing (3DS, bank redirect, etc.)
    Processing,
    /// Requires additional authentication
    RequiresAction,
    /// Payment succeeded
    Succeeded,
    /// Payment failed
    Failed,
    /// Payment cancelled
    Cancelled,
    /// Fully refunded
    Refunded,
    /// Partially refunded
    PartiallyRefunded,
    /// Disputed/chargeback
    Disputed,
}

impl PaymentStatus {
    /// Is this a terminal (final) status?
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::Refunded
        )
    }

    /// Is payment successful?
    #[must_use]
    pub const fn is_successful(&self) -> bool {
        matches!(self, Self::Succeeded | Self::PartiallyRefunded)
    }

    /// Can this payment be refunded?
    #[must_use]
    pub const fn can_refund(&self) -> bool {
        matches!(self, Self::Succeeded | Self::PartiallyRefunded)
    }

    /// Display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Processing => "Processing",
            Self::RequiresAction => "Action Required",
            Self::Succeeded => "Succeeded",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::Refunded => "Refunded",
            Self::PartiallyRefunded => "Partially Refunded",
            Self::Disputed => "Disputed",
        }
    }
}

/// Payment method type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PaymentMethodType {
    /// Credit/debit card
    Card,
    /// FPX (Malaysian bank transfer)
    Fpx,
    /// `GrabPay`
    GrabPay,
    /// Touch 'n Go eWallet
    TngEwallet,
    /// Boost
    Boost,
    /// Bank transfer
    BankTransfer,
}

impl PaymentMethodType {
    /// Stripe payment method type string
    #[must_use]
    pub const fn stripe_type(&self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Fpx => "fpx",
            Self::GrabPay => "grabpay",
            Self::TngEwallet => "tng_ewallet",
            Self::Boost => "boost",
            Self::BankTransfer => "bank_transfer",
        }
    }

    /// Display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Card => "Credit/Debit Card",
            Self::Fpx => "FPX Online Banking",
            Self::GrabPay => "GrabPay",
            Self::TngEwallet => "Touch 'n Go eWallet",
            Self::Boost => "Boost",
            Self::BankTransfer => "Bank Transfer",
        }
    }
}

/// Card brand
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardBrand {
    /// Visa
    Visa,
    /// Mastercard
    Mastercard,
    /// American Express
    Amex,
    /// Other/unknown
    Other(String),
}

impl CardBrand {
    /// Parse from Stripe brand string
    #[must_use]
    pub fn from_stripe(brand: &str) -> Self {
        match brand.to_lowercase().as_str() {
            "visa" => Self::Visa,
            "mastercard" => Self::Mastercard,
            "amex" | "american_express" => Self::Amex,
            other => Self::Other(other.to_string()),
        }
    }
}

/// Payment method details
#[derive(Debug, Clone)]
pub enum PaymentMethodDetails {
    /// Card payment
    Card {
        /// Card brand
        brand: CardBrand,
        /// Last 4 digits
        last4: String,
        /// Expiry month
        exp_month: u8,
        /// Expiry year
        exp_year: u16,
    },
    /// FPX payment
    Fpx {
        /// Bank code
        bank: String,
    },
    /// `GrabPay`
    GrabPay,
    /// Other payment method
    Other {
        /// Method type
        method_type: String,
    },
}

/// Payment request
#[derive(Debug, Clone)]
pub struct PaymentRequest {
    /// Amount to charge
    pub amount: Price,
    /// Currency (defaults to MYR)
    pub currency: CurrencyCode,
    /// Booking reference
    pub booking_ref: String,
    /// Customer email
    pub customer_email: String,
    /// Customer name
    pub customer_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Idempotency key (to prevent duplicate charges)
    pub idempotency_key: Option<String>,
    /// Allowed payment methods
    pub allowed_methods: Vec<PaymentMethodType>,
    /// Metadata
    pub metadata: std::collections::HashMap<String, String>,
    /// Return URL after payment
    pub return_url: Option<String>,
}

impl Default for PaymentRequest {
    fn default() -> Self {
        Self {
            amount: Price::new(MinorUnits::ZERO, CurrencyCode::MYR),
            currency: CurrencyCode::MYR,
            booking_ref: String::new(),
            customer_email: String::new(),
            customer_name: None,
            description: None,
            idempotency_key: None,
            allowed_methods: vec![PaymentMethodType::Card, PaymentMethodType::Fpx],
            metadata: std::collections::HashMap::new(),
            return_url: None,
        }
    }
}

impl PaymentRequest {
    /// Create new payment request
    #[must_use]
    pub fn new(
        amount: Price,
        booking_ref: impl Into<String>,
        customer_email: impl Into<String>,
    ) -> Self {
        Self {
            amount,
            currency: amount.currency,
            booking_ref: booking_ref.into(),
            customer_email: customer_email.into(),
            ..Default::default()
        }
    }

    /// Set description
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set return URL
    #[must_use]
    pub fn with_return_url(mut self, url: impl Into<String>) -> Self {
        self.return_url = Some(url.into());
        self
    }

    /// Set idempotency key
    #[must_use]
    pub fn with_idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Validate the request
    pub fn validate(&self) -> crate::PaymentResult<()> {
        if self.amount.amount == MinorUnits::ZERO {
            return Err(crate::PaymentError::AmountTooSmall {
                minimum: "0.01".to_string(),
            });
        }
        if self.booking_ref.is_empty() {
            return Err(crate::PaymentError::Configuration(
                "Booking reference is required".to_string(),
            ));
        }
        if self.customer_email.is_empty() {
            return Err(crate::PaymentError::Configuration(
                "Customer email is required".to_string(),
            ));
        }
        Ok(())
    }
}

/// Payment intent (Stripe terminology)
#[derive(Debug, Clone)]
pub struct PaymentIntent {
    /// Stripe payment intent ID
    pub id: String,
    /// Client secret for frontend
    pub client_secret: String,
    /// Amount
    pub amount: Price,
    /// Status
    pub status: PaymentStatus,
    /// Payment method used (if any)
    pub payment_method: Option<PaymentMethodDetails>,
    /// Created timestamp
    pub created_at: Timestamp,
    /// Last updated timestamp
    pub updated_at: Timestamp,
    /// Booking reference
    pub booking_ref: String,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Requires action URL (for 3DS, FPX redirect)
    pub next_action_url: Option<String>,
}

impl PaymentIntent {
    /// Check if payment requires user action
    #[must_use]
    pub fn requires_action(&self) -> bool {
        self.status == PaymentStatus::RequiresAction && self.next_action_url.is_some()
    }

    /// Check if payment is complete
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.status.is_terminal()
    }
}

/// Refund request
#[derive(Debug, Clone)]
pub struct RefundRequest {
    /// Payment ID to refund
    pub payment_id: String,
    /// Amount to refund (None = full refund)
    pub amount: Option<Price>,
    /// Reason for refund
    pub reason: RefundReason,
    /// Idempotency key
    pub idempotency_key: Option<String>,
}

/// Refund reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefundReason {
    /// Customer requested
    CustomerRequest,
    /// Duplicate charge
    Duplicate,
    /// Fraudulent
    Fraudulent,
    /// Booking cancelled
    BookingCancelled,
    /// Other reason
    Other,
}

impl RefundReason {
    /// Stripe reason string
    #[must_use]
    pub const fn stripe_reason(&self) -> &'static str {
        match self {
            Self::CustomerRequest | Self::BookingCancelled | Self::Other => "requested_by_customer",
            Self::Duplicate => "duplicate",
            Self::Fraudulent => "fraudulent",
        }
    }
}

/// Refund result
#[derive(Debug, Clone)]
pub struct Refund {
    /// Refund ID
    pub id: String,
    /// Payment ID
    pub payment_id: String,
    /// Amount refunded
    pub amount: Price,
    /// Status
    pub status: RefundStatus,
    /// Created timestamp
    pub created_at: Timestamp,
    /// Reason
    pub reason: RefundReason,
}

/// Refund status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefundStatus {
    /// Refund pending
    Pending,
    /// Refund succeeded
    Succeeded,
    /// Refund failed
    Failed,
    /// Refund cancelled
    Cancelled,
}

impl RefundStatus {
    /// Is this a terminal status?
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

/// Webhook event
#[derive(Debug, Clone)]
pub struct WebhookEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: WebhookEventType,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Related payment ID (if applicable)
    pub payment_id: Option<String>,
    /// Related refund ID (if applicable)
    pub refund_id: Option<String>,
    /// Raw event data
    pub data: serde_json::Value,
}

/// Webhook event types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WebhookEventType {
    /// Payment succeeded
    PaymentIntentSucceeded,
    /// Payment failed
    PaymentIntentFailed,
    /// Payment requires action
    PaymentIntentRequiresAction,
    /// Payment cancelled
    PaymentIntentCancelled,
    /// Refund succeeded
    ChargeRefunded,
    /// Refund updated
    RefundUpdated,
    /// Dispute created
    ChargeDisputeCreated,
    /// Dispute closed
    ChargeDisputeClosed,
    /// Unknown event
    Unknown(String),
}

impl WebhookEventType {
    /// Parse from Stripe event type
    #[must_use]
    pub fn from_stripe(event_type: &str) -> Self {
        match event_type {
            "payment_intent.succeeded" => Self::PaymentIntentSucceeded,
            "payment_intent.payment_failed" => Self::PaymentIntentFailed,
            "payment_intent.requires_action" => Self::PaymentIntentRequiresAction,
            "payment_intent.canceled" => Self::PaymentIntentCancelled,
            "charge.refunded" => Self::ChargeRefunded,
            "refund.updated" => Self::RefundUpdated,
            "charge.dispute.created" => Self::ChargeDisputeCreated,
            "charge.dispute.closed" => Self::ChargeDisputeClosed,
            other => Self::Unknown(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_status() {
        assert!(PaymentStatus::Succeeded.is_terminal());
        assert!(PaymentStatus::Failed.is_terminal());
        assert!(!PaymentStatus::Processing.is_terminal());

        assert!(PaymentStatus::Succeeded.is_successful());
        assert!(!PaymentStatus::Failed.is_successful());

        assert!(PaymentStatus::Succeeded.can_refund());
        assert!(!PaymentStatus::Pending.can_refund());
    }

    #[test]
    fn test_payment_method_type() {
        assert_eq!(PaymentMethodType::Card.stripe_type(), "card");
        assert_eq!(PaymentMethodType::Fpx.stripe_type(), "fpx");
    }

    #[test]
    fn test_card_brand() {
        assert_eq!(CardBrand::from_stripe("visa"), CardBrand::Visa);
        assert_eq!(CardBrand::from_stripe("MASTERCARD"), CardBrand::Mastercard);
        assert!(matches!(
            CardBrand::from_stripe("unknown"),
            CardBrand::Other(_)
        ));
    }

    #[test]
    fn test_payment_request() {
        let amount = Price::new(MinorUnits::new(50000), CurrencyCode::MYR);
        let request = PaymentRequest::new(amount, "VAY123", "user@example.com")
            .with_description("Flight booking")
            .with_metadata("flight", "MH123");

        assert_eq!(request.booking_ref, "VAY123");
        assert_eq!(request.customer_email, "user@example.com");
        assert!(request.description.is_some());
        assert!(request.metadata.contains_key("flight"));
    }

    #[test]
    fn test_payment_request_validation() {
        let amount = Price::new(MinorUnits::ZERO, CurrencyCode::MYR);
        let request = PaymentRequest::new(amount, "VAY123", "user@example.com");
        assert!(request.validate().is_err());

        let amount = Price::new(MinorUnits::new(50000), CurrencyCode::MYR);
        let request = PaymentRequest::new(amount, "VAY123", "user@example.com");
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_webhook_event_type() {
        assert_eq!(
            WebhookEventType::from_stripe("payment_intent.succeeded"),
            WebhookEventType::PaymentIntentSucceeded
        );
        assert!(matches!(
            WebhookEventType::from_stripe("unknown.event"),
            WebhookEventType::Unknown(_)
        ));
    }
}
