//! Activation functions for neural networks

use crate::matrix::Matrix;

/// Activation function types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Activation {
    /// Linear activation (identity function)
    Linear,
    /// Rectified Linear Unit: max(0, x)
    ReLU,
    /// Leaky ReLU: max(alpha * x, x)
    LeakyReLU(f32),
    /// Sigmoid: 1 / (1 + exp(-x))
    Sigmoid,
    /// Hyperbolic tangent: tanh(x)
    Tanh,
    /// Softmax (for output layer)
    Softmax,
}

impl Activation {
    /// Apply the activation function to a matrix
    pub fn forward(&self, x: &Matrix) -> Matrix {
        match self {
            Activation::Linear => x.clone(),
            Activation::ReLU => x.map(|v| v.max(0.0)),
            Activation::LeakyReLU(alpha) => {
                let alpha = *alpha;
                x.map(|v| if v > 0.0 { v } else { alpha * v })
            }
            Activation::Sigmoid => x.map(sigmoid),
            Activation::Tanh => x.map(|v| v.tanh()),
            Activation::Softmax => softmax(x),
        }
    }

    /// Compute the derivative of the activation function
    pub fn backward(&self, x: &Matrix, grad_output: &Matrix) -> Matrix {
        match self {
            Activation::Linear => grad_output.clone(),
            Activation::ReLU => {
                let derivative = x.map(|v| if v > 0.0 { 1.0 } else { 0.0 });
                derivative.hadamard(grad_output).unwrap()
            }
            Activation::LeakyReLU(alpha) => {
                let alpha = *alpha;
                let derivative = x.map(|v| if v > 0.0 { 1.0 } else { alpha });
                derivative.hadamard(grad_output).unwrap()
            }
            Activation::Sigmoid => {
                // d/dx sigmoid(x) = sigmoid(x) * (1 - sigmoid(x))
                let s = x.map(sigmoid);
                let one_minus_s = s.map(|v| 1.0 - v);
                let derivative = s.hadamard(&one_minus_s).unwrap();
                derivative.hadamard(grad_output).unwrap()
            }
            Activation::Tanh => {
                // d/dx tanh(x) = 1 - tanh(x)^2
                let t = x.map(|v| v.tanh());
                let derivative = t.map(|v| 1.0 - v * v);
                derivative.hadamard(grad_output).unwrap()
            }
            Activation::Softmax => {
                // For softmax with cross-entropy loss, the gradient simplifies
                // This is handled in the loss function
                grad_output.clone()
            }
        }
    }
}

/// Sigmoid function: 1 / (1 + exp(-x))
fn sigmoid(x: f32) -> f32 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let exp_x = x.exp();
        exp_x / (1.0 + exp_x)
    }
}

/// Softmax function (applied row-wise)
fn softmax(x: &Matrix) -> Matrix {
    let mut result = Matrix::zeros(x.rows(), x.cols());

    for i in 0..x.rows() {
        // Find max for numerical stability
        let mut max_val = f32::NEG_INFINITY;
        for j in 0..x.cols() {
            max_val = max_val.max(x.get(i, j));
        }

        // Compute exp(x - max) and sum
        let mut sum = 0.0;
        let mut exps = vec![0.0; x.cols()];
        for j in 0..x.cols() {
            exps[j] = (x.get(i, j) - max_val).exp();
            sum += exps[j];
        }

        // Normalize
        for j in 0..x.cols() {
            result.set(i, j, exps[j] / sum);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relu() {
        let x = Matrix::from_vec(vec![vec![-1.0, 0.0, 1.0, 2.0]]);
        let y = Activation::ReLU.forward(&x);

        assert_eq!(y.get(0, 0), 0.0);
        assert_eq!(y.get(0, 1), 0.0);
        assert_eq!(y.get(0, 2), 1.0);
        assert_eq!(y.get(0, 3), 2.0);
    }

    #[test]
    fn test_sigmoid() {
        let x = Matrix::from_vec(vec![vec![0.0]]);
        let y = Activation::Sigmoid.forward(&x);
        assert!((y.get(0, 0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_softmax() {
        let x = Matrix::from_vec(vec![vec![1.0, 2.0, 3.0]]);
        let y = Activation::Softmax.forward(&x);

        // Sum should be 1
        let sum: f32 = (0..3).map(|j| y.get(0, j)).sum();
        assert!((sum - 1.0).abs() < 1e-6);

        // Largest input should have largest output
        assert!(y.get(0, 2) > y.get(0, 1));
        assert!(y.get(0, 1) > y.get(0, 0));
    }
}
