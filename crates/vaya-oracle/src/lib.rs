//! vaya-oracle: Price predictions, alerts, and fare forecasting
//!
//! This crate provides intelligent pricing insights for travel bookings:
//!
//! - **Price predictions**: ML-based price forecasting with confidence levels
//! - **Price alerts**: Configurable alerts for price drops
//! - **Trend analysis**: Historical trend detection
//! - **Booking recommendations**: When to book based on predictions
//!
//! # Example Usage
//!
//! ```ignore
//! use vaya_oracle::{PricePredictor, PriceAlert, AlertManager};
//!
//! // Create a price alert
//! let alert = PriceAlert::price_below(
//!     "alert-1",
//!     "user-1",
//!     IataCode::SIN,
//!     IataCode::BKK,
//!     departure_date,
//!     MinorUnits::new(25000),
//!     CurrencyCode::SGD,
//! );
//!
//! // Check if triggered
//! let manager = AlertManager::new();
//! let result = manager.check_alert(&mut alert, current_price);
//! ```

mod alert;
mod error;
mod lstm_predictor;
mod prediction;

pub use alert::{AlertCheckResult, AlertManager, AlertStatus, AlertTrigger, PriceAlert};
pub use error::{OracleError, OracleResult};
pub use lstm_predictor::{EnsemblePredictor, LSTMConfig, LSTMPredictor, TrainingMetrics};
pub use prediction::{
    BookingRecommendation, ConfidenceLevel, PriceDataPoint, PricePrediction, PricePredictor,
    PriceTrend,
};

use time::Date;
use vaya_common::{CurrencyCode, IataCode, MinorUnits};

/// Oracle configuration
#[derive(Debug, Clone)]
pub struct OracleConfig {
    /// Minimum samples for prediction
    pub min_prediction_samples: usize,
    /// Maximum prediction days ahead
    pub max_prediction_days: u32,
    /// Minimum prediction confidence
    pub min_confidence: f64,
    /// Maximum data age for predictions (hours)
    pub max_data_age_hours: u64,
    /// Maximum alerts per user
    pub max_alerts_per_user: u32,
    /// Default alert expiry days
    pub default_alert_expiry_days: u32,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            min_prediction_samples: 10,
            max_prediction_days: 90,
            min_confidence: 0.25,
            max_data_age_hours: 72,
            max_alerts_per_user: 10,
            default_alert_expiry_days: 30,
        }
    }
}

impl OracleConfig {
    /// Create new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum samples
    pub fn with_min_samples(mut self, samples: usize) -> Self {
        self.min_prediction_samples = samples;
        self
    }

    /// Set minimum confidence
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence;
        self
    }

    /// Set max alerts per user
    pub fn with_max_alerts(mut self, max: u32) -> Self {
        self.max_alerts_per_user = max;
        self
    }
}

/// Quick price insight for a route
#[derive(Debug, Clone)]
pub struct PriceInsight {
    /// Origin
    pub origin: IataCode,
    /// Destination
    pub destination: IataCode,
    /// Current best price
    pub current_price: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Average price (30 day)
    pub avg_price_30d: MinorUnits,
    /// Lowest price (30 day)
    pub low_price_30d: MinorUnits,
    /// Highest price (30 day)
    pub high_price_30d: MinorUnits,
    /// Current trend
    pub trend: PriceTrend,
    /// Is current price good deal
    pub is_good_deal: bool,
    /// Deal score (0-100)
    pub deal_score: u8,
}

impl PriceInsight {
    /// Create insight from historical data
    pub fn from_data(
        origin: IataCode,
        destination: IataCode,
        current_price: MinorUnits,
        currency: CurrencyCode,
        historical_prices: &[MinorUnits],
    ) -> Self {
        let count = historical_prices.len();
        if count == 0 {
            return Self {
                origin,
                destination,
                current_price,
                currency,
                avg_price_30d: current_price,
                low_price_30d: current_price,
                high_price_30d: current_price,
                trend: PriceTrend::Stable,
                is_good_deal: false,
                deal_score: 50,
            };
        }

        let sum: i64 = historical_prices.iter().map(|p| p.as_i64()).sum();
        let avg = sum / count as i64;
        let low = historical_prices
            .iter()
            .map(|p| p.as_i64())
            .min()
            .unwrap_or(avg);
        let high = historical_prices
            .iter()
            .map(|p| p.as_i64())
            .max()
            .unwrap_or(avg);

        // Calculate deal score (0-100, higher = better deal)
        let current = current_price.as_i64();
        let deal_score = if high == low {
            50 // No variance
        } else {
            let range = high - low;
            let position = high - current;
            ((position as f64 / range as f64) * 100.0) as u8
        };

        let is_good_deal = deal_score >= 70;

        // Calculate trend from first half vs second half
        let mid = count / 2;
        let trend = if mid > 0 {
            let first_half_avg: i64 = historical_prices[..mid]
                .iter()
                .map(|p| p.as_i64())
                .sum::<i64>()
                / mid as i64;
            let second_half_avg: i64 = historical_prices[mid..]
                .iter()
                .map(|p| p.as_i64())
                .sum::<i64>()
                / (count - mid) as i64;
            let change_pct =
                ((second_half_avg - first_half_avg) as f64 / first_half_avg as f64) * 100.0;
            PriceTrend::from_change_percent(change_pct)
        } else {
            PriceTrend::Stable
        };

        Self {
            origin,
            destination,
            current_price,
            currency,
            avg_price_30d: MinorUnits::new(avg),
            low_price_30d: MinorUnits::new(low),
            high_price_30d: MinorUnits::new(high),
            trend,
            is_good_deal,
            deal_score,
        }
    }

    /// Get relative position text
    pub fn price_position(&self) -> &'static str {
        if self.deal_score >= 90 {
            "Excellent - lowest prices seen"
        } else if self.deal_score >= 70 {
            "Good - below average"
        } else if self.deal_score >= 50 {
            "Average"
        } else if self.deal_score >= 30 {
            "Above average"
        } else {
            "High - consider waiting"
        }
    }
}

/// Season classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Season {
    /// Peak travel season (holidays, school breaks)
    Peak,
    /// High demand season
    High,
    /// Normal season
    Normal,
    /// Low demand season
    Low,
    /// Off-peak season
    OffPeak,
}

impl Season {
    /// Get season for a date (simplified - would use actual calendar data)
    pub fn for_date(date: Date) -> Self {
        let month = date.month() as u8;

        match month {
            12 | 1 => Season::Peak,    // Year-end holidays
            6..=8 => Season::High,     // Summer vacation
            3 | 4 => Season::Normal,   // Shoulder season
            9 | 10 => Season::Normal,  // Shoulder season
            2 | 5 | 11 => Season::Low, // Low season
            _ => Season::Normal,
        }
    }

    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            Season::Peak => "PEAK",
            Season::High => "HIGH",
            Season::Normal => "NORMAL",
            Season::Low => "LOW",
            Season::OffPeak => "OFF_PEAK",
        }
    }

    /// Get typical price multiplier
    pub fn price_multiplier(&self) -> f64 {
        match self {
            Season::Peak => 1.5,
            Season::High => 1.25,
            Season::Normal => 1.0,
            Season::Low => 0.85,
            Season::OffPeak => 0.75,
        }
    }
}

/// Best time to book recommendation
#[derive(Debug, Clone)]
pub struct BestBookingTime {
    /// Departure date
    pub departure_date: Date,
    /// Recommended booking date
    pub book_by_date: Date,
    /// Days before departure
    pub days_before: u32,
    /// Confidence
    pub confidence: f64,
    /// Typical price at this time
    pub expected_price: MinorUnits,
    /// Season classification
    pub season: Season,
}

impl BestBookingTime {
    /// Calculate best booking time for a date
    pub fn calculate(
        departure_date: Date,
        base_price: MinorUnits,
        _currency: CurrencyCode,
    ) -> Self {
        let season = Season::for_date(departure_date);

        // Optimal booking windows by season
        let (days_before, confidence) = match season {
            Season::Peak => (60, 0.75),    // Book early for peak
            Season::High => (45, 0.70),    // Book fairly early
            Season::Normal => (30, 0.65),  // Standard window
            Season::Low => (21, 0.60),     // Can wait a bit
            Season::OffPeak => (14, 0.55), // Last minute ok
        };

        let book_by = departure_date - time::Duration::days(days_before as i64);
        let expected_price =
            MinorUnits::new((base_price.as_i64() as f64 * season.price_multiplier()) as i64);

        Self {
            departure_date,
            book_by_date: book_by,
            days_before,
            confidence,
            expected_price,
            season,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_config() {
        let config = OracleConfig::new()
            .with_min_samples(5)
            .with_min_confidence(0.5)
            .with_max_alerts(20);

        assert_eq!(config.min_prediction_samples, 5);
        assert_eq!(config.min_confidence, 0.5);
        assert_eq!(config.max_alerts_per_user, 20);
    }

    #[test]
    fn test_price_insight() {
        let prices: Vec<MinorUnits> = vec![
            MinorUnits::new(25000),
            MinorUnits::new(26000),
            MinorUnits::new(24000),
            MinorUnits::new(27000),
            MinorUnits::new(23000),
        ];

        let insight = PriceInsight::from_data(
            IataCode::SIN,
            IataCode::BKK,
            MinorUnits::new(23500), // Current price
            CurrencyCode::SGD,
            &prices,
        );

        assert_eq!(insight.low_price_30d.as_i64(), 23000);
        assert_eq!(insight.high_price_30d.as_i64(), 27000);
        assert!(insight.deal_score >= 50); // Should be a decent deal
    }

    #[test]
    fn test_price_insight_good_deal() {
        let prices: Vec<MinorUnits> = vec![
            MinorUnits::new(30000),
            MinorUnits::new(32000),
            MinorUnits::new(28000),
            MinorUnits::new(35000),
        ];

        let insight = PriceInsight::from_data(
            IataCode::SIN,
            IataCode::BKK,
            MinorUnits::new(28500), // Near the low
            CurrencyCode::SGD,
            &prices,
        );

        assert!(insight.deal_score >= 70);
        assert!(insight.is_good_deal);
    }

    #[test]
    fn test_season_classification() {
        let dec = Date::from_calendar_date(2025, time::Month::December, 25).unwrap();
        assert_eq!(Season::for_date(dec), Season::Peak);

        let jul = Date::from_calendar_date(2025, time::Month::July, 15).unwrap();
        assert_eq!(Season::for_date(jul), Season::High);

        let feb = Date::from_calendar_date(2025, time::Month::February, 15).unwrap();
        assert_eq!(Season::for_date(feb), Season::Low);
    }

    #[test]
    fn test_season_multiplier() {
        assert!(Season::Peak.price_multiplier() > 1.0);
        assert!(Season::Low.price_multiplier() < 1.0);
        assert_eq!(Season::Normal.price_multiplier(), 1.0);
    }

    #[test]
    fn test_best_booking_time() {
        let departure = Date::from_calendar_date(2025, time::Month::December, 25).unwrap();
        let best = BestBookingTime::calculate(departure, MinorUnits::new(25000), CurrencyCode::SGD);

        assert_eq!(best.season, Season::Peak);
        assert_eq!(best.days_before, 60);
        assert!(best.expected_price.as_i64() > 25000); // Peak = higher price
    }

    #[test]
    fn test_price_position_text() {
        let insight = PriceInsight {
            origin: IataCode::SIN,
            destination: IataCode::BKK,
            current_price: MinorUnits::new(25000),
            currency: CurrencyCode::SGD,
            avg_price_30d: MinorUnits::new(25000),
            low_price_30d: MinorUnits::new(25000),
            high_price_30d: MinorUnits::new(25000),
            trend: PriceTrend::Stable,
            is_good_deal: true,
            deal_score: 85,
        };

        assert_eq!(insight.price_position(), "Good - below average");
    }
}
