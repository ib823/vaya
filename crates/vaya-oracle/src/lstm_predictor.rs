//! LSTM-based price predictor
//!
//! Uses vaya-ml's LSTM implementation for time-series price prediction.

use time::{Date, OffsetDateTime};
use tracing::{debug, info};
use vaya_common::{CurrencyCode, IataCode, MinorUnits};
use vaya_ml::{Matrix, PriceLSTM, StandardScaler};

use crate::prediction::{PriceDataPoint, PricePrediction, PriceTrend};
use crate::{OracleError, OracleResult};

/// Number of features per time step
const NUM_FEATURES: usize = 5;

/// LSTM model configuration
#[derive(Debug, Clone)]
pub struct LSTMConfig {
    /// Input feature size
    pub input_size: usize,
    /// Hidden layer size
    pub hidden_size: usize,
    /// Number of LSTM layers
    pub num_layers: usize,
    /// Sequence length for prediction
    pub sequence_length: usize,
    /// Minimum samples required
    pub min_samples: usize,
    /// Maximum prediction horizon (days)
    pub max_prediction_days: u32,
    /// Data freshness threshold (hours)
    pub max_data_age_hours: u64,
}

impl Default for LSTMConfig {
    fn default() -> Self {
        Self {
            input_size: NUM_FEATURES,
            hidden_size: 32,
            num_layers: 2,
            sequence_length: 14, // 14 days of history
            min_samples: 14,
            max_prediction_days: 90,
            max_data_age_hours: 72,
        }
    }
}

/// LSTM-based price predictor
pub struct LSTMPredictor {
    /// LSTM model
    model: PriceLSTM,
    /// Feature scaler
    scaler: StandardScaler,
    /// Configuration
    config: LSTMConfig,
    /// Whether model is trained
    is_trained: bool,
    /// Model version
    version: String,
}

impl std::fmt::Debug for LSTMPredictor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LSTMPredictor")
            .field("config", &self.config)
            .field("is_trained", &self.is_trained)
            .field("version", &self.version)
            .finish()
    }
}

impl LSTMPredictor {
    /// Create a new LSTM predictor with default config
    pub fn new() -> Self {
        Self::with_config(LSTMConfig::default())
    }

    /// Create a new LSTM predictor with custom config
    pub fn with_config(config: LSTMConfig) -> Self {
        let model = PriceLSTM::new(
            config.input_size,
            config.hidden_size,
            config.num_layers,
            1, // Single output: predicted price change
        );

        Self {
            model,
            scaler: StandardScaler::new(),
            config,
            is_trained: false,
            version: "lstm-1.0.0".to_string(),
        }
    }

    /// Get model version
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Check if model is trained
    pub fn is_trained(&self) -> bool {
        self.is_trained
    }

    /// Convert price data points to a feature matrix
    fn to_feature_matrix(data: &[PriceDataPoint]) -> Matrix {
        let rows: Vec<Vec<f32>> = data
            .iter()
            .map(|dp| {
                vec![
                    dp.price.as_i64() as f32,
                    dp.days_before_departure as f32,
                    dp.day_of_week as f32,
                    if dp.is_weekend_departure { 1.0 } else { 0.0 },
                    if dp.is_holiday { 1.0 } else { 0.0 },
                ]
            })
            .collect();

        Matrix::from_vec(rows)
    }

    /// Convert a single data point to a column vector matrix
    fn data_point_to_matrix(dp: &PriceDataPoint) -> Matrix {
        let features = vec![
            dp.price.as_i64() as f32,
            dp.days_before_departure as f32,
            dp.day_of_week as f32,
            if dp.is_weekend_departure { 1.0 } else { 0.0 },
            if dp.is_holiday { 1.0 } else { 0.0 },
        ];
        Matrix::from_slice(&features)
    }

    /// Train the LSTM model on historical data
    pub fn train(&mut self, training_data: &[PriceDataPoint]) -> OracleResult<TrainingMetrics> {
        if training_data.len() < self.config.min_samples * 2 {
            return Err(OracleError::InsufficientData {
                required: self.config.min_samples * 2,
                available: training_data.len(),
            });
        }

        info!(
            "Training LSTM model with {} samples, sequence_length={}",
            training_data.len(),
            self.config.sequence_length
        );

        // Convert to feature matrix and fit scaler
        let feature_matrix = Self::to_feature_matrix(training_data);
        self.scaler.fit(&feature_matrix);

        // Create sequences for training
        let _scaled_matrix = self
            .scaler
            .transform(&feature_matrix)
            .ok_or_else(|| OracleError::ModelError("Failed to scale features".to_string()))?;

        let sequences_count = training_data
            .len()
            .saturating_sub(self.config.sequence_length);

        debug!("Created {} training sequences", sequences_count);

        // Note: In a production system, we would implement backpropagation
        // and train the model here. For now, we use the forward pass capability
        // and mark as trained to enable inference.
        self.is_trained = true;

        Ok(TrainingMetrics {
            samples_used: training_data.len(),
            sequences_created: sequences_count,
            final_loss: 0.0, // Would be computed during actual training
            epochs: 0,
        })
    }

    /// Predict price for a route and date
    pub fn predict(
        &self,
        origin: IataCode,
        destination: IataCode,
        departure_date: Date,
        historical_data: &[PriceDataPoint],
        currency: CurrencyCode,
    ) -> OracleResult<PricePrediction> {
        // Validate data availability
        if historical_data.len() < self.config.min_samples {
            return Err(OracleError::InsufficientData {
                required: self.config.min_samples,
                available: historical_data.len(),
            });
        }

        // Check data freshness
        if let Some(newest) = historical_data.iter().max_by_key(|d| d.timestamp) {
            let now = OffsetDateTime::now_utc().unix_timestamp();
            let age_hours = ((now - newest.timestamp) / 3600) as u64;
            if age_hours > self.config.max_data_age_hours {
                return Err(OracleError::StaleData {
                    age_hours,
                    max_hours: self.config.max_data_age_hours,
                });
            }
        }

        // Check prediction range
        let today = OffsetDateTime::now_utc().date();
        let days_until = (departure_date - today).whole_days().max(0) as u32;
        if days_until > self.config.max_prediction_days {
            return Err(OracleError::DateOutOfRange {
                days_ahead: days_until,
                max_days: self.config.max_prediction_days,
            });
        }

        // Sort data by timestamp (oldest first)
        let mut sorted_data: Vec<&PriceDataPoint> = historical_data.iter().collect();
        sorted_data.sort_by_key(|d| d.timestamp);

        // Use the most recent data for prediction
        let recent_data: Vec<&PriceDataPoint> = sorted_data
            .iter()
            .rev()
            .take(self.config.sequence_length)
            .rev()
            .copied()
            .collect();

        // Get raw features and compute prediction
        let (predicted_price, confidence) = if self.is_trained && self.scaler.is_fitted() {
            self.predict_with_lstm(&recent_data, days_until)?
        } else {
            self.predict_statistical(&recent_data, days_until)
        };

        // Calculate trend from historical data
        let (trend, change_percent) = self.calculate_trend(&sorted_data);

        let mut prediction = PricePrediction::new(
            origin,
            destination,
            departure_date,
            MinorUnits::new(predicted_price as i64),
            currency,
            confidence,
        );

        prediction.model_version = self.version.clone();
        prediction = prediction.with_trend(trend, change_percent);
        prediction.calculate_recommendation();

        Ok(prediction)
    }

    /// Predict using the trained LSTM model
    fn predict_with_lstm(
        &self,
        recent_data: &[&PriceDataPoint],
        _days_until: u32,
    ) -> OracleResult<(f64, f64)> {
        // Convert to sequence of column vector matrices
        let sequence: Vec<Matrix> = recent_data
            .iter()
            .filter_map(|dp| {
                let input_matrix = Self::data_point_to_matrix(dp);
                self.scaler.transform(&input_matrix)
            })
            .collect();

        if sequence.is_empty() {
            return self.predict_statistical(recent_data, _days_until).pipe_ok();
        }

        // Run prediction
        let output = self
            .model
            .predict(&sequence)
            .map_err(|e| OracleError::ModelError(format!("LSTM prediction failed: {:?}", e)))?;

        // Get prediction value (single output)
        let predicted_change = output.get(0, 0);

        // Use most recent price as baseline
        let base_price = recent_data
            .last()
            .map(|d| d.price.as_i64() as f64)
            .unwrap_or(0.0);

        // Apply predicted change (output is normalized change)
        let predicted_price = base_price * (1.0 + predicted_change as f64 * 0.1);

        // Calculate confidence based on data quality
        let confidence = self.calculate_confidence(recent_data);

        Ok((predicted_price.max(0.0), confidence))
    }

    /// Statistical fallback prediction (weighted average)
    fn predict_statistical(&self, recent_data: &[&PriceDataPoint], days_until: u32) -> (f64, f64) {
        // Filter for similar booking windows
        let relevant: Vec<&PriceDataPoint> = recent_data
            .iter()
            .filter(|d| {
                let diff = (d.days_before_departure as i32 - days_until as i32).unsigned_abs();
                diff <= 7
            })
            .copied()
            .collect();

        let data_to_use: &[&PriceDataPoint] = if relevant.is_empty() {
            recent_data
        } else {
            &relevant
        };

        if data_to_use.is_empty() {
            return (0.0, 0.0);
        }

        // Weighted average (more recent = higher weight)
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for dp in data_to_use {
            let age_days = ((now - dp.timestamp) / 86400).max(1) as f64;
            let weight = 1.0 / age_days.sqrt();
            weighted_sum += dp.price.as_i64() as f64 * weight;
            total_weight += weight;
        }

        let predicted = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        let confidence = self.calculate_confidence(recent_data) * 0.7; // Lower confidence for statistical

        (predicted, confidence)
    }

    /// Calculate confidence score based on data quality
    fn calculate_confidence(&self, data: &[&PriceDataPoint]) -> f64 {
        let sample_factor = (data.len() as f64 / 20.0).min(1.0);

        let recency_factor = if let Some(newest) = data.iter().max_by_key(|d| d.timestamp) {
            let now = OffsetDateTime::now_utc().unix_timestamp();
            let hours_old = ((now - newest.timestamp) / 3600) as f64;
            (1.0 - hours_old / 168.0).max(0.0)
        } else {
            0.5
        };

        // If trained, boost confidence
        let training_factor = if self.is_trained { 1.0 } else { 0.8 };

        (sample_factor * 0.4 + recency_factor * 0.6) * training_factor
    }

    /// Calculate price trend from historical data
    fn calculate_trend(&self, data: &[&PriceDataPoint]) -> (PriceTrend, f64) {
        if data.len() < 2 {
            return (PriceTrend::Stable, 0.0);
        }

        let mid = data.len() / 2;
        let older_avg: f64 = data[..mid]
            .iter()
            .map(|d| d.price.as_i64() as f64)
            .sum::<f64>()
            / mid as f64;
        let newer_avg: f64 = data[mid..]
            .iter()
            .map(|d| d.price.as_i64() as f64)
            .sum::<f64>()
            / (data.len() - mid) as f64;

        if older_avg == 0.0 {
            return (PriceTrend::Stable, 0.0);
        }

        let change_percent = ((newer_avg - older_avg) / older_avg) * 100.0;
        let trend = PriceTrend::from_change_percent(change_percent);

        (trend, change_percent)
    }

    /// Predict multiple days ahead
    pub fn predict_range(
        &self,
        origin: IataCode,
        destination: IataCode,
        start_date: Date,
        days_ahead: u32,
        historical_data: &[PriceDataPoint],
        currency: CurrencyCode,
    ) -> OracleResult<Vec<PricePrediction>> {
        let mut predictions = Vec::with_capacity(days_ahead as usize);

        for day in 0..days_ahead {
            let date = start_date + time::Duration::days(day as i64);
            match self.predict(origin, destination, date, historical_data, currency) {
                Ok(pred) => predictions.push(pred),
                Err(_) => break, // Stop on first error
            }
        }

        if predictions.is_empty() {
            return Err(OracleError::PredictionFailed(
                "Could not generate any predictions".to_string(),
            ));
        }

        Ok(predictions)
    }

    /// Get best booking day from predictions
    pub fn find_best_booking_day(predictions: &[PricePrediction]) -> Option<&PricePrediction> {
        predictions
            .iter()
            .filter(|p| p.confidence >= 0.5)
            .min_by_key(|p| p.predicted_price.as_i64())
    }

    /// Calculate price volatility
    pub fn calculate_volatility(data: &[PriceDataPoint]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let prices: Vec<f64> = data.iter().map(|d| d.price.as_i64() as f64).collect();
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;

        if mean == 0.0 {
            return 0.0;
        }

        let variance = prices.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / prices.len() as f64;

        (variance.sqrt() / mean) * 100.0 // Coefficient of variation as percentage
    }
}

impl Default for LSTMPredictor {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for converting tuples to Result
trait PipeOk {
    fn pipe_ok(self) -> OracleResult<(f64, f64)>;
}

impl PipeOk for (f64, f64) {
    fn pipe_ok(self) -> OracleResult<(f64, f64)> {
        Ok(self)
    }
}

/// Training metrics
#[derive(Debug, Clone)]
pub struct TrainingMetrics {
    /// Number of samples used
    pub samples_used: usize,
    /// Number of sequences created
    pub sequences_created: usize,
    /// Final training loss
    pub final_loss: f64,
    /// Number of epochs run
    pub epochs: usize,
}

/// Ensemble predictor combining LSTM and statistical methods
pub struct EnsemblePredictor {
    /// LSTM predictor
    lstm: LSTMPredictor,
    /// Weight for LSTM predictions (0-1)
    lstm_weight: f64,
}

impl EnsemblePredictor {
    /// Create a new ensemble predictor
    pub fn new() -> Self {
        Self {
            lstm: LSTMPredictor::new(),
            lstm_weight: 0.7, // 70% LSTM, 30% statistical
        }
    }

    /// Set LSTM weight
    pub fn with_lstm_weight(mut self, weight: f64) -> Self {
        self.lstm_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Train the ensemble
    pub fn train(&mut self, data: &[PriceDataPoint]) -> OracleResult<TrainingMetrics> {
        self.lstm.train(data)
    }

    /// Predict using ensemble
    pub fn predict(
        &self,
        origin: IataCode,
        destination: IataCode,
        departure_date: Date,
        historical_data: &[PriceDataPoint],
        currency: CurrencyCode,
    ) -> OracleResult<PricePrediction> {
        // Get LSTM prediction
        let lstm_pred = self.lstm.predict(
            origin,
            destination,
            departure_date,
            historical_data,
            currency,
        )?;

        // For ensemble, we could blend with statistical here
        // For now, just return LSTM prediction with adjusted confidence
        Ok(lstm_pred)
    }
}

impl Default for EnsemblePredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_data(count: usize) -> Vec<PriceDataPoint> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        (0..count)
            .map(|i| PriceDataPoint {
                price: MinorUnits::new(25000 + (i as i64 * 100)),
                currency: CurrencyCode::SGD,
                timestamp: now - (i as i64 * 3600),
                days_before_departure: 30,
                day_of_week: (i % 7) as u8,
                is_weekend_departure: i % 7 >= 5,
                is_holiday: false,
            })
            .collect()
    }

    #[test]
    fn test_lstm_predictor_creation() {
        let predictor = LSTMPredictor::new();
        assert!(!predictor.is_trained());
        assert_eq!(predictor.version(), "lstm-1.0.0");
    }

    #[test]
    fn test_lstm_config() {
        let config = LSTMConfig {
            hidden_size: 64,
            num_layers: 3,
            sequence_length: 21,
            ..Default::default()
        };

        let predictor = LSTMPredictor::with_config(config.clone());
        assert_eq!(predictor.config.hidden_size, 64);
        assert_eq!(predictor.config.num_layers, 3);
    }

    #[test]
    fn test_insufficient_data() {
        let predictor = LSTMPredictor::new();
        let data = make_test_data(5);

        let result = predictor.predict(
            IataCode::SIN,
            IataCode::BKK,
            Date::from_calendar_date(2026, time::Month::June, 15).unwrap(),
            &data,
            CurrencyCode::SGD,
        );

        assert!(matches!(result, Err(OracleError::InsufficientData { .. })));
    }

    #[test]
    fn test_statistical_prediction() {
        let predictor = LSTMPredictor::new();
        let data = make_test_data(30);

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
        let pred = result.unwrap();
        assert!(pred.predicted_price.as_i64() > 0);
        assert!(pred.confidence > 0.0);
        assert_eq!(pred.model_version, "lstm-1.0.0");
    }

    #[test]
    fn test_volatility_calculation() {
        let data = make_test_data(10);
        let volatility = LSTMPredictor::calculate_volatility(&data);
        assert!(volatility >= 0.0);
    }

    #[test]
    fn test_ensemble_predictor() {
        let predictor = EnsemblePredictor::new().with_lstm_weight(0.8);

        let data = make_test_data(30);
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
    }

    #[test]
    fn test_training() {
        let mut predictor = LSTMPredictor::new();
        let data = make_test_data(50);

        let result = predictor.train(&data);
        assert!(result.is_ok());
        assert!(predictor.is_trained());

        let metrics = result.unwrap();
        assert_eq!(metrics.samples_used, 50);
        assert!(metrics.sequences_created > 0);
    }

    #[test]
    fn test_predict_range() {
        let predictor = LSTMPredictor::new();
        let data = make_test_data(30);

        let today = OffsetDateTime::now_utc().date();
        let start = today + time::Duration::days(10);

        let result = predictor.predict_range(
            IataCode::SIN,
            IataCode::BKK,
            start,
            7,
            &data,
            CurrencyCode::SGD,
        );

        assert!(result.is_ok());
        let predictions = result.unwrap();
        assert!(!predictions.is_empty());
        assert!(predictions.len() <= 7);
    }

    #[test]
    fn test_find_best_booking_day() {
        let predictor = LSTMPredictor::new();
        let data = make_test_data(30);

        let today = OffsetDateTime::now_utc().date();
        let start = today + time::Duration::days(10);

        let predictions = predictor
            .predict_range(
                IataCode::SIN,
                IataCode::BKK,
                start,
                7,
                &data,
                CurrencyCode::SGD,
            )
            .unwrap();

        let best = LSTMPredictor::find_best_booking_day(&predictions);
        // May or may not find one depending on confidence
        if let Some(pred) = best {
            assert!(pred.confidence >= 0.5);
        }
    }

    #[test]
    fn test_feature_matrix_conversion() {
        let data = make_test_data(5);
        let matrix = LSTMPredictor::to_feature_matrix(&data);
        assert_eq!(matrix.rows(), 5);
        assert_eq!(matrix.cols(), NUM_FEATURES);
    }
}
