//! Neural network layers

use crate::activation::Activation;
use crate::matrix::Matrix;
use crate::MlResult;

/// A dense (fully connected) neural network layer
#[derive(Debug, Clone)]
pub struct Layer {
    /// Weight matrix (input_size x output_size)
    weights: Matrix,
    /// Bias vector (1 x output_size)
    biases: Matrix,
    /// Activation function
    activation: Activation,
    /// Cached input (for backpropagation)
    cached_input: Option<Matrix>,
    /// Cached pre-activation output (for backpropagation)
    cached_z: Option<Matrix>,
}

impl Layer {
    /// Create a new dense layer
    pub fn new(input_size: usize, output_size: usize, activation: Activation) -> Self {
        // Initialize weights using Xavier/He initialization based on activation
        let weights = match activation {
            Activation::ReLU | Activation::LeakyReLU(_) => {
                Matrix::random_he(input_size, output_size)
            }
            _ => Matrix::random_xavier(input_size, output_size),
        };

        let biases = Matrix::zeros(1, output_size);

        Self {
            weights,
            biases,
            activation,
            cached_input: None,
            cached_z: None,
        }
    }

    /// Create a layer with specific weights and biases
    pub fn with_weights(weights: Matrix, biases: Matrix, activation: Activation) -> MlResult<Self> {
        if biases.rows() != 1 || biases.cols() != weights.cols() {
            return Err(crate::MlError::DimensionMismatch {
                expected: (1, weights.cols()),
                actual: biases.shape(),
            });
        }

        Ok(Self {
            weights,
            biases,
            activation,
            cached_input: None,
            cached_z: None,
        })
    }

    /// Forward pass
    pub fn forward(&mut self, input: &Matrix, training: bool) -> MlResult<Matrix> {
        // z = X @ W + b
        let z = input.matmul(&self.weights)?.add_bias(&self.biases)?;

        // Cache for backpropagation
        if training {
            self.cached_input = Some(input.clone());
            self.cached_z = Some(z.clone());
        }

        // Apply activation
        let output = self.activation.forward(&z);

        Ok(output)
    }

    /// Backward pass (returns gradients for weights, biases, and input)
    pub fn backward(&self, grad_output: &Matrix, learning_rate: f32) -> MlResult<(Matrix, Layer)> {
        let input = self
            .cached_input
            .as_ref()
            .ok_or(crate::MlError::NotTrained)?;
        let z = self.cached_z.as_ref().ok_or(crate::MlError::NotTrained)?;

        // Gradient through activation
        let grad_z = self.activation.backward(z, grad_output);

        // Gradient for weights: dW = X^T @ dZ
        let grad_weights = input.transpose().matmul(&grad_z)?;

        // Gradient for biases: db = sum(dZ, axis=0)
        let grad_biases = grad_z.sum_axis(0);

        // Gradient for input: dX = dZ @ W^T
        let grad_input = grad_z.matmul(&self.weights.transpose())?;

        // Create updated layer with new weights
        let new_weights = self
            .weights
            .sub(&grad_weights.scale(learning_rate / input.rows() as f32))?;
        let new_biases = self
            .biases
            .sub(&grad_biases.scale(learning_rate / input.rows() as f32))?;

        let updated_layer = Layer {
            weights: new_weights,
            biases: new_biases,
            activation: self.activation,
            cached_input: None,
            cached_z: None,
        };

        Ok((grad_input, updated_layer))
    }

    /// Get the input size
    pub fn input_size(&self) -> usize {
        self.weights.rows()
    }

    /// Get the output size
    pub fn output_size(&self) -> usize {
        self.weights.cols()
    }

    /// Get weights
    pub fn weights(&self) -> &Matrix {
        &self.weights
    }

    /// Get biases
    pub fn biases(&self) -> &Matrix {
        &self.biases
    }

    /// Get activation function
    pub fn activation(&self) -> Activation {
        self.activation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let layer = Layer::new(10, 5, Activation::ReLU);
        assert_eq!(layer.input_size(), 10);
        assert_eq!(layer.output_size(), 5);
    }

    #[test]
    fn test_forward_pass() {
        let mut layer = Layer::new(3, 2, Activation::Linear);
        let input = Matrix::from_vec(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);

        let output = layer.forward(&input, false).unwrap();
        assert_eq!(output.shape(), (2, 2));
    }
}
