//! VAYA UI Types - Data Transfer Objects for API communication
//!
//! These types mirror the core types in `vaya-common` but are designed for
//! frontend use with serde JSON serialization instead of rkyv zero-copy.
//!
//! This separation is necessary because `vaya-common` uses `ring` for UUID
//! generation, which doesn't compile to WASM.

use serde::{Deserialize, Serialize};

// ============================================================================
// PRIMITIVE TYPES
// ============================================================================

/// Airport/City Code (3-letter IATA)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IataCode(pub String);

impl IataCode {
    pub fn new(code: &str) -> Self {
        Self(code.to_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_valid(&self) -> bool {
        self.0.len() == 3 && self.0.chars().all(|c| c.is_ascii_uppercase())
    }
}

impl std::fmt::Display for IataCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Currency Code (3-letter ISO 4217)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyCode(pub String);

impl CurrencyCode {
    pub fn new(code: &str) -> Self {
        Self(code.to_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get decimal places for this currency
    pub fn decimals(&self) -> u8 {
        match self.0.as_str() {
            "JPY" | "KRW" | "VND" => 0,
            "BHD" | "KWD" | "OMR" => 3,
            _ => 2,
        }
    }

    // Common currencies
    pub const MYR: &'static str = "MYR";
    pub const USD: &'static str = "USD";
    pub const SGD: &'static str = "SGD";
}

impl std::fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Price with currency
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Price {
    /// Amount in minor units (cents/sen)
    pub amount: i64,
    /// Currency code
    pub currency: String,
}

impl Price {
    pub fn new(amount: i64, currency: &str) -> Self {
        Self {
            amount,
            currency: currency.to_string(),
        }
    }

    /// Create price in MYR (sen)
    pub fn myr(sen: i64) -> Self {
        Self::new(sen, "MYR")
    }

    /// Get display amount (major units)
    pub fn display_amount(&self) -> f64 {
        let decimals = match self.currency.as_str() {
            "JPY" | "KRW" | "VND" => 0,
            "BHD" | "KWD" | "OMR" => 3,
            _ => 2,
        };
        self.amount as f64 / 10f64.powi(decimals)
    }

    /// Format for display (e.g., "RM 150.00")
    pub fn format(&self) -> String {
        let symbol = match self.currency.as_str() {
            "MYR" => "RM",
            "USD" => "$",
            "SGD" => "S$",
            "EUR" => "€",
            "GBP" => "£",
            _ => &self.currency,
        };
        format!("{} {:.2}", symbol, self.display_amount())
    }
}

impl std::fmt::Display for Price {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// Route (origin -> destination)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub origin: String,
    pub destination: String,
}

impl Route {
    pub fn new(origin: &str, destination: &str) -> Self {
        Self {
            origin: origin.to_uppercase(),
            destination: destination.to_uppercase(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.origin.len() == 3
            && self.destination.len() == 3
            && self.origin != self.destination
            && self.origin.chars().all(|c| c.is_ascii_uppercase())
            && self.destination.chars().all(|c| c.is_ascii_uppercase())
    }

    /// Format as "KUL-NRT"
    pub fn to_string_compact(&self) -> String {
        format!("{}-{}", self.origin, self.destination)
    }
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} → {}", self.origin, self.destination)
    }
}

/// Date (YYYY-MM-DD)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    /// Parse from ISO 8601 date string (YYYY-MM-DD)
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(Self {
            year: parts[0].parse().ok()?,
            month: parts[1].parse().ok()?,
            day: parts[2].parse().ok()?,
        })
    }

    pub fn is_valid(&self) -> bool {
        if self.month < 1 || self.month > 12 || self.day < 1 {
            return false;
        }
        let days_in_month = match self.month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => {
                if (self.year % 4 == 0 && self.year % 100 != 0) || (self.year % 400 == 0) {
                    29
                } else {
                    28
                }
            }
            _ => return false,
        };
        self.day <= days_in_month
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

// ============================================================================
// API TYPES
// ============================================================================

/// Flight search request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub origin: String,
    pub destination: String,
    pub departure_date: String,
    pub return_date: Option<String>,
    pub passengers: u8,
    pub cabin_class: String,
}

/// Flight result from search
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Flight {
    pub id: String,
    pub airline: String,
    pub airline_name: String,
    pub flight_number: String,
    pub origin: String,
    pub destination: String,
    pub departure_time: String,
    pub arrival_time: String,
    pub duration_minutes: u32,
    pub price: Price,
    pub cabin_class: String,
    pub stops: u8,
}

/// Oracle prediction verdict
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleVerdict {
    BookNow,
    Wait,
    JoinPool,
    Uncertain,
}

impl OracleVerdict {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::BookNow => "BOOK NOW",
            Self::JoinPool => "JOIN POOL",
            Self::Uncertain => "EXPLORE OPTIONS",
            Self::Wait => "WAIT",
        }
    }

    pub fn display_text_with_days(&self, days: Option<u32>) -> String {
        match self {
            Self::Wait => {
                if let Some(d) = days {
                    format!("WAIT {} DAYS", d)
                } else {
                    "WAIT".to_string()
                }
            }
            _ => self.display_text().to_string(),
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::BookNow => "verdict-book",
            Self::Wait => "verdict-wait",
            Self::JoinPool => "verdict-pool",
            Self::Uncertain => "verdict-uncertain",
        }
    }

    pub fn cta_text(&self) -> &'static str {
        match self {
            Self::BookNow => "Book This Flight",
            Self::Wait => "Set Price Alert",
            Self::JoinPool => "Join Demand Pool",
            Self::Uncertain => "See All Options",
        }
    }
}

/// Oracle prediction result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OraclePrediction {
    pub id: String,
    pub verdict: OracleVerdict,
    pub confidence: u8,
    pub current_price: Price,
    pub predicted_price: Option<Price>,
    pub wait_days: Option<u32>,
    pub price_trend: Option<PriceTrend>,
    pub reasoning: Vec<String>,
}

/// Price trend direction
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceTrend {
    Rising,
    Falling,
    Stable,
    Volatile,
}

impl PriceTrend {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Rising => "Prices rising",
            Self::Falling => "Prices falling",
            Self::Stable => "Prices stable",
            Self::Volatile => "Prices volatile",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Rising => "↗",
            Self::Falling => "↘",
            Self::Stable => "→",
            Self::Volatile => "↕",
        }
    }
}

/// Confidence level label
pub fn confidence_label(confidence: u8) -> &'static str {
    match confidence {
        90..=100 => "Very High",
        75..=89 => "High",
        60..=74 => "Moderate",
        _ => "Limited Data",
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// API error response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

// ============================================================================
// BOOKING TYPES
// ============================================================================

/// Passenger information
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Passenger {
    pub id: String,
    pub passenger_type: PassengerType,
    pub title: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: Option<String>,
    pub nationality: Option<String>,
    pub passport_number: Option<String>,
    pub passport_expiry: Option<String>,
}

/// Passenger type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassengerType {
    #[default]
    Adult,
    Child,
    Infant,
}

impl PassengerType {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Adult => "Adult",
            Self::Child => "Child (2-11)",
            Self::Infant => "Infant (0-2)",
        }
    }
}

/// Booking extras/add-ons
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BookingExtras {
    pub checked_bags: u8,
    pub seat_selection: Option<String>,
    pub meal: Option<String>,
    pub insurance: Option<InsuranceType>,
}

/// Insurance type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsuranceType {
    Basic,
    Standard,
    Premium,
}

impl InsuranceType {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Basic => "Basic Coverage",
            Self::Standard => "Standard Coverage",
            Self::Premium => "Premium Coverage",
        }
    }

    pub fn price_myr(&self) -> i64 {
        match self {
            Self::Basic => 2500,    // RM 25
            Self::Standard => 4500, // RM 45
            Self::Premium => 8500,  // RM 85
        }
    }
}

/// Price breakdown data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PriceBreakdownData {
    pub base_fare: i64,
    pub taxes: i64,
    pub fees: i64,
    pub extras: i64,
    pub discount: i64,
}

impl PriceBreakdownData {
    pub fn total(&self) -> i64 {
        self.base_fare + self.taxes + self.fees + self.extras - self.discount
    }
}

/// Full booking
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Booking {
    pub id: String,
    pub reference: String,
    pub flight: Option<Flight>,
    pub passengers: Vec<Passenger>,
    pub extras: BookingExtras,
    pub contact_email: String,
    pub contact_phone: String,
    pub price_breakdown: PriceBreakdownData,
    pub total: i64,
    pub status: BookingStatus,
}

/// Booking status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookingStatus {
    #[default]
    Draft,
    PriceLocked,
    Confirmed,
    Ticketed,
    Cancelled,
}

impl BookingStatus {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Draft => "Draft",
            Self::PriceLocked => "Price Locked",
            Self::Confirmed => "Confirmed",
            Self::Ticketed => "Ticketed",
            Self::Cancelled => "Cancelled",
        }
    }
}

// ============================================================================
// PAYMENT TYPES
// ============================================================================

/// Payment method
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Card,
    Fpx,
    GrabPay,
    TouchNGo,
    Boost,
    ShopeePay,
}

impl PaymentMethod {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Card => "Credit/Debit Card",
            Self::Fpx => "FPX Online Banking",
            Self::GrabPay => "GrabPay",
            Self::TouchNGo => "Touch 'n Go eWallet",
            Self::Boost => "Boost",
            Self::ShopeePay => "ShopeePay",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Card => "💳",
            Self::Fpx => "🏦",
            Self::GrabPay => "🟢",
            Self::TouchNGo => "🔵",
            Self::Boost => "🟠",
            Self::ShopeePay => "🟠",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Card,
            Self::Fpx,
            Self::GrabPay,
            Self::TouchNGo,
            Self::Boost,
            Self::ShopeePay,
        ]
    }
}

/// FPX Bank
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FpxBank {
    pub code: String,
    pub name: String,
    pub online: bool,
}

impl FpxBank {
    pub fn mock_banks() -> Vec<Self> {
        vec![
            FpxBank {
                code: "MBB".to_string(),
                name: "Maybank".to_string(),
                online: true,
            },
            FpxBank {
                code: "CIMB".to_string(),
                name: "CIMB Bank".to_string(),
                online: true,
            },
            FpxBank {
                code: "PBB".to_string(),
                name: "Public Bank".to_string(),
                online: true,
            },
            FpxBank {
                code: "RHB".to_string(),
                name: "RHB Bank".to_string(),
                online: true,
            },
            FpxBank {
                code: "HLB".to_string(),
                name: "Hong Leong Bank".to_string(),
                online: true,
            },
            FpxBank {
                code: "AMBANK".to_string(),
                name: "AmBank".to_string(),
                online: true,
            },
            FpxBank {
                code: "BIMB".to_string(),
                name: "Bank Islam".to_string(),
                online: true,
            },
            FpxBank {
                code: "BSN".to_string(),
                name: "BSN".to_string(),
                online: false,
            },
            FpxBank {
                code: "OCBC".to_string(),
                name: "OCBC Bank".to_string(),
                online: true,
            },
            FpxBank {
                code: "SCB".to_string(),
                name: "Standard Chartered".to_string(),
                online: true,
            },
        ]
    }
}

/// Payment status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Processing,
    RequiresAction, // 3DS
    Succeeded,
    Failed,
}

/// Payment initiation response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentInitiation {
    pub payment_id: String,
    pub status: PaymentStatus,
    pub redirect_url: Option<String>,
    pub three_ds_url: Option<String>,
}

/// Payment result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentResult {
    pub status: PaymentStatus,
    pub error: Option<PaymentError>,
    pub booking_reference: Option<String>,
    pub receipt_url: Option<String>,
}

/// Payment error codes (18 variants)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentError {
    CardDeclined,
    InsufficientFunds,
    ExpiredCard,
    InvalidCard,
    FraudSuspected,
    BankUnavailable,
    NetworkError,
    ThreeDsFailed,
    CurrencyNotSupported,
    LimitExceeded,
    CardNotSupported,
    ProcessorError,
    TimeoutError,
    DuplicateTransaction,
    InvalidCvv,
    AddressVerificationFailed,
    RiskBlocked,
    GeneralError,
}

impl PaymentError {
    pub fn display_message(&self) -> &'static str {
        match self {
            Self::CardDeclined => "Your card was declined. Please try another payment method.",
            Self::InsufficientFunds => {
                "Insufficient funds. Please check your balance or try another card."
            }
            Self::ExpiredCard => "Your card has expired. Please use a valid card.",
            Self::InvalidCard => "Invalid card details. Please check and try again.",
            Self::FraudSuspected => "Transaction flagged for security. Please contact your bank.",
            Self::BankUnavailable => "Bank is temporarily unavailable. Please try again later.",
            Self::NetworkError => "Network error. Please check your connection and try again.",
            Self::ThreeDsFailed => "3D Secure verification failed. Please try again.",
            Self::CurrencyNotSupported => {
                "Currency not supported. Please try another payment method."
            }
            Self::LimitExceeded => "Transaction limit exceeded. Please contact your bank.",
            Self::CardNotSupported => "Card type not supported. Please try another card.",
            Self::ProcessorError => "Payment processor error. Please try again.",
            Self::TimeoutError => "Request timed out. Please try again.",
            Self::DuplicateTransaction => {
                "Duplicate transaction detected. Please wait or check your email."
            }
            Self::InvalidCvv => "Invalid CVV. Please check the security code on your card.",
            Self::AddressVerificationFailed => {
                "Address verification failed. Please check your billing address."
            }
            Self::RiskBlocked => "Transaction blocked by security. Please contact support.",
            Self::GeneralError => "Something went wrong. Please try again.",
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::BankUnavailable | Self::NetworkError | Self::ProcessorError | Self::TimeoutError
        )
    }
}

// ============================================================================
// PRICE LOCK TYPES
// ============================================================================

/// Price lock duration options
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceLockDuration {
    Hours24,
    Hours48,
    Hours72,
}

impl PriceLockDuration {
    pub fn display_text(&self) -> &'static str {
        match self {
            Self::Hours24 => "24 Hours",
            Self::Hours48 => "48 Hours",
            Self::Hours72 => "72 Hours",
        }
    }

    pub fn hours(&self) -> u32 {
        match self {
            Self::Hours24 => 24,
            Self::Hours48 => 48,
            Self::Hours72 => 72,
        }
    }

    pub fn fee_myr(&self) -> i64 {
        match self {
            Self::Hours24 => 1500, // RM 15
            Self::Hours48 => 2500, // RM 25
            Self::Hours72 => 3500, // RM 35
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Hours24, Self::Hours48, Self::Hours72]
    }
}

/// Price lock
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceLock {
    pub id: String,
    pub flight_id: String,
    pub locked_price: i64,
    pub duration: PriceLockDuration,
    pub expires_at: String,
    pub fee: i64,
}
