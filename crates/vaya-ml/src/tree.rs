//! Decision tree implementation for gradient boosting

use crate::matrix::Matrix;
use crate::MlResult;

/// A decision tree node
#[derive(Debug, Clone)]
enum TreeNode {
    /// Internal node with a split
    Split {
        feature_index: usize,
        threshold: f32,
        left: usize,
        right: usize,
    },
    /// Leaf node with a prediction value
    Leaf { value: f32 },
}

/// A decision tree regressor
#[derive(Debug, Clone)]
pub struct DecisionTree {
    /// Tree nodes stored in a vector
    nodes: Vec<TreeNode>,
    /// Maximum depth of the tree
    max_depth: usize,
    /// Minimum samples required to split
    min_samples_split: usize,
}

impl DecisionTree {
    /// Create a new decision tree
    pub fn new(max_depth: usize, min_samples_split: usize) -> Self {
        Self {
            nodes: Vec::new(),
            max_depth,
            min_samples_split,
        }
    }

    /// Fit the tree to data
    pub fn fit(&mut self, x: &Matrix, y: &Matrix) -> MlResult<()> {
        self.nodes.clear();

        // Collect all indices
        let indices: Vec<usize> = (0..x.rows()).collect();

        // Build tree recursively
        self.build_tree(x, y, &indices, 0);

        Ok(())
    }

    /// Predict values for input data
    pub fn predict(&self, x: &Matrix) -> Matrix {
        let mut predictions = Matrix::zeros(x.rows(), 1);

        for i in 0..x.rows() {
            let features: Vec<f32> = (0..x.cols()).map(|j| x.get(i, j)).collect();
            let pred = self.predict_single(&features);
            predictions.set(i, 0, pred);
        }

        predictions
    }

    /// Predict a single sample
    fn predict_single(&self, features: &[f32]) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }

        let mut node_idx = 0;

        loop {
            match &self.nodes[node_idx] {
                TreeNode::Leaf { value } => return *value,
                TreeNode::Split {
                    feature_index,
                    threshold,
                    left,
                    right,
                } => {
                    if features[*feature_index] <= *threshold {
                        node_idx = *left;
                    } else {
                        node_idx = *right;
                    }
                }
            }
        }
    }

    /// Build the tree recursively
    fn build_tree(&mut self, x: &Matrix, y: &Matrix, indices: &[usize], depth: usize) -> usize {
        let node_idx = self.nodes.len();

        // Check stopping conditions
        if depth >= self.max_depth
            || indices.len() < self.min_samples_split
            || self.is_pure(y, indices)
        {
            // Create leaf node
            let value = self.calculate_mean(y, indices);
            self.nodes.push(TreeNode::Leaf { value });
            return node_idx;
        }

        // Find best split
        if let Some((feature_idx, threshold, left_indices, right_indices)) =
            self.find_best_split(x, y, indices)
        {
            // Placeholder for children indices
            self.nodes.push(TreeNode::Leaf { value: 0.0 });

            // Build left and right subtrees
            let left_idx = self.build_tree(x, y, &left_indices, depth + 1);
            let right_idx = self.build_tree(x, y, &right_indices, depth + 1);

            // Update current node to be a split node
            self.nodes[node_idx] = TreeNode::Split {
                feature_index: feature_idx,
                threshold,
                left: left_idx,
                right: right_idx,
            };

            node_idx
        } else {
            // No valid split found, create leaf
            let value = self.calculate_mean(y, indices);
            self.nodes.push(TreeNode::Leaf { value });
            node_idx
        }
    }

    /// Find the best split for the given indices
    fn find_best_split(
        &self,
        x: &Matrix,
        y: &Matrix,
        indices: &[usize],
    ) -> Option<(usize, f32, Vec<usize>, Vec<usize>)> {
        let mut best_gain = f32::NEG_INFINITY;
        let mut best_split = None;

        let n_features = x.cols();
        let parent_var = self.calculate_variance(y, indices);

        for feature_idx in 0..n_features {
            // Get unique values for this feature
            let mut values: Vec<f32> = indices.iter().map(|&i| x.get(i, feature_idx)).collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            values.dedup();

            // Try splits at midpoints between consecutive values
            for i in 0..values.len().saturating_sub(1) {
                let threshold = (values[i] + values[i + 1]) / 2.0;

                // Split indices
                let (left_indices, right_indices): (Vec<usize>, Vec<usize>) = indices
                    .iter()
                    .partition(|&&idx| x.get(idx, feature_idx) <= threshold);

                if left_indices.is_empty() || right_indices.is_empty() {
                    continue;
                }

                // Calculate information gain (variance reduction)
                let left_var = self.calculate_variance(y, &left_indices);
                let right_var = self.calculate_variance(y, &right_indices);

                let left_weight = left_indices.len() as f32 / indices.len() as f32;
                let right_weight = right_indices.len() as f32 / indices.len() as f32;

                let weighted_var = left_weight * left_var + right_weight * right_var;
                let gain = parent_var - weighted_var;

                if gain > best_gain {
                    best_gain = gain;
                    best_split = Some((feature_idx, threshold, left_indices, right_indices));
                }
            }
        }

        best_split
    }

    /// Calculate the mean of y values at given indices
    fn calculate_mean(&self, y: &Matrix, indices: &[usize]) -> f32 {
        if indices.is_empty() {
            return 0.0;
        }
        let sum: f32 = indices.iter().map(|&i| y.get(i, 0)).sum();
        sum / indices.len() as f32
    }

    /// Calculate the variance of y values at given indices
    fn calculate_variance(&self, y: &Matrix, indices: &[usize]) -> f32 {
        if indices.is_empty() {
            return 0.0;
        }
        let mean = self.calculate_mean(y, indices);
        let sum_sq: f32 = indices.iter().map(|&i| (y.get(i, 0) - mean).powi(2)).sum();
        sum_sq / indices.len() as f32
    }

    /// Check if all y values at given indices are the same
    fn is_pure(&self, y: &Matrix, indices: &[usize]) -> bool {
        if indices.is_empty() {
            return true;
        }
        let first = y.get(indices[0], 0);
        indices.iter().all(|&i| (y.get(i, 0) - first).abs() < 1e-8)
    }

    /// Get the number of nodes in the tree
    pub fn num_nodes(&self) -> usize {
        self.nodes.len()
    }
}

/// Gradient Boosting Regressor
#[derive(Debug, Clone)]
pub struct GradientBoostingRegressor {
    /// Ensemble of trees
    trees: Vec<DecisionTree>,
    /// Learning rate
    learning_rate: f32,
    /// Number of trees
    n_estimators: usize,
    /// Maximum depth per tree
    max_depth: usize,
    /// Initial prediction (mean of training data)
    initial_prediction: f32,
}

impl GradientBoostingRegressor {
    /// Create a new gradient boosting regressor
    pub fn new(n_estimators: usize, learning_rate: f32, max_depth: usize) -> Self {
        Self {
            trees: Vec::new(),
            learning_rate,
            n_estimators,
            max_depth,
            initial_prediction: 0.0,
        }
    }

    /// Fit the model to training data
    pub fn fit(&mut self, x: &Matrix, y: &Matrix) -> MlResult<()> {
        self.trees.clear();

        // Initial prediction is the mean
        self.initial_prediction = y.mean();

        // Current predictions
        let mut predictions = Matrix::zeros(y.rows(), 1);
        for i in 0..y.rows() {
            predictions.set(i, 0, self.initial_prediction);
        }

        for _ in 0..self.n_estimators {
            // Calculate residuals (negative gradient for MSE loss)
            let residuals = y.sub(&predictions)?;

            // Fit a tree to the residuals
            let mut tree = DecisionTree::new(self.max_depth, 2);
            tree.fit(x, &residuals)?;

            // Update predictions
            let tree_predictions = tree.predict(x);
            for i in 0..y.rows() {
                let new_pred =
                    predictions.get(i, 0) + self.learning_rate * tree_predictions.get(i, 0);
                predictions.set(i, 0, new_pred);
            }

            self.trees.push(tree);
        }

        Ok(())
    }

    /// Predict values for input data
    pub fn predict(&self, x: &Matrix) -> Matrix {
        let mut predictions = Matrix::zeros(x.rows(), 1);

        // Start with initial prediction
        for i in 0..x.rows() {
            predictions.set(i, 0, self.initial_prediction);
        }

        // Add contributions from each tree
        for tree in &self.trees {
            let tree_predictions = tree.predict(x);
            for i in 0..x.rows() {
                let new_pred =
                    predictions.get(i, 0) + self.learning_rate * tree_predictions.get(i, 0);
                predictions.set(i, 0, new_pred);
            }
        }

        predictions
    }

    /// Get the number of trees
    pub fn num_trees(&self) -> usize {
        self.trees.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_tree() {
        let x = Matrix::from_vec(vec![
            vec![1.0],
            vec![2.0],
            vec![3.0],
            vec![4.0],
            vec![5.0],
        ]);
        let y = Matrix::from_vec(vec![
            vec![1.0],
            vec![2.0],
            vec![3.0],
            vec![4.0],
            vec![5.0],
        ]);

        let mut tree = DecisionTree::new(3, 1);
        tree.fit(&x, &y).unwrap();

        let predictions = tree.predict(&x);
        assert_eq!(predictions.rows(), 5);
    }

    #[test]
    fn test_gradient_boosting() {
        let x = Matrix::from_vec(vec![
            vec![1.0],
            vec![2.0],
            vec![3.0],
            vec![4.0],
            vec![5.0],
        ]);
        let y = Matrix::from_vec(vec![
            vec![2.0],
            vec![4.0],
            vec![6.0],
            vec![8.0],
            vec![10.0],
        ]);

        let mut gbr = GradientBoostingRegressor::new(10, 0.1, 3);
        gbr.fit(&x, &y).unwrap();

        let predictions = gbr.predict(&x);

        // Predictions should be reasonably close to actual values
        for i in 0..5 {
            let pred = predictions.get(i, 0);
            let actual = y.get(i, 0);
            assert!((pred - actual).abs() < 2.0);
        }
    }
}
