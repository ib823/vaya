//! Price prediction engine

use time::{Date, OffsetDateTime};
use vaya_common::{CurrencyCode, IataCode, MinorUnits};

use crate::{OracleError, OracleResult};

/// Price confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    /// Very high confidence (>90%)
    VeryHigh,
    /// High confidence (75-90%)
    High,
    /// Medium confidence (50-75%)
    Medium,
    /// Low confidence (25-50%)
    Low,
    /// Very low confidence (<25%)
    VeryLow,
}

impl ConfidenceLevel {
    /// Get confidence level from percentage
    pub fn from_confidence(confidence: f64) -> Self {
        if confidence >= 0.90 {
            ConfidenceLevel::VeryHigh
        } else if confidence >= 0.75 {
            ConfidenceLevel::High
        } else if confidence >= 0.50 {
            ConfidenceLevel::Medium
        } else if confidence >= 0.25 {
            ConfidenceLevel::Low
        } else {
            ConfidenceLevel::VeryLow
        }
    }

    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfidenceLevel::VeryHigh => "VERY_HIGH",
            ConfidenceLevel::High => "HIGH",
            ConfidenceLevel::Medium => "MEDIUM",
            ConfidenceLevel::Low => "LOW",
            ConfidenceLevel::VeryLow => "VERY_LOW",
        }
    }
}

/// Price trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceTrend {
    /// Prices expected to increase significantly
    StrongUp,
    /// Prices expected to increase
    Up,
    /// Prices expected to remain stable
    Stable,
    /// Prices expected to decrease
    Down,
    /// Prices expected to decrease significantly
    StrongDown,
}

impl PriceTrend {
    /// Get trend from price change percentage
    pub fn from_change_percent(change: f64) -> Self {
        if change > 10.0 {
            PriceTrend::StrongUp
        } else if change > 3.0 {
            PriceTrend::Up
        } else if change > -3.0 {
            PriceTrend::Stable
        } else if change > -10.0 {
            PriceTrend::Down
        } else {
            PriceTrend::StrongDown
        }
    }

    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            PriceTrend::StrongUp => "STRONG_UP",
            PriceTrend::Up => "UP",
            PriceTrend::Stable => "STABLE",
            PriceTrend::Down => "DOWN",
            PriceTrend::StrongDown => "STRONG_DOWN",
        }
    }

    /// Check if this is a favorable trend for buying
    pub fn is_favorable(&self) -> bool {
        matches!(
            self,
            PriceTrend::Down | PriceTrend::StrongDown | PriceTrend::Stable
        )
    }
}

/// Booking recommendation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookingRecommendation {
    /// Book now - price likely to increase
    BookNow,
    /// Book soon - within a few days
    BookSoon,
    /// Wait for better price
    Wait,
    /// Monitor prices closely
    Monitor,
}

impl BookingRecommendation {
    /// Get display string
    pub fn as_str(&self) -> &'static str {
        match self {
            BookingRecommendation::BookNow => "BOOK_NOW",
            BookingRecommendation::BookSoon => "BOOK_SOON",
            BookingRecommendation::Wait => "WAIT",
            BookingRecommendation::Monitor => "MONITOR",
        }
    }
}

/// Price prediction result
#[derive(Debug, Clone)]
pub struct PricePrediction {
    /// Route origin
    pub origin: IataCode,
    /// Route destination
    pub destination: IataCode,
    /// Departure date
    pub departure_date: Date,
    /// Predicted price
    pub predicted_price: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Confidence score (0-1)
    pub confidence: f64,
    /// Confidence level
    pub confidence_level: ConfidenceLevel,
    /// Price range - lower bound
    pub price_low: MinorUnits,
    /// Price range - upper bound
    pub price_high: MinorUnits,
    /// Expected trend
    pub trend: PriceTrend,
    /// Expected change percentage
    pub expected_change_percent: f64,
    /// Booking recommendation
    pub recommendation: BookingRecommendation,
    /// Days until departure
    pub days_until_departure: u32,
    /// Prediction timestamp
    pub predicted_at: i64,
    /// Model version used
    pub model_version: String,
}

impl PricePrediction {
    /// Create a new prediction
    pub fn new(
        origin: IataCode,
        destination: IataCode,
        departure_date: Date,
        predicted_price: MinorUnits,
        currency: CurrencyCode,
        confidence: f64,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        let today = now.date();
        let days_until = (departure_date - today).whole_days().max(0) as u32;

        // Calculate price range based on confidence
        let uncertainty = 1.0 - confidence;
        let base = predicted_price.as_i64() as f64;
        let range = base * uncertainty * 0.3; // 30% max range at 0 confidence

        Self {
            origin,
            destination,
            departure_date,
            predicted_price,
            currency,
            confidence,
            confidence_level: ConfidenceLevel::from_confidence(confidence),
            price_low: MinorUnits::new((base - range).max(0.0) as i64),
            price_high: MinorUnits::new((base + range) as i64),
            trend: PriceTrend::Stable,
            expected_change_percent: 0.0,
            recommendation: BookingRecommendation::Monitor,
            days_until_departure: days_until,
            predicted_at: now.unix_timestamp(),
            model_version: "1.0.0".into(),
        }
    }

    /// Set trend information
    pub fn with_trend(mut self, trend: PriceTrend, change_percent: f64) -> Self {
        self.trend = trend;
        self.expected_change_percent = change_percent;
        self
    }

    /// Set recommendation
    pub fn with_recommendation(mut self, rec: BookingRecommendation) -> Self {
        self.recommendation = rec;
        self
    }

    /// Calculate recommendation based on trend and days until departure
    pub fn calculate_recommendation(&mut self) {
        self.recommendation = if self.days_until_departure <= 3 {
            // Very close to departure
            BookingRecommendation::BookNow
        } else if self.trend == PriceTrend::StrongUp {
            BookingRecommendation::BookNow
        } else if self.trend == PriceTrend::Up {
            BookingRecommendation::BookSoon
        } else if self.trend == PriceTrend::StrongDown && self.days_until_departure > 7 {
            BookingRecommendation::Wait
        } else if self.trend == PriceTrend::Down && self.days_until_departure > 14 {
            BookingRecommendation::Wait
        } else {
            BookingRecommendation::Monitor
        };
    }

    /// Check if prediction is still valid (not too old)
    pub fn is_valid(&self) -> bool {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let age_hours = (now - self.predicted_at) / 3600;
        age_hours < 24 // Valid for 24 hours
    }
}

/// Historical price data point
#[derive(Debug, Clone)]
pub struct PriceDataPoint {
    /// Price
    pub price: MinorUnits,
    /// Currency
    pub currency: CurrencyCode,
    /// Observation timestamp
    pub timestamp: i64,
    /// Days before departure when observed
    pub days_before_departure: u32,
    /// Day of week (0 = Sunday)
    pub day_of_week: u8,
    /// Is weekend departure
    pub is_weekend_departure: bool,
    /// Is holiday period
    pub is_holiday: bool,
}

impl PriceDataPoint {
    /// Convert to feature vector for ML
    pub fn to_features(&self) -> Vec<f64> {
        vec![
            self.price.as_i64() as f64,
            self.days_before_departure as f64,
            self.day_of_week as f64,
            if self.is_weekend_departure { 1.0 } else { 0.0 },
            if self.is_holiday { 1.0 } else { 0.0 },
        ]
    }
}

/// Price predictor using historical data
#[derive(Debug)]
pub struct PricePredictor {
    /// Minimum samples required for prediction
    min_samples: usize,
    /// Maximum prediction days ahead
    max_prediction_days: u32,
    /// Minimum confidence threshold
    min_confidence: f64,
    /// Data staleness threshold (hours)
    max_data_age_hours: u64,
}

impl Default for PricePredictor {
    fn default() -> Self {
        Self {
            min_samples: 10,
            max_prediction_days: 90,
            min_confidence: 0.25,
            max_data_age_hours: 72,
        }
    }
}

impl PricePredictor {
    /// Create a new predictor
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum samples
    pub fn with_min_samples(mut self, samples: usize) -> Self {
        self.min_samples = samples;
        self
    }

    /// Set minimum confidence
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence;
        self
    }

    /// Predict price based on historical data
    pub fn predict(
        &self,
        origin: IataCode,
        destination: IataCode,
        departure_date: Date,
        historical_data: &[PriceDataPoint],
        currency: CurrencyCode,
    ) -> OracleResult<PricePrediction> {
        // Check data availability
        if historical_data.len() < self.min_samples {
            return Err(OracleError::InsufficientData {
                required: self.min_samples,
                available: historical_data.len(),
            });
        }

        // Check data freshness
        if let Some(newest) = historical_data.iter().max_by_key(|d| d.timestamp) {
            let now = OffsetDateTime::now_utc().unix_timestamp();
            let age_hours = ((now - newest.timestamp) / 3600) as u64;
            if age_hours > self.max_data_age_hours {
                return Err(OracleError::StaleData {
                    age_hours,
                    max_hours: self.max_data_age_hours,
                });
            }
        }

        // Check prediction range
        let today = OffsetDateTime::now_utc().date();
        let days_until = (departure_date - today).whole_days().max(0) as u32;
        if days_until > self.max_prediction_days {
            return Err(OracleError::DateOutOfRange {
                days_ahead: days_until,
                max_days: self.max_prediction_days,
            });
        }

        // Simple prediction: weighted average of recent prices
        // In production, this would use vaya-ml models
        let (predicted_price, confidence) = self.calculate_prediction(historical_data, days_until);

        if confidence < self.min_confidence {
            return Err(OracleError::LowConfidence {
                confidence,
                threshold: self.min_confidence,
            });
        }

        // Calculate trend
        let (trend, change_percent) = self.calculate_trend(historical_data);

        let mut prediction = PricePrediction::new(
            origin,
            destination,
            departure_date,
            MinorUnits::new(predicted_price as i64),
            currency,
            confidence,
        );

        prediction = prediction.with_trend(trend, change_percent);
        prediction.calculate_recommendation();

        Ok(prediction)
    }

    /// Calculate prediction from historical data
    fn calculate_prediction(&self, data: &[PriceDataPoint], days_until: u32) -> (f64, f64) {
        // Filter data for similar booking windows
        let relevant_data: Vec<&PriceDataPoint> = data
            .iter()
            .filter(|d| {
                let diff = (d.days_before_departure as i32 - days_until as i32).unsigned_abs();
                diff <= 7 // Within 7 days of target booking window
            })
            .collect();

        if relevant_data.is_empty() {
            // Fallback to all data
            let avg: f64 =
                data.iter().map(|d| d.price.as_i64() as f64).sum::<f64>() / data.len() as f64;
            let confidence = 0.3; // Low confidence for fallback
            return (avg, confidence);
        }

        // Weighted average (more recent = higher weight)
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for dp in &relevant_data {
            let age_days = ((now - dp.timestamp) / 86400).max(1) as f64;
            let weight = 1.0 / age_days.sqrt(); // Diminishing weight with age
            weighted_sum += dp.price.as_i64() as f64 * weight;
            total_weight += weight;
        }

        let predicted = weighted_sum / total_weight;

        // Calculate confidence based on data quality
        let sample_factor = (relevant_data.len() as f64 / 20.0).min(1.0);
        let recency_factor = {
            if let Some(newest) = relevant_data.iter().max_by_key(|d| d.timestamp) {
                let hours_old = ((now - newest.timestamp) / 3600) as f64;
                (1.0 - hours_old / 168.0).max(0.0) // 7 days decay
            } else {
                0.5
            }
        };
        let confidence = sample_factor * 0.5 + recency_factor * 0.5;

        (predicted, confidence.min(0.95))
    }

    /// Calculate price trend from historical data
    fn calculate_trend(&self, data: &[PriceDataPoint]) -> (PriceTrend, f64) {
        if data.len() < 2 {
            return (PriceTrend::Stable, 0.0);
        }

        // Sort by timestamp
        let mut sorted: Vec<&PriceDataPoint> = data.iter().collect();
        sorted.sort_by_key(|d| d.timestamp);

        // Compare recent vs older prices
        let mid = sorted.len() / 2;
        let older_avg: f64 = sorted[..mid]
            .iter()
            .map(|d| d.price.as_i64() as f64)
            .sum::<f64>()
            / mid as f64;
        let newer_avg: f64 = sorted[mid..]
            .iter()
            .map(|d| d.price.as_i64() as f64)
            .sum::<f64>()
            / (sorted.len() - mid) as f64;

        let change_percent = ((newer_avg - older_avg) / older_avg) * 100.0;
        let trend = PriceTrend::from_change_percent(change_percent);

        (trend, change_percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_data_point(price: i64, days_before: u32, hours_ago: i64) -> PriceDataPoint {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        PriceDataPoint {
            price: MinorUnits::new(price),
            currency: CurrencyCode::SGD,
            timestamp: now - (hours_ago * 3600),
            days_before_departure: days_before,
            day_of_week: 1,
            is_weekend_departure: false,
            is_holiday: false,
        }
    }

    #[test]
    fn test_confidence_level() {
        assert_eq!(
            ConfidenceLevel::from_confidence(0.95),
            ConfidenceLevel::VeryHigh
        );
        assert_eq!(
            ConfidenceLevel::from_confidence(0.80),
            ConfidenceLevel::High
        );
        assert_eq!(
            ConfidenceLevel::from_confidence(0.60),
            ConfidenceLevel::Medium
        );
        assert_eq!(ConfidenceLevel::from_confidence(0.35), ConfidenceLevel::Low);
        assert_eq!(
            ConfidenceLevel::from_confidence(0.10),
            ConfidenceLevel::VeryLow
        );
    }

    #[test]
    fn test_price_trend() {
        assert_eq!(PriceTrend::from_change_percent(15.0), PriceTrend::StrongUp);
        assert_eq!(PriceTrend::from_change_percent(5.0), PriceTrend::Up);
        assert_eq!(PriceTrend::from_change_percent(0.0), PriceTrend::Stable);
        assert_eq!(PriceTrend::from_change_percent(-5.0), PriceTrend::Down);
        assert_eq!(
            PriceTrend::from_change_percent(-15.0),
            PriceTrend::StrongDown
        );

        assert!(PriceTrend::Down.is_favorable());
        assert!(!PriceTrend::Up.is_favorable());
    }

    #[test]
    fn test_prediction_creation() {
        let prediction = PricePrediction::new(
            IataCode::SIN,
            IataCode::BKK,
            Date::from_calendar_date(2026, time::Month::July, 15).unwrap(),
            MinorUnits::new(25000),
            CurrencyCode::SGD,
            0.85,
        );

        assert_eq!(prediction.predicted_price.as_i64(), 25000);
        assert_eq!(prediction.confidence, 0.85);
        assert_eq!(prediction.confidence_level, ConfidenceLevel::High);
        assert!(prediction.is_valid());
    }

    #[test]
    fn test_prediction_recommendation() {
        let mut prediction = PricePrediction::new(
            IataCode::SIN,
            IataCode::BKK,
            Date::from_calendar_date(2026, time::Month::July, 15).unwrap(),
            MinorUnits::new(25000),
            CurrencyCode::SGD,
            0.85,
        );

        prediction.trend = PriceTrend::StrongUp;
        prediction.days_until_departure = 30; // Enough time
        prediction.calculate_recommendation();
        assert_eq!(prediction.recommendation, BookingRecommendation::BookNow);

        prediction.trend = PriceTrend::Up;
        prediction.days_until_departure = 30;
        prediction.calculate_recommendation();
        assert_eq!(prediction.recommendation, BookingRecommendation::BookSoon);
    }

    #[test]
    fn test_predictor_insufficient_data() {
        let predictor = PricePredictor::new().with_min_samples(10);
        let data = vec![make_data_point(25000, 30, 1)]; // Only 1 data point

        let result = predictor.predict(
            IataCode::SIN,
            IataCode::BKK,
            Date::from_calendar_date(2026, time::Month::July, 15).unwrap(),
            &data,
            CurrencyCode::SGD,
        );

        assert!(matches!(result, Err(OracleError::InsufficientData { .. })));
    }

    #[test]
    fn test_predictor_basic_prediction() {
        let predictor = PricePredictor::new().with_min_samples(3);

        // Create historical data
        let data: Vec<PriceDataPoint> = (0..10)
            .map(|i| make_data_point(25000 + (i * 100), 30, i as i64))
            .collect();

        // Use a departure date within the 90 day prediction window
        let today = OffsetDateTime::now_utc().date();
        let departure = today + time::Duration::days(30);

        let result = predictor.predict(
            IataCode::SIN,
            IataCode::BKK,
            departure,
            &data,
            CurrencyCode::SGD,
        );

        assert!(result.is_ok());
        let prediction = result.unwrap();
        assert!(prediction.predicted_price.as_i64() > 0);
        assert!(prediction.confidence > 0.0);
    }

    #[test]
    fn test_trend_calculation() {
        let predictor = PricePredictor::new();

        // Rising prices
        let rising: Vec<PriceDataPoint> = (0..10)
            .map(|i| make_data_point(20000 + (i * 1000), 30, (10 - i) as i64))
            .collect();

        let (trend, change) = predictor.calculate_trend(&rising);
        assert!(matches!(trend, PriceTrend::Up | PriceTrend::StrongUp));
        assert!(change > 0.0);

        // Falling prices
        let falling: Vec<PriceDataPoint> = (0..10)
            .map(|i| make_data_point(30000 - (i * 1000), 30, (10 - i) as i64))
            .collect();

        let (trend, change) = predictor.calculate_trend(&falling);
        assert!(matches!(trend, PriceTrend::Down | PriceTrend::StrongDown));
        assert!(change < 0.0);
    }

    #[test]
    fn test_data_point_features() {
        let dp = make_data_point(25000, 30, 1);
        let features = dp.to_features();

        assert_eq!(features.len(), 5);
        assert_eq!(features[0], 25000.0); // price
        assert_eq!(features[1], 30.0); // days before departure
    }
}
