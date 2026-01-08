//! Neural network implementation

use crate::activation::Activation;
use crate::layer::Layer;
use crate::matrix::Matrix;
use crate::{MlError, MlResult};

/// A feedforward neural network
#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    /// Network layers
    layers: Vec<Layer>,
    /// Whether the network has been trained
    trained: bool,
}

impl NeuralNetwork {
    /// Create a new neural network builder
    pub fn builder() -> NeuralNetworkBuilder {
        NeuralNetworkBuilder::new()
    }

    /// Create a network from layers
    pub fn from_layers(layers: Vec<Layer>) -> Self {
        Self {
            layers,
            trained: false,
        }
    }

    /// Forward pass through the entire network
    pub fn forward(&mut self, input: &Matrix, training: bool) -> MlResult<Matrix> {
        let mut output = input.clone();
        for layer in &mut self.layers {
            output = layer.forward(&output, training)?;
        }
        Ok(output)
    }

    /// Predict (forward pass without caching)
    pub fn predict(&mut self, input: &Matrix) -> MlResult<Matrix> {
        self.forward(input, false)
    }

    /// Train the network using mini-batch gradient descent
    pub fn train(
        &mut self,
        x_train: &Matrix,
        y_train: &Matrix,
        epochs: usize,
        learning_rate: f32,
        batch_size: usize,
    ) -> MlResult<Vec<f32>> {
        let n_samples = x_train.rows();
        let mut losses = Vec::with_capacity(epochs);

        for epoch in 0..epochs {
            let mut epoch_loss = 0.0;
            let mut n_batches = 0;

            // Process in batches
            for batch_start in (0..n_samples).step_by(batch_size) {
                let batch_end = (batch_start + batch_size).min(n_samples);
                let batch_size = batch_end - batch_start;

                // Get batch data
                let x_batch = self.slice_rows(x_train, batch_start, batch_end);
                let y_batch = self.slice_rows(y_train, batch_start, batch_end);

                // Forward pass
                let output = self.forward(&x_batch, true)?;

                // Compute loss (MSE for regression)
                let diff = output.sub(&y_batch)?;
                let loss = diff.hadamard(&diff)?.mean();
                epoch_loss += loss;
                n_batches += 1;

                // Backward pass
                let grad_output = diff.scale(2.0 / batch_size as f32);
                self.backward(&grad_output, learning_rate)?;
            }

            let avg_loss = epoch_loss / n_batches as f32;
            losses.push(avg_loss);

            if epoch % 100 == 0 {
                tracing::debug!("Epoch {}: loss = {:.6}", epoch, avg_loss);
            }
        }

        self.trained = true;
        Ok(losses)
    }

    /// Backward pass through the network
    fn backward(&mut self, grad_output: &Matrix, learning_rate: f32) -> MlResult<()> {
        let mut grad = grad_output.clone();
        let mut updated_layers = Vec::with_capacity(self.layers.len());

        // Propagate gradients backwards
        for layer in self.layers.iter().rev() {
            let (grad_input, updated_layer) = layer.backward(&grad, learning_rate)?;
            grad = grad_input;
            updated_layers.push(updated_layer);
        }

        // Reverse to get correct order
        updated_layers.reverse();
        self.layers = updated_layers;

        Ok(())
    }

    /// Helper to slice rows from a matrix
    fn slice_rows(&self, m: &Matrix, start: usize, end: usize) -> Matrix {
        let cols = m.cols();
        let mut data = Vec::with_capacity((end - start) * cols);
        for i in start..end {
            for j in 0..cols {
                data.push(m.get(i, j));
            }
        }
        Matrix::from_flat(data, end - start, cols).unwrap()
    }

    /// Get the number of layers
    pub fn num_layers(&self) -> usize {
        self.layers.len()
    }

    /// Check if the network has been trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }
}

/// Builder for constructing neural networks
pub struct NeuralNetworkBuilder {
    layers: Vec<(usize, Activation)>,
    input_size: Option<usize>,
}

impl NeuralNetworkBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            layers: Vec::new(),
            input_size: None,
        }
    }

    /// Set the input size
    pub fn input(mut self, size: usize) -> Self {
        self.input_size = Some(size);
        self
    }

    /// Add a dense layer
    pub fn dense(mut self, size: usize, activation: Activation) -> Self {
        self.layers.push((size, activation));
        self
    }

    /// Build the network
    pub fn build(self) -> MlResult<NeuralNetwork> {
        let input_size = self
            .input_size
            .ok_or_else(|| MlError::InvalidParameter("Input size not specified".into()))?;

        if self.layers.is_empty() {
            return Err(MlError::InvalidParameter("No layers specified".into()));
        }

        let mut layers = Vec::with_capacity(self.layers.len());
        let mut prev_size = input_size;

        for (size, activation) in self.layers {
            layers.push(Layer::new(prev_size, size, activation));
            prev_size = size;
        }

        Ok(NeuralNetwork {
            layers,
            trained: false,
        })
    }
}

impl Default for NeuralNetworkBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_builder() {
        let network = NeuralNetwork::builder()
            .input(10)
            .dense(64, Activation::ReLU)
            .dense(32, Activation::ReLU)
            .dense(1, Activation::Linear)
            .build()
            .unwrap();

        assert_eq!(network.num_layers(), 3);
    }

    #[test]
    fn test_forward_pass() {
        let mut network = NeuralNetwork::builder()
            .input(2)
            .dense(4, Activation::ReLU)
            .dense(1, Activation::Linear)
            .build()
            .unwrap();

        let input = Matrix::from_vec(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        let output = network.predict(&input).unwrap();

        assert_eq!(output.shape(), (2, 1));
    }

    #[test]
    fn test_training() {
        // Simple XOR-like problem
        let mut network = NeuralNetwork::builder()
            .input(2)
            .dense(4, Activation::ReLU)
            .dense(1, Activation::Sigmoid)
            .build()
            .unwrap();

        let x = Matrix::from_vec(vec![
            vec![0.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
        ]);
        let y = Matrix::from_vec(vec![vec![0.0], vec![1.0], vec![1.0], vec![0.0]]);

        let losses = network.train(&x, &y, 100, 0.5, 4).unwrap();

        // Loss should decrease
        assert!(losses.last().unwrap() < losses.first().unwrap());
    }
}
