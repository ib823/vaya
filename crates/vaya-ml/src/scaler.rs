//! Feature scaling utilities

use crate::matrix::Matrix;

/// Standard scaler (z-score normalization)
///
/// Transforms features to have mean 0 and standard deviation 1.
#[derive(Debug, Clone)]
pub struct StandardScaler {
    /// Mean of each feature
    mean: Option<Vec<f32>>,
    /// Standard deviation of each feature
    std: Option<Vec<f32>>,
}

impl StandardScaler {
    /// Create a new standard scaler
    pub fn new() -> Self {
        Self {
            mean: None,
            std: None,
        }
    }

    /// Fit the scaler to the data
    pub fn fit(&mut self, data: &Matrix) {
        let n_features = data.cols();
        let n_samples = data.rows() as f32;

        let mut mean = vec![0.0; n_features];
        let mut std = vec![0.0; n_features];

        // Calculate mean
        for i in 0..data.rows() {
            for j in 0..n_features {
                mean[j] += data.get(i, j);
            }
        }
        for j in 0..n_features {
            mean[j] /= n_samples;
        }

        // Calculate standard deviation
        for i in 0..data.rows() {
            for j in 0..n_features {
                let diff = data.get(i, j) - mean[j];
                std[j] += diff * diff;
            }
        }
        for j in 0..n_features {
            std[j] = (std[j] / n_samples).sqrt();
            // Avoid division by zero
            if std[j] < 1e-8 {
                std[j] = 1.0;
            }
        }

        self.mean = Some(mean);
        self.std = Some(std);
    }

    /// Transform the data using the fitted parameters
    pub fn transform(&self, data: &Matrix) -> Option<Matrix> {
        let mean = self.mean.as_ref()?;
        let std = self.std.as_ref()?;

        let mut result = Matrix::zeros(data.rows(), data.cols());

        for i in 0..data.rows() {
            for j in 0..data.cols() {
                let scaled = (data.get(i, j) - mean[j]) / std[j];
                result.set(i, j, scaled);
            }
        }

        Some(result)
    }

    /// Fit and transform in one step
    pub fn fit_transform(&mut self, data: &Matrix) -> Matrix {
        self.fit(data);
        self.transform(data).unwrap()
    }

    /// Inverse transform to original scale
    pub fn inverse_transform(&self, data: &Matrix) -> Option<Matrix> {
        let mean = self.mean.as_ref()?;
        let std = self.std.as_ref()?;

        let mut result = Matrix::zeros(data.rows(), data.cols());

        for i in 0..data.rows() {
            for j in 0..data.cols() {
                let original = data.get(i, j) * std[j] + mean[j];
                result.set(i, j, original);
            }
        }

        Some(result)
    }

    /// Check if the scaler has been fitted
    pub fn is_fitted(&self) -> bool {
        self.mean.is_some() && self.std.is_some()
    }
}

impl Default for StandardScaler {
    fn default() -> Self {
        Self::new()
    }
}

/// Min-max scaler
///
/// Transforms features to be in the range [0, 1].
#[derive(Debug, Clone)]
pub struct MinMaxScaler {
    /// Minimum of each feature
    min: Option<Vec<f32>>,
    /// Maximum of each feature
    max: Option<Vec<f32>>,
}

impl MinMaxScaler {
    /// Create a new min-max scaler
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    /// Fit the scaler to the data
    pub fn fit(&mut self, data: &Matrix) {
        let n_features = data.cols();

        let mut min = vec![f32::INFINITY; n_features];
        let mut max = vec![f32::NEG_INFINITY; n_features];

        for i in 0..data.rows() {
            for j in 0..n_features {
                let val = data.get(i, j);
                min[j] = min[j].min(val);
                max[j] = max[j].max(val);
            }
        }

        // Avoid division by zero
        for j in 0..n_features {
            if (max[j] - min[j]).abs() < 1e-8 {
                max[j] = min[j] + 1.0;
            }
        }

        self.min = Some(min);
        self.max = Some(max);
    }

    /// Transform the data using the fitted parameters
    pub fn transform(&self, data: &Matrix) -> Option<Matrix> {
        let min = self.min.as_ref()?;
        let max = self.max.as_ref()?;

        let mut result = Matrix::zeros(data.rows(), data.cols());

        for i in 0..data.rows() {
            for j in 0..data.cols() {
                let scaled = (data.get(i, j) - min[j]) / (max[j] - min[j]);
                result.set(i, j, scaled);
            }
        }

        Some(result)
    }

    /// Fit and transform in one step
    pub fn fit_transform(&mut self, data: &Matrix) -> Matrix {
        self.fit(data);
        self.transform(data).unwrap()
    }

    /// Inverse transform to original scale
    pub fn inverse_transform(&self, data: &Matrix) -> Option<Matrix> {
        let min = self.min.as_ref()?;
        let max = self.max.as_ref()?;

        let mut result = Matrix::zeros(data.rows(), data.cols());

        for i in 0..data.rows() {
            for j in 0..data.cols() {
                let original = data.get(i, j) * (max[j] - min[j]) + min[j];
                result.set(i, j, original);
            }
        }

        Some(result)
    }

    /// Check if the scaler has been fitted
    pub fn is_fitted(&self) -> bool {
        self.min.is_some() && self.max.is_some()
    }
}

impl Default for MinMaxScaler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_scaler() {
        let data = Matrix::from_vec(vec![vec![1.0, 100.0], vec![2.0, 200.0], vec![3.0, 300.0]]);

        let mut scaler = StandardScaler::new();
        let scaled = scaler.fit_transform(&data);

        // Check that mean is approximately 0
        let mean = scaled.sum_axis(0).scale(1.0 / 3.0);
        assert!(mean.get(0, 0).abs() < 1e-5);
        assert!(mean.get(0, 1).abs() < 1e-5);

        // Check inverse transform
        let recovered = scaler.inverse_transform(&scaled).unwrap();
        assert!((recovered.get(0, 0) - 1.0).abs() < 1e-5);
        assert!((recovered.get(1, 1) - 200.0).abs() < 1e-3);
    }

    #[test]
    fn test_minmax_scaler() {
        let data = Matrix::from_vec(vec![vec![1.0, 100.0], vec![2.0, 200.0], vec![3.0, 300.0]]);

        let mut scaler = MinMaxScaler::new();
        let scaled = scaler.fit_transform(&data);

        // Check that min is 0 and max is 1
        assert!((scaled.get(0, 0) - 0.0).abs() < 1e-5);
        assert!((scaled.get(2, 0) - 1.0).abs() < 1e-5);
        assert!((scaled.get(0, 1) - 0.0).abs() < 1e-5);
        assert!((scaled.get(2, 1) - 1.0).abs() < 1e-5);
    }
}
