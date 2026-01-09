//! Domain enums for VAYA
//!
//! All enums use u8 representation for compact storage.

use rkyv::{Archive, Deserialize, Serialize};
use std::fmt;

// ============================================================================
// USER DOMAIN
// ============================================================================

/// User account status
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum UserStatus {
    /// Anonymous user (not logged in)
    #[default]
    Anonymous = 0,
    /// Registered and active
    Registered = 1,
    /// Premium subscriber
    Premium = 2,
    /// Churned (was premium, cancelled)
    Churned = 3,
    /// Suspended (policy violation)
    Suspended = 4,
    /// Deleted (soft delete)
    Deleted = 5,
}

impl UserStatus {
    /// Returns the string representation of the user status.
    ///
    /// Used for serialization, API responses, and display purposes.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::Registered => "registered",
            Self::Premium => "premium",
            Self::Churned => "churned",
            Self::Suspended => "suspended",
            Self::Deleted => "deleted",
        }
    }

    /// Returns true if the user account is active and can perform actions.
    ///
    /// Active statuses: `Registered`, `Premium`
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Registered | Self::Premium)
    }

    /// Returns true if the user is allowed to make bookings.
    ///
    /// Only registered and premium users can create bookings.
    pub fn can_book(&self) -> bool {
        matches!(self, Self::Registered | Self::Premium)
    }
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// User subscription tier
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum UserTier {
    /// Free tier
    #[default]
    Free = 0,
    /// ATA Premium (RM29/month)
    Premium = 1,
    /// Enterprise/API
    Enterprise = 2,
}

impl UserTier {
    /// Returns the string representation of the user tier.
    ///
    /// Used for serialization and API responses.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Premium => "premium",
            Self::Enterprise => "enterprise",
        }
    }

    /// Daily search limit
    pub fn search_limit(&self) -> u32 {
        match self {
            Self::Free => 50,
            Self::Premium => 500,
            Self::Enterprise => 10000,
        }
    }

    /// Max active alerts
    pub fn alert_limit(&self) -> u32 {
        match self {
            Self::Free => 3,
            Self::Premium => 50,
            Self::Enterprise => 1000,
        }
    }

    /// ATA enabled
    pub fn has_ata(&self) -> bool {
        matches!(self, Self::Premium | Self::Enterprise)
    }
}

impl fmt::Display for UserTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// BOOKING DOMAIN
// ============================================================================

/// Booking status
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum BookingStatus {
    /// Payment pending
    #[default]
    Pending = 0,
    /// Payment confirmed, booking in progress
    Confirmed = 1,
    /// Ticket issued
    Ticketed = 2,
    /// Cancelled by user
    Cancelled = 3,
    /// Refunded
    Refunded = 4,
    /// Booking failed
    Failed = 5,
    /// Flight completed
    Completed = 6,
    /// No-show
    NoShow = 7,
}

impl BookingStatus {
    /// Returns the string representation of the booking status.
    ///
    /// Used for serialization, API responses, and database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Confirmed => "confirmed",
            Self::Ticketed => "ticketed",
            Self::Cancelled => "cancelled",
            Self::Refunded => "refunded",
            Self::Failed => "failed",
            Self::Completed => "completed",
            Self::NoShow => "no_show",
        }
    }

    /// Returns true if the booking has reached a final state.
    ///
    /// Terminal statuses cannot be changed and require no further action.
    /// Terminal: `Cancelled`, `Refunded`, `Failed`, `Completed`, `NoShow`
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Cancelled | Self::Refunded | Self::Failed | Self::Completed | Self::NoShow
        )
    }

    /// Returns true if the booking is still in progress.
    ///
    /// Active bookings require monitoring and may need user action.
    /// Active: `Pending`, `Confirmed`, `Ticketed`
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Pending | Self::Confirmed | Self::Ticketed)
    }

    /// Returns true if the booking can be cancelled by the user.
    ///
    /// Cancellation is allowed before the flight departs.
    /// Cancellable: `Pending`, `Confirmed`, `Ticketed`
    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Pending | Self::Confirmed | Self::Ticketed)
    }
}

impl fmt::Display for BookingStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Trip type
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum TripType {
    /// One-way trip
    #[default]
    OneWay = 0,
    /// Round trip
    RoundTrip = 1,
    /// Multi-city
    MultiCity = 2,
}

impl TripType {
    /// Returns the string representation of the trip type.
    ///
    /// Used for API requests and search queries.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneWay => "one_way",
            Self::RoundTrip => "round_trip",
            Self::MultiCity => "multi_city",
        }
    }
}

impl fmt::Display for TripType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Cabin class
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum CabinClass {
    /// Economy class
    #[default]
    Economy = 0,
    /// Premium economy
    PremiumEconomy = 1,
    /// Business class
    Business = 2,
    /// First class
    First = 3,
}

impl CabinClass {
    /// Returns the string representation of the cabin class.
    ///
    /// Used for API requests and display purposes.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Economy => "economy",
            Self::PremiumEconomy => "premium_economy",
            Self::Business => "business",
            Self::First => "first",
        }
    }

    /// Returns the IATA cabin class code.
    ///
    /// Standard codes: Y (Economy), W (Premium Economy), C (Business), F (First)
    pub fn code(&self) -> char {
        match self {
            Self::Economy => 'Y',
            Self::PremiumEconomy => 'W',
            Self::Business => 'C',
            Self::First => 'F',
        }
    }
}

impl fmt::Display for CabinClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Traveler type (for fare calculation)
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum TravelerType {
    /// Adult (12+)
    #[default]
    Adult = 0,
    /// Child (2-11)
    Child = 1,
    /// Infant (0-1)
    Infant = 2,
}

impl TravelerType {
    /// Returns the string representation of the traveler type.
    ///
    /// Used for API requests and fare calculations.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Adult => "adult",
            Self::Child => "child",
            Self::Infant => "infant",
        }
    }

    /// Returns the PTC (Passenger Type Code) for fare calculation.
    ///
    /// Standard codes: ADT (Adult), CHD (Child), INF (Infant)
    pub fn code(&self) -> &'static str {
        match self {
            Self::Adult => "ADT",
            Self::Child => "CHD",
            Self::Infant => "INF",
        }
    }
}

impl fmt::Display for TravelerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// POOL DOMAIN
// ============================================================================

/// Pool status
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum PoolStatus {
    /// Pool is forming, collecting members
    #[default]
    Forming = 0,
    /// Pool is active, threshold reached
    Active = 1,
    /// Bidding is closed, evaluating bids
    BiddingClosed = 2,
    /// Booking in progress
    Booking = 3,
    /// Pool completed successfully
    Completed = 4,
    /// Pool expired (didn't reach threshold)
    Expired = 5,
    /// No bids received
    NoBids = 6,
    /// Pool cancelled by admin
    Cancelled = 7,
}

impl PoolStatus {
    /// Returns the string representation of the pool status.
    ///
    /// Used for API responses and database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Forming => "forming",
            Self::Active => "active",
            Self::BiddingClosed => "bidding_closed",
            Self::Booking => "booking",
            Self::Completed => "completed",
            Self::Expired => "expired",
            Self::NoBids => "no_bids",
            Self::Cancelled => "cancelled",
        }
    }

    /// Returns true if new members can join the pool.
    ///
    /// Joinable statuses: `Forming`, `Active`
    pub fn is_joinable(&self) -> bool {
        matches!(self, Self::Forming | Self::Active)
    }

    /// Returns true if the pool has reached a final state.
    ///
    /// Terminal pools cannot accept new members or process bids.
    /// Terminal: `Completed`, `Expired`, `NoBids`, `Cancelled`
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Expired | Self::NoBids | Self::Cancelled
        )
    }
}

impl fmt::Display for PoolStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// PAYMENT DOMAIN
// ============================================================================

/// Payment status
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum PaymentStatus {
    /// Payment initiated
    #[default]
    Pending = 0,
    /// Payment processing
    Processing = 1,
    /// Requires user action (3DS, OTP)
    RequiresAction = 2,
    /// Payment completed successfully
    Completed = 3,
    /// Payment failed
    Failed = 4,
    /// Payment refunded
    Refunded = 5,
    /// Partially refunded
    PartiallyRefunded = 6,
    /// Disputed/chargeback
    Disputed = 7,
}

impl PaymentStatus {
    /// Returns the string representation of the payment status.
    ///
    /// Used for API responses, webhooks, and database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::RequiresAction => "requires_action",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Refunded => "refunded",
            Self::PartiallyRefunded => "partially_refunded",
            Self::Disputed => "disputed",
        }
    }

    /// Returns true if the payment was completed successfully.
    ///
    /// Only `Completed` status indicates successful payment capture.
    pub fn is_successful(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Returns true if the payment has reached a final state.
    ///
    /// Terminal payments require no further processing.
    /// Terminal: `Completed`, `Failed`, `Refunded`, `PartiallyRefunded`, `Disputed`
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed
                | Self::Failed
                | Self::Refunded
                | Self::PartiallyRefunded
                | Self::Disputed
        )
    }
}

impl fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Payment method
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum PaymentMethod {
    /// Credit/Debit card
    #[default]
    Card = 0,
    /// FPX (Malaysian bank transfer)
    Fpx = 1,
    /// GrabPay
    GrabPay = 2,
    /// Touch 'n Go eWallet
    TouchNGo = 3,
    /// Boost
    Boost = 4,
    /// ShopeePay
    ShopeePay = 5,
}

impl PaymentMethod {
    /// Returns the string representation of the payment method.
    ///
    /// Used for API requests and database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Card => "card",
            Self::Fpx => "fpx",
            Self::GrabPay => "grabpay",
            Self::TouchNGo => "touchngo",
            Self::Boost => "boost",
            Self::ShopeePay => "shopeepay",
        }
    }

    /// Returns the human-readable display name of the payment method.
    ///
    /// Used for UI display to users selecting a payment method.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Card => "Credit/Debit Card",
            Self::Fpx => "FPX Online Banking",
            Self::GrabPay => "GrabPay",
            Self::TouchNGo => "Touch 'n Go eWallet",
            Self::Boost => "Boost",
            Self::ShopeePay => "ShopeePay",
        }
    }
}

impl fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// ALERT DOMAIN
// ============================================================================

/// Alert status
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum AlertStatus {
    /// Alert is active, monitoring prices
    #[default]
    Active = 0,
    /// Alert was triggered (target price reached)
    Triggered = 1,
    /// Alert is paused by user
    Paused = 2,
    /// Alert expired (date passed)
    Expired = 3,
    /// Alert deleted by user
    Deleted = 4,
}

impl AlertStatus {
    /// Returns the string representation of the alert status.
    ///
    /// Used for API responses and database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Triggered => "triggered",
            Self::Paused => "paused",
            Self::Expired => "expired",
            Self::Deleted => "deleted",
        }
    }

    /// Returns true if the alert is actively monitoring prices.
    ///
    /// Only `Active` alerts are checked against new price data.
    pub fn is_monitoring(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl fmt::Display for AlertStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Notification channel flags
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(transparent)]
pub struct NotificationChannels(u8);

impl NotificationChannels {
    /// No notification channels enabled.
    pub const NONE: Self = Self(0);
    /// Email channel bit flag.
    pub const EMAIL: u8 = 1;
    /// Push notification channel bit flag.
    pub const PUSH: u8 = 2;
    /// SMS channel bit flag.
    pub const SMS: u8 = 4;

    /// Creates a new NotificationChannels with no channels enabled.
    pub fn new() -> Self {
        Self(0)
    }

    /// Enables email notifications and returns self for chaining.
    pub fn with_email(mut self) -> Self {
        self.0 |= Self::EMAIL;
        self
    }

    /// Enables push notifications and returns self for chaining.
    pub fn with_push(mut self) -> Self {
        self.0 |= Self::PUSH;
        self
    }

    /// Enables SMS notifications and returns self for chaining.
    pub fn with_sms(mut self) -> Self {
        self.0 |= Self::SMS;
        self
    }

    /// Returns true if email notifications are enabled.
    pub fn has_email(&self) -> bool {
        self.0 & Self::EMAIL != 0
    }

    /// Returns true if push notifications are enabled.
    pub fn has_push(&self) -> bool {
        self.0 & Self::PUSH != 0
    }

    /// Returns true if SMS notifications are enabled.
    pub fn has_sms(&self) -> bool {
        self.0 & Self::SMS != 0
    }

    /// Returns the raw u8 value of the channel flags.
    ///
    /// Used for serialization and database storage.
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

// ============================================================================
// ORACLE/ML DOMAIN
// ============================================================================

/// Oracle recommendation
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum OracleRecommendation {
    /// Buy now - price unlikely to go lower
    BuyNow = 0,
    /// Wait - price may drop
    #[default]
    Wait = 1,
    /// Watch - price is volatile, monitor closely
    Watch = 2,
}

impl OracleRecommendation {
    /// Returns the string representation of the recommendation.
    ///
    /// Used for API responses and analytics.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BuyNow => "buy_now",
            Self::Wait => "wait",
            Self::Watch => "watch",
        }
    }

    /// Returns the user-facing display message for the recommendation.
    ///
    /// Used in UI to communicate the Oracle's advice to users.
    pub fn display_message(&self) -> &'static str {
        match self {
            Self::BuyNow => "Buy Now - Lowest Price Expected",
            Self::Wait => "Wait - Price May Drop",
            Self::Watch => "Watch - Price is Volatile",
        }
    }
}

impl fmt::Display for OracleRecommendation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_message())
    }
}

/// Offer/price source
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum OfferSource {
    /// Kiwi.com API
    #[default]
    Kiwi = 0,
    /// Travelpayouts API
    Travelpayouts = 1,
    /// Amadeus API
    Amadeus = 2,
    /// Duffel API
    Duffel = 3,
    /// Direct airline integration
    Direct = 4,
}

impl OfferSource {
    /// Returns the string representation of the offer source.
    ///
    /// Used for API responses and analytics tracking.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Kiwi => "kiwi",
            Self::Travelpayouts => "travelpayouts",
            Self::Amadeus => "amadeus",
            Self::Duffel => "duffel",
            Self::Direct => "direct",
        }
    }

    /// Returns the human-readable display name of the source.
    ///
    /// Used in UI to show where a price quote originated.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Kiwi => "Kiwi.com",
            Self::Travelpayouts => "Travelpayouts",
            Self::Amadeus => "Amadeus",
            Self::Duffel => "Duffel",
            Self::Direct => "Direct",
        }
    }

    /// Returns true if offers from this source can be booked directly.
    ///
    /// Bookable sources: `Kiwi`, `Duffel`, `Direct`
    pub fn is_bookable(&self) -> bool {
        matches!(self, Self::Kiwi | Self::Duffel | Self::Direct)
    }
}

impl fmt::Display for OfferSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// AUTH DOMAIN
// ============================================================================

/// OAuth provider
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum OAuthProvider {
    /// Google OAuth
    Google = 0,
    /// Apple Sign-In
    Apple = 1,
    /// Facebook Login
    Facebook = 2,
}

impl OAuthProvider {
    /// Returns the string representation of the OAuth provider.
    ///
    /// Used for API requests and database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Apple => "apple",
            Self::Facebook => "facebook",
        }
    }
}

impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Gender (for passport/travel documents)
#[derive(Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[archive(compare(PartialEq))]
#[archive_attr(derive(Debug, PartialEq, Eq, Hash))]
#[repr(u8)]
pub enum Gender {
    /// Gender not specified or unknown
    #[default]
    Unknown = 0,
    /// Male
    Male = 1,
    /// Female
    Female = 2,
    /// Other or non-binary
    Other = 3,
}

impl Gender {
    /// Returns the string representation of the gender.
    ///
    /// Used for API responses and user profiles.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Male => "male",
            Self::Female => "female",
            Self::Other => "other",
        }
    }

    /// Returns the single-character code for travel documents.
    ///
    /// Standard codes: M (Male), F (Female), X (Other/Unspecified), U (Unknown)
    pub fn code(&self) -> char {
        match self {
            Self::Unknown => 'U',
            Self::Male => 'M',
            Self::Female => 'F',
            Self::Other => 'X',
        }
    }
}

impl fmt::Display for Gender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_status() {
        assert!(UserStatus::Registered.is_active());
        assert!(UserStatus::Premium.can_book());
        assert!(!UserStatus::Suspended.is_active());
    }

    #[test]
    fn test_user_tier() {
        assert_eq!(UserTier::Free.search_limit(), 50);
        assert_eq!(UserTier::Premium.alert_limit(), 50);
        assert!(UserTier::Premium.has_ata());
        assert!(!UserTier::Free.has_ata());
    }

    #[test]
    fn test_booking_status() {
        assert!(BookingStatus::Completed.is_terminal());
        assert!(BookingStatus::Ticketed.can_cancel());
        assert!(!BookingStatus::Failed.is_active());
    }

    #[test]
    fn test_pool_status() {
        assert!(PoolStatus::Forming.is_joinable());
        assert!(PoolStatus::Completed.is_terminal());
    }

    #[test]
    fn test_notification_channels() {
        let channels = NotificationChannels::new().with_email().with_push();

        assert!(channels.has_email());
        assert!(channels.has_push());
        assert!(!channels.has_sms());
    }
}
