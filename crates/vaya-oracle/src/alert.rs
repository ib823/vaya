//! Price alert system

use time::{Date, OffsetDateTime};
use vaya_common::{CurrencyCode, IataCode, MinorUnits};

use crate::{OracleError, OracleResult};

/// Alert trigger type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertTrigger {
    /// Trigger when price drops below threshold
    PriceDropsBelow,
    /// Trigger when price drops by percentage
    PriceDropsBy,
    /// Trigger when any price available
    AnyPrice,
    /// Trigger on best price in time window
    BestPrice,
}

impl AlertTrigger {
    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertTrigger::PriceDropsBelow => "PRICE_DROPS_BELOW",
            AlertTrigger::PriceDropsBy => "PRICE_DROPS_BY",
            AlertTrigger::AnyPrice => "ANY_PRICE",
            AlertTrigger::BestPrice => "BEST_PRICE",
        }
    }
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertStatus {
    /// Alert is active
    Active,
    /// Alert has been triggered
    Triggered,
    /// Alert is paused
    Paused,
    /// Alert has expired
    Expired,
    /// Alert was cancelled
    Cancelled,
}

impl AlertStatus {
    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertStatus::Active => "ACTIVE",
            AlertStatus::Triggered => "TRIGGERED",
            AlertStatus::Paused => "PAUSED",
            AlertStatus::Expired => "EXPIRED",
            AlertStatus::Cancelled => "CANCELLED",
        }
    }

    /// Check if alert can receive notifications
    pub fn can_notify(&self) -> bool {
        matches!(self, AlertStatus::Active)
    }
}

/// Price alert configuration
#[derive(Debug, Clone)]
pub struct PriceAlert {
    /// Alert ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Origin airport
    pub origin: IataCode,
    /// Destination airport
    pub destination: IataCode,
    /// Departure date (or start of range)
    pub departure_date: Date,
    /// End of date range (if flexible)
    pub departure_date_end: Option<Date>,
    /// Trigger type
    pub trigger: AlertTrigger,
    /// Threshold price (for PriceDropsBelow)
    pub threshold_price: Option<MinorUnits>,
    /// Threshold percentage (for PriceDropsBy)
    pub threshold_percent: Option<u8>,
    /// Reference price (for percentage alerts)
    pub reference_price: Option<MinorUnits>,
    /// Currency
    pub currency: CurrencyCode,
    /// Current status
    pub status: AlertStatus,
    /// Creation timestamp
    pub created_at: i64,
    /// Last checked timestamp
    pub last_checked_at: Option<i64>,
    /// Triggered timestamp
    pub triggered_at: Option<i64>,
    /// Triggered price
    pub triggered_price: Option<MinorUnits>,
    /// Expiry timestamp
    pub expires_at: i64,
    /// Notification count
    pub notification_count: u32,
    /// Max notifications (0 = unlimited)
    pub max_notifications: u32,
}

impl PriceAlert {
    /// Create a price drop alert
    pub fn price_below(
        id: impl Into<String>,
        user_id: impl Into<String>,
        origin: IataCode,
        destination: IataCode,
        departure_date: Date,
        threshold: MinorUnits,
        currency: CurrencyCode,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        Self {
            id: id.into(),
            user_id: user_id.into(),
            origin,
            destination,
            departure_date,
            departure_date_end: None,
            trigger: AlertTrigger::PriceDropsBelow,
            threshold_price: Some(threshold),
            threshold_percent: None,
            reference_price: None,
            currency,
            status: AlertStatus::Active,
            created_at: now,
            last_checked_at: None,
            triggered_at: None,
            triggered_price: None,
            expires_at: now + (30 * 24 * 3600), // 30 days default expiry
            notification_count: 0,
            max_notifications: 1, // Trigger once by default
        }
    }

    /// Create a percentage drop alert
    pub fn price_drop_percent(
        id: impl Into<String>,
        user_id: impl Into<String>,
        origin: IataCode,
        destination: IataCode,
        departure_date: Date,
        reference_price: MinorUnits,
        drop_percent: u8,
        currency: CurrencyCode,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        Self {
            id: id.into(),
            user_id: user_id.into(),
            origin,
            destination,
            departure_date,
            departure_date_end: None,
            trigger: AlertTrigger::PriceDropsBy,
            threshold_price: None,
            threshold_percent: Some(drop_percent),
            reference_price: Some(reference_price),
            currency,
            status: AlertStatus::Active,
            created_at: now,
            last_checked_at: None,
            triggered_at: None,
            triggered_price: None,
            expires_at: now + (30 * 24 * 3600),
            notification_count: 0,
            max_notifications: 1,
        }
    }

    /// Create alert for any price availability
    pub fn any_price(
        id: impl Into<String>,
        user_id: impl Into<String>,
        origin: IataCode,
        destination: IataCode,
        departure_date: Date,
        currency: CurrencyCode,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();

        Self {
            id: id.into(),
            user_id: user_id.into(),
            origin,
            destination,
            departure_date,
            departure_date_end: None,
            trigger: AlertTrigger::AnyPrice,
            threshold_price: None,
            threshold_percent: None,
            reference_price: None,
            currency,
            status: AlertStatus::Active,
            created_at: now,
            last_checked_at: None,
            triggered_at: None,
            triggered_price: None,
            expires_at: now + (30 * 24 * 3600),
            notification_count: 0,
            max_notifications: 0, // Unlimited
        }
    }

    /// Set flexible date range
    pub fn with_date_range(mut self, end_date: Date) -> Self {
        self.departure_date_end = Some(end_date);
        self
    }

    /// Set expiry
    pub fn with_expiry_days(mut self, days: u32) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.expires_at = now + (days as i64 * 24 * 3600);
        self
    }

    /// Set max notifications
    pub fn with_max_notifications(mut self, max: u32) -> Self {
        self.max_notifications = max;
        self
    }

    /// Check if alert should trigger for given price
    pub fn should_trigger(&self, current_price: MinorUnits) -> bool {
        if !self.status.can_notify() {
            return false;
        }

        // Check notification limit
        if self.max_notifications > 0 && self.notification_count >= self.max_notifications {
            return false;
        }

        match self.trigger {
            AlertTrigger::PriceDropsBelow => {
                if let Some(threshold) = self.threshold_price {
                    current_price.as_i64() <= threshold.as_i64()
                } else {
                    false
                }
            }
            AlertTrigger::PriceDropsBy => {
                if let (Some(reference), Some(percent)) =
                    (self.reference_price, self.threshold_percent)
                {
                    let target = reference.as_i64() * (100 - percent as i64) / 100;
                    current_price.as_i64() <= target
                } else {
                    false
                }
            }
            AlertTrigger::AnyPrice => true,
            AlertTrigger::BestPrice => true, // Handled externally
        }
    }

    /// Mark alert as triggered
    pub fn trigger(&mut self, price: MinorUnits) -> OracleResult<()> {
        if !self.status.can_notify() {
            return Err(OracleError::AlertAlreadyTriggered);
        }

        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.triggered_at = Some(now);
        self.triggered_price = Some(price);
        self.notification_count += 1;

        // Mark as triggered if max notifications reached
        if self.max_notifications > 0 && self.notification_count >= self.max_notifications {
            self.status = AlertStatus::Triggered;
        }

        Ok(())
    }

    /// Mark last check time
    pub fn mark_checked(&mut self) {
        self.last_checked_at = Some(OffsetDateTime::now_utc().unix_timestamp());
    }

    /// Pause alert
    pub fn pause(&mut self) {
        self.status = AlertStatus::Paused;
    }

    /// Resume alert
    pub fn resume(&mut self) -> OracleResult<()> {
        if self.status == AlertStatus::Triggered || self.status == AlertStatus::Expired {
            return Err(OracleError::AlertAlreadyTriggered);
        }
        self.status = AlertStatus::Active;
        Ok(())
    }

    /// Cancel alert
    pub fn cancel(&mut self) {
        self.status = AlertStatus::Cancelled;
    }

    /// Check if alert has expired
    pub fn check_expiry(&mut self) -> bool {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        if now > self.expires_at && self.status == AlertStatus::Active {
            self.status = AlertStatus::Expired;
            true
        } else {
            false
        }
    }

    /// Check if departure date has passed
    pub fn is_past_departure(&self) -> bool {
        let today = OffsetDateTime::now_utc().date();
        self.departure_date < today
    }

    /// Get days until expiry
    pub fn days_until_expiry(&self) -> i64 {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        ((self.expires_at - now) / 86400).max(0)
    }

    /// Calculate target price for percentage drop
    pub fn target_price(&self) -> Option<MinorUnits> {
        match self.trigger {
            AlertTrigger::PriceDropsBelow => self.threshold_price,
            AlertTrigger::PriceDropsBy => {
                if let (Some(reference), Some(percent)) =
                    (self.reference_price, self.threshold_percent)
                {
                    Some(MinorUnits::new(
                        reference.as_i64() * (100 - percent as i64) / 100,
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Alert check result
#[derive(Debug, Clone)]
pub struct AlertCheckResult {
    /// Alert ID
    pub alert_id: String,
    /// Whether alert was triggered
    pub triggered: bool,
    /// Current price
    pub current_price: MinorUnits,
    /// Savings from target (if triggered)
    pub savings: Option<MinorUnits>,
    /// Check timestamp
    pub checked_at: i64,
}

/// Alert manager for handling multiple alerts
#[derive(Debug)]
pub struct AlertManager {
    /// Maximum alerts per user
    max_alerts_per_user: u32,
    /// Default expiry days
    default_expiry_days: u32,
}

impl Default for AlertManager {
    fn default() -> Self {
        Self {
            max_alerts_per_user: 10,
            default_expiry_days: 30,
        }
    }
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Set max alerts per user
    pub fn with_max_alerts(mut self, max: u32) -> Self {
        self.max_alerts_per_user = max;
        self
    }

    /// Validate alert creation
    pub fn validate_new_alert(
        &self,
        user_alert_count: u32,
        alert: &PriceAlert,
    ) -> OracleResult<()> {
        // Check user limit
        if user_alert_count >= self.max_alerts_per_user {
            return Err(OracleError::AlertLimitReached {
                current: user_alert_count,
                max: self.max_alerts_per_user,
            });
        }

        // Validate threshold for price drop alerts
        if alert.trigger == AlertTrigger::PriceDropsBelow && alert.threshold_price.is_none() {
            return Err(OracleError::InvalidThreshold(
                "Threshold price required for PRICE_DROPS_BELOW alert".into(),
            ));
        }

        if alert.trigger == AlertTrigger::PriceDropsBy {
            if alert.reference_price.is_none() {
                return Err(OracleError::InvalidThreshold(
                    "Reference price required for PRICE_DROPS_BY alert".into(),
                ));
            }
            if alert.threshold_percent.is_none() {
                return Err(OracleError::InvalidThreshold(
                    "Threshold percentage required for PRICE_DROPS_BY alert".into(),
                ));
            }
        }

        // Check departure date is in future
        let today = OffsetDateTime::now_utc().date();
        if alert.departure_date <= today {
            return Err(OracleError::InvalidConfig(
                "Departure date must be in the future".into(),
            ));
        }

        Ok(())
    }

    /// Check an alert against a price
    pub fn check_alert(&self, alert: &mut PriceAlert, price: MinorUnits) -> AlertCheckResult {
        alert.mark_checked();

        let triggered = alert.should_trigger(price);
        let savings = if triggered {
            alert
                .target_price()
                .map(|target| MinorUnits::new((target.as_i64() - price.as_i64()).max(0)))
        } else {
            None
        };

        if triggered {
            let _ = alert.trigger(price);
        }

        AlertCheckResult {
            alert_id: alert.id.clone(),
            triggered,
            current_price: price,
            savings,
            checked_at: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_alert() -> PriceAlert {
        PriceAlert::price_below(
            "alert-1",
            "user-1",
            IataCode::SIN,
            IataCode::BKK,
            Date::from_calendar_date(2026, time::Month::July, 15).unwrap(),
            MinorUnits::new(25000),
            CurrencyCode::SGD,
        )
    }

    #[test]
    fn test_alert_creation() {
        let alert = create_test_alert();

        assert_eq!(alert.id, "alert-1");
        assert_eq!(alert.trigger, AlertTrigger::PriceDropsBelow);
        assert_eq!(alert.threshold_price, Some(MinorUnits::new(25000)));
        assert_eq!(alert.status, AlertStatus::Active);
    }

    #[test]
    fn test_percent_drop_alert() {
        let alert = PriceAlert::price_drop_percent(
            "alert-2",
            "user-1",
            IataCode::SIN,
            IataCode::BKK,
            Date::from_calendar_date(2026, time::Month::July, 15).unwrap(),
            MinorUnits::new(30000), // Reference $300
            20,                     // 20% drop
            CurrencyCode::SGD,
        );

        assert_eq!(alert.trigger, AlertTrigger::PriceDropsBy);
        assert_eq!(alert.reference_price, Some(MinorUnits::new(30000)));
        assert_eq!(alert.threshold_percent, Some(20));

        // Target should be $240 (20% off $300)
        assert_eq!(alert.target_price(), Some(MinorUnits::new(24000)));
    }

    #[test]
    fn test_should_trigger_price_below() {
        let alert = create_test_alert();

        // Below threshold - should trigger
        assert!(alert.should_trigger(MinorUnits::new(20000)));

        // At threshold - should trigger
        assert!(alert.should_trigger(MinorUnits::new(25000)));

        // Above threshold - should not trigger
        assert!(!alert.should_trigger(MinorUnits::new(30000)));
    }

    #[test]
    fn test_should_trigger_percent_drop() {
        let alert = PriceAlert::price_drop_percent(
            "alert-1",
            "user-1",
            IataCode::SIN,
            IataCode::BKK,
            Date::from_calendar_date(2026, time::Month::July, 15).unwrap(),
            MinorUnits::new(30000),
            20,
            CurrencyCode::SGD,
        );

        // 25% drop ($225) - should trigger
        assert!(alert.should_trigger(MinorUnits::new(22500)));

        // 10% drop ($270) - should not trigger
        assert!(!alert.should_trigger(MinorUnits::new(27000)));
    }

    #[test]
    fn test_trigger_alert() {
        let mut alert = create_test_alert();

        assert!(alert.trigger(MinorUnits::new(20000)).is_ok());
        assert_eq!(alert.status, AlertStatus::Triggered);
        assert!(alert.triggered_at.is_some());
        assert_eq!(alert.triggered_price, Some(MinorUnits::new(20000)));

        // Cannot trigger again
        assert!(alert.trigger(MinorUnits::new(18000)).is_err());
    }

    #[test]
    fn test_pause_resume() {
        let mut alert = create_test_alert();

        alert.pause();
        assert_eq!(alert.status, AlertStatus::Paused);
        assert!(!alert.status.can_notify());

        alert.resume().unwrap();
        assert_eq!(alert.status, AlertStatus::Active);
    }

    #[test]
    fn test_multi_notification() {
        let mut alert = create_test_alert().with_max_notifications(3);

        assert!(alert.trigger(MinorUnits::new(24000)).is_ok());
        assert_eq!(alert.status, AlertStatus::Active); // Still active

        assert!(alert.trigger(MinorUnits::new(23000)).is_ok());
        assert!(alert.trigger(MinorUnits::new(22000)).is_ok());

        assert_eq!(alert.status, AlertStatus::Triggered); // Now triggered
        assert_eq!(alert.notification_count, 3);
    }

    #[test]
    fn test_alert_manager_validation() {
        let manager = AlertManager::new().with_max_alerts(5);
        let alert = create_test_alert();

        // Valid
        assert!(manager.validate_new_alert(0, &alert).is_ok());

        // At limit
        assert!(manager.validate_new_alert(5, &alert).is_err());
    }

    #[test]
    fn test_alert_check() {
        let manager = AlertManager::new();
        let mut alert = create_test_alert();

        // Check with low price (triggers)
        let result = manager.check_alert(&mut alert, MinorUnits::new(20000));
        assert!(result.triggered);
        assert!(result.savings.is_some());

        // Alert should now be triggered
        assert_eq!(alert.status, AlertStatus::Triggered);
    }

    #[test]
    fn test_days_until_expiry() {
        let alert = create_test_alert().with_expiry_days(7);
        let days = alert.days_until_expiry();
        assert!(days >= 6 && days <= 7);
    }
}
