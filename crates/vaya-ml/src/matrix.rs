//! Matrix operations for neural network computations
//!
//! A custom matrix implementation optimized for ML workloads.

use crate::{MlError, MlResult};
use rand::Rng;
use std::ops::{Add, Mul, Sub};

/// A 2D matrix of f32 values
#[derive(Debug, Clone, PartialEq)]
pub struct Matrix {
    /// Matrix data in row-major order
    data: Vec<f32>,
    /// Number of rows
    rows: usize,
    /// Number of columns
    cols: usize,
}

impl Matrix {
    /// Create a new matrix filled with zeros
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![0.0; rows * cols],
            rows,
            cols,
        }
    }

    /// Create a new matrix filled with ones
    pub fn ones(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![1.0; rows * cols],
            rows,
            cols,
        }
    }

    /// Create a matrix from a 2D vector
    pub fn from_vec(data: Vec<Vec<f32>>) -> Self {
        let rows = data.len();
        let cols = if rows > 0 { data[0].len() } else { 0 };
        let flat: Vec<f32> = data.into_iter().flatten().collect();
        Self {
            data: flat,
            rows,
            cols,
        }
    }

    /// Create a matrix from a flat vector with given dimensions
    pub fn from_flat(data: Vec<f32>, rows: usize, cols: usize) -> MlResult<Self> {
        if data.len() != rows * cols {
            return Err(MlError::DimensionMismatch {
                expected: (rows, cols),
                actual: (data.len(), 1),
            });
        }
        Ok(Self { data, rows, cols })
    }

    /// Create a column vector from a slice
    pub fn from_slice(data: &[f32]) -> Self {
        Self {
            data: data.to_vec(),
            rows: data.len(),
            cols: 1,
        }
    }

    /// Create a matrix with random values (Xavier initialization)
    pub fn random_xavier(rows: usize, cols: usize) -> Self {
        let mut rng = rand::thread_rng();
        let scale = (2.0 / (rows + cols) as f32).sqrt();
        let data: Vec<f32> = (0..rows * cols)
            .map(|_| rng.gen_range(-scale..scale))
            .collect();
        Self { data, rows, cols }
    }

    /// Create a matrix with random values (He initialization)
    pub fn random_he(rows: usize, cols: usize) -> Self {
        let mut rng = rand::thread_rng();
        let scale = (2.0 / rows as f32).sqrt();
        let data: Vec<f32> = (0..rows * cols)
            .map(|_| rng.gen_range(-scale..scale))
            .collect();
        Self { data, rows, cols }
    }

    /// Get the number of rows
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Get the number of columns
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Get the shape as (rows, cols)
    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// Get a reference to the underlying data
    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// Get a mutable reference to the underlying data
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }

    /// Get an element at (row, col)
    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.data[row * self.cols + col]
    }

    /// Set an element at (row, col)
    pub fn set(&mut self, row: usize, col: usize, value: f32) {
        self.data[row * self.cols + col] = value;
    }

    /// Matrix multiplication
    pub fn matmul(&self, other: &Matrix) -> MlResult<Matrix> {
        if self.cols != other.rows {
            return Err(MlError::DimensionMismatch {
                expected: (self.rows, other.cols),
                actual: (self.cols, other.rows),
            });
        }

        let mut result = Matrix::zeros(self.rows, other.cols);

        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self.get(i, k) * other.get(k, j);
                }
                result.set(i, j, sum);
            }
        }

        Ok(result)
    }

    /// Transpose the matrix
    pub fn transpose(&self) -> Matrix {
        let mut result = Matrix::zeros(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.set(j, i, self.get(i, j));
            }
        }
        result
    }

    /// Element-wise addition
    pub fn add(&self, other: &Matrix) -> MlResult<Matrix> {
        if self.shape() != other.shape() {
            return Err(MlError::DimensionMismatch {
                expected: self.shape(),
                actual: other.shape(),
            });
        }

        let data: Vec<f32> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a + b)
            .collect();

        Ok(Matrix {
            data,
            rows: self.rows,
            cols: self.cols,
        })
    }

    /// Element-wise subtraction
    pub fn sub(&self, other: &Matrix) -> MlResult<Matrix> {
        if self.shape() != other.shape() {
            return Err(MlError::DimensionMismatch {
                expected: self.shape(),
                actual: other.shape(),
            });
        }

        let data: Vec<f32> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a - b)
            .collect();

        Ok(Matrix {
            data,
            rows: self.rows,
            cols: self.cols,
        })
    }

    /// Element-wise multiplication (Hadamard product)
    pub fn hadamard(&self, other: &Matrix) -> MlResult<Matrix> {
        if self.shape() != other.shape() {
            return Err(MlError::DimensionMismatch {
                expected: self.shape(),
                actual: other.shape(),
            });
        }

        let data: Vec<f32> = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| a * b)
            .collect();

        Ok(Matrix {
            data,
            rows: self.rows,
            cols: self.cols,
        })
    }

    /// Scalar multiplication
    pub fn scale(&self, scalar: f32) -> Matrix {
        let data: Vec<f32> = self.data.iter().map(|x| x * scalar).collect();
        Matrix {
            data,
            rows: self.rows,
            cols: self.cols,
        }
    }

    /// Apply a function element-wise
    pub fn map<F>(&self, f: F) -> Matrix
    where
        F: Fn(f32) -> f32,
    {
        let data: Vec<f32> = self.data.iter().map(|x| f(*x)).collect();
        Matrix {
            data,
            rows: self.rows,
            cols: self.cols,
        }
    }

    /// Sum all elements
    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    /// Mean of all elements
    pub fn mean(&self) -> f32 {
        self.sum() / self.data.len() as f32
    }

    /// Sum along axis (0 = columns, 1 = rows)
    pub fn sum_axis(&self, axis: usize) -> Matrix {
        match axis {
            0 => {
                // Sum along columns (result is 1 x cols)
                let mut result = Matrix::zeros(1, self.cols);
                for j in 0..self.cols {
                    let mut sum = 0.0;
                    for i in 0..self.rows {
                        sum += self.get(i, j);
                    }
                    result.set(0, j, sum);
                }
                result
            }
            1 => {
                // Sum along rows (result is rows x 1)
                let mut result = Matrix::zeros(self.rows, 1);
                for i in 0..self.rows {
                    let mut sum = 0.0;
                    for j in 0..self.cols {
                        sum += self.get(i, j);
                    }
                    result.set(i, 0, sum);
                }
                result
            }
            _ => panic!("Invalid axis: {}", axis),
        }
    }

    /// Add a bias vector (broadcast along rows)
    pub fn add_bias(&self, bias: &Matrix) -> MlResult<Matrix> {
        if bias.cols != self.cols || bias.rows != 1 {
            return Err(MlError::DimensionMismatch {
                expected: (1, self.cols),
                actual: bias.shape(),
            });
        }

        let mut result = self.clone();
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.set(i, j, self.get(i, j) + bias.get(0, j));
            }
        }
        Ok(result)
    }

    /// Clip values to a range
    pub fn clip(&self, min: f32, max: f32) -> Matrix {
        self.map(|x| x.max(min).min(max))
    }
}

impl Add for &Matrix {
    type Output = MlResult<Matrix>;

    fn add(self, other: &Matrix) -> MlResult<Matrix> {
        self.add(other)
    }
}

impl Sub for &Matrix {
    type Output = MlResult<Matrix>;

    fn sub(self, other: &Matrix) -> MlResult<Matrix> {
        self.sub(other)
    }
}

impl Mul<f32> for &Matrix {
    type Output = Matrix;

    fn mul(self, scalar: f32) -> Matrix {
        self.scale(scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_creation() {
        let m = Matrix::zeros(2, 3);
        assert_eq!(m.shape(), (2, 3));
        assert_eq!(m.get(0, 0), 0.0);

        let m = Matrix::ones(2, 2);
        assert_eq!(m.get(1, 1), 1.0);
    }

    #[test]
    fn test_matrix_from_vec() {
        let m = Matrix::from_vec(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        assert_eq!(m.shape(), (2, 2));
        assert_eq!(m.get(0, 0), 1.0);
        assert_eq!(m.get(1, 1), 4.0);
    }

    #[test]
    fn test_matmul() {
        let a = Matrix::from_vec(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        let b = Matrix::from_vec(vec![vec![5.0, 6.0], vec![7.0, 8.0]]);
        let c = a.matmul(&b).unwrap();

        assert_eq!(c.get(0, 0), 19.0); // 1*5 + 2*7
        assert_eq!(c.get(0, 1), 22.0); // 1*6 + 2*8
        assert_eq!(c.get(1, 0), 43.0); // 3*5 + 4*7
        assert_eq!(c.get(1, 1), 50.0); // 3*6 + 4*8
    }

    #[test]
    fn test_transpose() {
        let a = Matrix::from_vec(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);
        let t = a.transpose();

        assert_eq!(t.shape(), (3, 2));
        assert_eq!(t.get(0, 0), 1.0);
        assert_eq!(t.get(0, 1), 4.0);
        assert_eq!(t.get(2, 1), 6.0);
    }

    #[test]
    fn test_hadamard() {
        let a = Matrix::from_vec(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        let b = Matrix::from_vec(vec![vec![2.0, 3.0], vec![4.0, 5.0]]);
        let c = a.hadamard(&b).unwrap();

        assert_eq!(c.get(0, 0), 2.0);
        assert_eq!(c.get(1, 1), 20.0);
    }

    #[test]
    fn test_sum_axis() {
        let m = Matrix::from_vec(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]);

        let col_sum = m.sum_axis(0);
        assert_eq!(col_sum.shape(), (1, 3));
        assert_eq!(col_sum.get(0, 0), 5.0);
        assert_eq!(col_sum.get(0, 1), 7.0);
        assert_eq!(col_sum.get(0, 2), 9.0);

        let row_sum = m.sum_axis(1);
        assert_eq!(row_sum.shape(), (2, 1));
        assert_eq!(row_sum.get(0, 0), 6.0);
        assert_eq!(row_sum.get(1, 0), 15.0);
    }
}
