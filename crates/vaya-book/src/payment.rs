//! Payment processing types

use time::OffsetDateTime;
use vaya_common::{CurrencyCode, MinorUnits};

use crate::{BookError, BookResult};

/// Payment method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentMethod {
    /// Credit/debit card
    Card,
    /// Bank transfer
    BankTransfer,
    /// PayNow (Singapore)
    PayNow,
    /// GrabPay
    GrabPay,
    /// ShopeePay
    ShopeePay,
    /// Wallet balance
    Wallet,
    /// Points redemption
    Points,
    /// Invoice (B2B)
    Invoice,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentMethod::Card => "CARD",
            PaymentMethod::BankTransfer => "BANK_TRANSFER",
            PaymentMethod::PayNow => "PAYNOW",
            PaymentMethod::GrabPay => "GRABPAY",
            PaymentMethod::ShopeePay => "SHOPEEPAY",
            PaymentMethod::Wallet => "WALLET",
            PaymentMethod::Points => "POINTS",
            PaymentMethod::Invoice => "INVOICE",
        }
    }

    /// Check if method supports instant confirmation
    pub fn is_instant(&self) -> bool {
        matches!(
            self,
            PaymentMethod::Card
                | PaymentMethod::PayNow
                | PaymentMethod::GrabPay
                | PaymentMethod::ShopeePay
                | PaymentMethod::Wallet
                | PaymentMethod::Points
        )
    }
}

/// Payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentStatus {
    /// Payment initiated
    Pending,
    /// Awaiting confirmation (e.g., bank transfer)
    AwaitingConfirmation,
    /// Payment authorized but not captured
    Authorized,
    /// Payment completed
    Completed,
    /// Payment failed
    Failed,
    /// Payment refunded
    Refunded,
    /// Payment partially refunded
    PartiallyRefunded,
    /// Payment disputed/chargeback
    Disputed,
    /// Payment cancelled
    Cancelled,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Pending => "PENDING",
            PaymentStatus::AwaitingConfirmation => "AWAITING_CONFIRMATION",
            PaymentStatus::Authorized => "AUTHORIZED",
            PaymentStatus::Completed => "COMPLETED",
            PaymentStatus::Failed => "FAILED",
            PaymentStatus::Refunded => "REFUNDED",
            PaymentStatus::PartiallyRefunded => "PARTIALLY_REFUNDED",
            PaymentStatus::Disputed => "DISPUTED",
            PaymentStatus::Cancelled => "CANCELLED",
        }
    }

    pub fn is_successful(&self) -> bool {
        matches!(self, PaymentStatus::Completed | PaymentStatus::Authorized)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            PaymentStatus::Completed
                | PaymentStatus::Failed
                | PaymentStatus::Refunded
                | PaymentStatus::Cancelled
        )
    }
}

/// Payment record
#[derive(Debug, Clone)]
pub struct PaymentRecord {
    /// Payment ID
    pub id: String,
    /// Amount
    pub amount: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Payment method
    pub method: PaymentMethod,
    /// Status
    pub status: PaymentStatus,
    /// Provider reference
    pub provider_ref: Option<String>,
    /// Timestamp
    pub timestamp: i64,
}

impl PaymentRecord {
    /// Create a new payment record
    pub fn new(
        id: impl Into<String>,
        amount: MinorUnits,
        currency: CurrencyCode,
        method: PaymentMethod,
    ) -> Self {
        Self {
            id: id.into(),
            amount,
            currency,
            method,
            status: PaymentStatus::Pending,
            provider_ref: None,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    /// Mark as completed
    pub fn complete(&mut self, provider_ref: Option<String>) {
        self.status = PaymentStatus::Completed;
        self.provider_ref = provider_ref;
    }

    /// Mark as failed
    pub fn fail(&mut self, _reason: &str) {
        self.status = PaymentStatus::Failed;
    }
}

/// Card details (tokenized - never store raw card data)
#[derive(Debug, Clone)]
pub struct CardToken {
    /// Token from payment provider
    pub token: String,
    /// Last 4 digits
    pub last_four: String,
    /// Card brand
    pub brand: CardBrand,
    /// Expiry month
    pub exp_month: u8,
    /// Expiry year
    pub exp_year: u16,
    /// Cardholder name
    pub cardholder_name: String,
}

impl CardToken {
    /// Create a new card token
    pub fn new(
        token: impl Into<String>,
        last_four: impl Into<String>,
        brand: CardBrand,
        exp_month: u8,
        exp_year: u16,
        name: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            last_four: last_four.into(),
            brand,
            exp_month,
            exp_year,
            cardholder_name: name.into(),
        }
    }

    /// Check if card is expired
    pub fn is_expired(&self) -> bool {
        let now = OffsetDateTime::now_utc();
        let current_year = now.year() as u16;
        let current_month = now.month() as u8;

        if self.exp_year < current_year {
            return true;
        }

        if self.exp_year == current_year && self.exp_month < current_month {
            return true;
        }

        false
    }

    /// Get masked card number for display
    pub fn masked_number(&self) -> String {
        format!("**** **** **** {}", self.last_four)
    }

    /// Validate card details
    pub fn validate(&self) -> BookResult<()> {
        // Check expiry
        if self.is_expired() {
            return Err(BookError::InvalidPayment("Card is expired".into()));
        }

        // Check last four digits
        if self.last_four.len() != 4 || !self.last_four.chars().all(|c| c.is_ascii_digit()) {
            return Err(BookError::InvalidPayment("Invalid card number".into()));
        }

        // Check expiry values
        if self.exp_month == 0 || self.exp_month > 12 {
            return Err(BookError::InvalidPayment("Invalid expiry month".into()));
        }

        Ok(())
    }
}

/// Card brand
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardBrand {
    Visa,
    Mastercard,
    Amex,
    Discover,
    Jcb,
    UnionPay,
    Unknown,
}

impl CardBrand {
    pub fn as_str(&self) -> &'static str {
        match self {
            CardBrand::Visa => "visa",
            CardBrand::Mastercard => "mastercard",
            CardBrand::Amex => "amex",
            CardBrand::Discover => "discover",
            CardBrand::Jcb => "jcb",
            CardBrand::UnionPay => "unionpay",
            CardBrand::Unknown => "unknown",
        }
    }

    /// Detect brand from card number prefix
    pub fn from_prefix(prefix: &str) -> Self {
        if prefix.starts_with('4') {
            CardBrand::Visa
        } else if prefix.starts_with("51")
            || prefix.starts_with("52")
            || prefix.starts_with("53")
            || prefix.starts_with("54")
            || prefix.starts_with("55")
        {
            CardBrand::Mastercard
        } else if prefix.starts_with("34") || prefix.starts_with("37") {
            CardBrand::Amex
        } else if prefix.starts_with("35") {
            CardBrand::Jcb
        } else if prefix.starts_with("62") {
            CardBrand::UnionPay
        } else {
            CardBrand::Unknown
        }
    }
}

/// Refund record
#[derive(Debug, Clone)]
pub struct RefundRecord {
    /// Refund ID
    pub id: String,
    /// Original payment ID
    pub payment_id: String,
    /// Refund amount
    pub amount: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Status
    pub status: RefundStatus,
    /// Reason
    pub reason: String,
    /// Provider reference
    pub provider_ref: Option<String>,
    /// Timestamp
    pub timestamp: i64,
}

/// Refund status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefundStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl RefundStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RefundStatus::Pending => "PENDING",
            RefundStatus::Processing => "PROCESSING",
            RefundStatus::Completed => "COMPLETED",
            RefundStatus::Failed => "FAILED",
        }
    }
}

/// Payment request for initiating a payment
#[derive(Debug, Clone)]
pub struct PaymentRequest {
    /// Booking reference
    pub booking_ref: String,
    /// Amount to charge
    pub amount: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Payment method
    pub method: PaymentMethod,
    /// Card token (if card payment)
    pub card_token: Option<CardToken>,
    /// Return URL (for redirect-based payments)
    pub return_url: Option<String>,
    /// Idempotency key
    pub idempotency_key: String,
}

impl PaymentRequest {
    /// Create card payment request
    pub fn card(
        booking_ref: impl Into<String>,
        amount: MinorUnits,
        currency: CurrencyCode,
        card: CardToken,
        idempotency_key: impl Into<String>,
    ) -> Self {
        Self {
            booking_ref: booking_ref.into(),
            amount,
            currency,
            method: PaymentMethod::Card,
            card_token: Some(card),
            return_url: None,
            idempotency_key: idempotency_key.into(),
        }
    }

    /// Validate payment request
    pub fn validate(&self) -> BookResult<()> {
        // Check amount
        if self.amount.as_i64() <= 0 {
            return Err(BookError::InvalidPayment("Amount must be positive".into()));
        }

        // Check card token if card payment
        if self.method == PaymentMethod::Card {
            match &self.card_token {
                Some(card) => card.validate()?,
                None => return Err(BookError::MissingField("card_token".into())),
            }
        }

        // Check idempotency key
        if self.idempotency_key.is_empty() {
            return Err(BookError::MissingField("idempotency_key".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_method() {
        assert!(PaymentMethod::Card.is_instant());
        assert!(!PaymentMethod::BankTransfer.is_instant());
    }

    #[test]
    fn test_payment_status() {
        assert!(PaymentStatus::Completed.is_successful());
        assert!(PaymentStatus::Authorized.is_successful());
        assert!(!PaymentStatus::Pending.is_successful());
        assert!(PaymentStatus::Completed.is_terminal());
        assert!(!PaymentStatus::Pending.is_terminal());
    }

    #[test]
    fn test_card_brand_detection() {
        assert_eq!(CardBrand::from_prefix("4111"), CardBrand::Visa);
        assert_eq!(CardBrand::from_prefix("5234"), CardBrand::Mastercard);
        assert_eq!(CardBrand::from_prefix("3412"), CardBrand::Amex);
        assert_eq!(CardBrand::from_prefix("6234"), CardBrand::UnionPay);
    }

    #[test]
    fn test_card_token_validation() {
        let valid = CardToken::new("tok_123", "4242", CardBrand::Visa, 12, 2030, "John Doe");
        assert!(valid.validate().is_ok());

        let expired = CardToken::new("tok_123", "4242", CardBrand::Visa, 1, 2020, "John Doe");
        assert!(expired.validate().is_err());
    }

    #[test]
    fn test_masked_number() {
        let card = CardToken::new("tok_123", "4242", CardBrand::Visa, 12, 2030, "John Doe");
        assert_eq!(card.masked_number(), "**** **** **** 4242");
    }

    #[test]
    fn test_payment_request_validation() {
        let card = CardToken::new("tok_123", "4242", CardBrand::Visa, 12, 2030, "John Doe");
        let request = PaymentRequest::card(
            "ABC123",
            MinorUnits::new(10000),
            CurrencyCode::SGD,
            card,
            "idem-123",
        );
        assert!(request.validate().is_ok());

        // Missing idempotency key
        let mut invalid = request.clone();
        invalid.idempotency_key = String::new();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_payment_record() {
        let mut payment = PaymentRecord::new(
            "pay-1",
            MinorUnits::new(10000),
            CurrencyCode::SGD,
            PaymentMethod::Card,
        );
        assert_eq!(payment.status, PaymentStatus::Pending);

        payment.complete(Some("stripe-123".into()));
        assert_eq!(payment.status, PaymentStatus::Completed);
    }
}
