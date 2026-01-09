//! LSTM (Long Short-Term Memory) implementation
//!
//! A custom LSTM implementation for time-series price prediction.
//!
//! # LSTM Equations
//!
//! ```text
//! f_t = σ(W_f · [h_{t-1}, x_t] + b_f)  // Forget gate
//! i_t = σ(W_i · [h_{t-1}, x_t] + b_i)  // Input gate
//! c̃_t = tanh(W_c · [h_{t-1}, x_t] + b_c)  // Candidate
//! c_t = f_t ⊙ c_{t-1} + i_t ⊙ c̃_t  // Cell state
//! o_t = σ(W_o · [h_{t-1}, x_t] + b_o)  // Output gate
//! h_t = o_t ⊙ tanh(c_t)  // Hidden state
//! ```

use crate::matrix::Matrix;
use crate::{MlError, MlResult};

/// Sigmoid activation function
fn sigmoid(x: f32) -> f32 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let exp_x = x.exp();
        exp_x / (1.0 + exp_x)
    }
}

/// LSTM state containing hidden and cell states
#[derive(Debug, Clone)]
pub struct LSTMState {
    /// Hidden state h_t
    pub hidden: Matrix,
    /// Cell state c_t
    pub cell: Matrix,
}

impl LSTMState {
    /// Create a new zero-initialized state
    pub fn zeros(hidden_size: usize) -> Self {
        Self {
            hidden: Matrix::zeros(hidden_size, 1),
            cell: Matrix::zeros(hidden_size, 1),
        }
    }
}

/// LSTM Cell - processes a single timestep
#[derive(Debug, Clone)]
pub struct LSTMCell {
    /// Input size
    input_size: usize,
    /// Hidden state size
    hidden_size: usize,
    /// Forget gate weights
    w_f: Matrix,
    /// Forget gate bias
    b_f: Matrix,
    /// Input gate weights
    w_i: Matrix,
    /// Input gate bias
    b_i: Matrix,
    /// Candidate weights
    w_c: Matrix,
    /// Candidate bias
    b_c: Matrix,
    /// Output gate weights
    w_o: Matrix,
    /// Output gate bias
    b_o: Matrix,
}

impl LSTMCell {
    /// Create a new LSTM cell with Xavier initialization
    pub fn new(input_size: usize, hidden_size: usize) -> Self {
        let combined_size = input_size + hidden_size;

        Self {
            input_size,
            hidden_size,
            // Forget gate
            w_f: Matrix::random_xavier(hidden_size, combined_size),
            b_f: Matrix::zeros(hidden_size, 1),
            // Input gate
            w_i: Matrix::random_xavier(hidden_size, combined_size),
            b_i: Matrix::zeros(hidden_size, 1),
            // Candidate
            w_c: Matrix::random_xavier(hidden_size, combined_size),
            b_c: Matrix::zeros(hidden_size, 1),
            // Output gate
            w_o: Matrix::random_xavier(hidden_size, combined_size),
            b_o: Matrix::zeros(hidden_size, 1),
        }
    }

    /// Get input size
    pub fn input_size(&self) -> usize {
        self.input_size
    }

    /// Get hidden size
    pub fn hidden_size(&self) -> usize {
        self.hidden_size
    }

    /// Forward pass through the LSTM cell
    pub fn forward(&self, input: &Matrix, prev_state: &LSTMState) -> MlResult<LSTMState> {
        // Concatenate [h_{t-1}, x_t]
        let combined = prev_state.hidden.concat_vertical(input)?;

        // Forget gate: f_t = σ(W_f · combined + b_f)
        let f_linear = self.w_f.matmul(&combined)?;
        let f_biased = add_column_bias(&f_linear, &self.b_f)?;
        let f_t = f_biased.map(sigmoid);

        // Input gate: i_t = σ(W_i · combined + b_i)
        let i_linear = self.w_i.matmul(&combined)?;
        let i_biased = add_column_bias(&i_linear, &self.b_i)?;
        let i_t = i_biased.map(sigmoid);

        // Candidate: c̃_t = tanh(W_c · combined + b_c)
        let c_linear = self.w_c.matmul(&combined)?;
        let c_biased = add_column_bias(&c_linear, &self.b_c)?;
        let c_tilde = c_biased.map(|x| x.tanh());

        // Cell state: c_t = f_t ⊙ c_{t-1} + i_t ⊙ c̃_t
        let forget_term = f_t.hadamard(&prev_state.cell)?;
        let input_term = i_t.hadamard(&c_tilde)?;
        let cell = forget_term.add(&input_term)?;

        // Output gate: o_t = σ(W_o · combined + b_o)
        let o_linear = self.w_o.matmul(&combined)?;
        let o_biased = add_column_bias(&o_linear, &self.b_o)?;
        let o_t = o_biased.map(sigmoid);

        // Hidden state: h_t = o_t ⊙ tanh(c_t)
        let cell_tanh = cell.map(|x| x.tanh());
        let hidden = o_t.hadamard(&cell_tanh)?;

        Ok(LSTMState { hidden, cell })
    }

    /// Initialize state for this cell
    pub fn init_state(&self) -> LSTMState {
        LSTMState::zeros(self.hidden_size)
    }
}

/// LSTM layer - processes sequences
#[derive(Debug, Clone)]
pub struct LSTM {
    /// LSTM cells for each layer
    cells: Vec<LSTMCell>,
    /// Number of layers
    num_layers: usize,
    /// Hidden size
    hidden_size: usize,
}

impl LSTM {
    /// Create a new multi-layer LSTM
    pub fn new(input_size: usize, hidden_size: usize, num_layers: usize) -> Self {
        let mut cells = Vec::with_capacity(num_layers);

        // First layer takes input_size
        cells.push(LSTMCell::new(input_size, hidden_size));

        // Subsequent layers take hidden_size
        for _ in 1..num_layers {
            cells.push(LSTMCell::new(hidden_size, hidden_size));
        }

        Self {
            cells,
            num_layers,
            hidden_size,
        }
    }

    /// Get hidden size
    pub fn hidden_size(&self) -> usize {
        self.hidden_size
    }

    /// Get number of layers
    pub fn num_layers(&self) -> usize {
        self.num_layers
    }

    /// Process a sequence and return the final hidden state
    pub fn forward(&self, sequence: &[Matrix]) -> MlResult<Matrix> {
        if sequence.is_empty() {
            return Ok(Matrix::zeros(self.hidden_size, 1));
        }

        // Initialize states for all layers
        let mut states: Vec<LSTMState> = self.cells.iter().map(|c| c.init_state()).collect();

        // Process each timestep
        for input in sequence {
            let mut layer_input = input.clone();

            // Process through all layers
            for (i, cell) in self.cells.iter().enumerate() {
                states[i] = cell.forward(&layer_input, &states[i])?;
                layer_input = states[i].hidden.clone();
            }
        }

        // Return final hidden state from last layer
        Ok(states.last().unwrap().hidden.clone())
    }

    /// Process a sequence and return all hidden states (for attention mechanisms)
    pub fn forward_all(&self, sequence: &[Matrix]) -> MlResult<Vec<Matrix>> {
        if sequence.is_empty() {
            return Ok(Vec::new());
        }

        // Initialize states for all layers
        let mut states: Vec<LSTMState> = self.cells.iter().map(|c| c.init_state()).collect();
        let mut outputs = Vec::with_capacity(sequence.len());

        // Process each timestep
        for input in sequence {
            let mut layer_input = input.clone();

            // Process through all layers
            for (i, cell) in self.cells.iter().enumerate() {
                states[i] = cell.forward(&layer_input, &states[i])?;
                layer_input = states[i].hidden.clone();
            }

            // Collect output from last layer
            outputs.push(states.last().unwrap().hidden.clone());
        }

        Ok(outputs)
    }

    /// Forward with initial state
    pub fn forward_with_state(
        &self,
        sequence: &[Matrix],
        initial_states: Vec<LSTMState>,
    ) -> MlResult<(Matrix, Vec<LSTMState>)> {
        if sequence.is_empty() {
            return Ok((Matrix::zeros(self.hidden_size, 1), initial_states));
        }

        let mut states = initial_states;

        // Process each timestep
        for input in sequence {
            let mut layer_input = input.clone();

            for (i, cell) in self.cells.iter().enumerate() {
                states[i] = cell.forward(&layer_input, &states[i])?;
                layer_input = states[i].hidden.clone();
            }
        }

        let final_hidden = states.last().unwrap().hidden.clone();
        Ok((final_hidden, states))
    }
}

/// Add column bias to a column vector
fn add_column_bias(x: &Matrix, bias: &Matrix) -> MlResult<Matrix> {
    if x.rows() != bias.rows() || x.cols() != 1 || bias.cols() != 1 {
        return Err(MlError::DimensionMismatch {
            expected: (bias.rows(), 1),
            actual: x.shape(),
        });
    }
    x.add(bias)
}

/// LSTM for price prediction with output layer
#[derive(Debug)]
pub struct PriceLSTM {
    /// The LSTM layers
    lstm: LSTM,
    /// Output projection weights
    output_weights: Matrix,
    /// Output bias
    output_bias: Matrix,
}

impl PriceLSTM {
    /// Create a new price prediction LSTM
    pub fn new(input_size: usize, hidden_size: usize, num_layers: usize, output_size: usize) -> Self {
        Self {
            lstm: LSTM::new(input_size, hidden_size, num_layers),
            output_weights: Matrix::random_xavier(output_size, hidden_size),
            output_bias: Matrix::zeros(output_size, 1),
        }
    }

    /// Predict price change from a sequence of features
    pub fn predict(&self, sequence: &[Matrix]) -> MlResult<Matrix> {
        let hidden = self.lstm.forward(sequence)?;
        let output = self.output_weights.matmul(&hidden)?;
        add_column_bias(&output, &self.output_bias)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lstm_cell_creation() {
        let cell = LSTMCell::new(10, 20);
        assert_eq!(cell.input_size(), 10);
        assert_eq!(cell.hidden_size(), 20);
    }

    #[test]
    fn test_lstm_cell_forward() {
        let cell = LSTMCell::new(10, 20);
        let input = Matrix::zeros(10, 1);
        let state = cell.init_state();

        let new_state = cell.forward(&input, &state).unwrap();

        assert_eq!(new_state.hidden.rows(), 20);
        assert_eq!(new_state.hidden.cols(), 1);
        assert_eq!(new_state.cell.rows(), 20);
        assert_eq!(new_state.cell.cols(), 1);
    }

    #[test]
    fn test_lstm_sequence() {
        let lstm = LSTM::new(10, 20, 2);

        let sequence: Vec<Matrix> = (0..5).map(|_| Matrix::zeros(10, 1)).collect();

        let output = lstm.forward(&sequence).unwrap();

        assert_eq!(output.rows(), 20);
        assert_eq!(output.cols(), 1);
    }

    #[test]
    fn test_lstm_forward_all() {
        let lstm = LSTM::new(5, 10, 1);

        let sequence: Vec<Matrix> = (0..3).map(|_| Matrix::zeros(5, 1)).collect();

        let outputs = lstm.forward_all(&sequence).unwrap();

        assert_eq!(outputs.len(), 3);
        for output in outputs {
            assert_eq!(output.rows(), 10);
            assert_eq!(output.cols(), 1);
        }
    }

    #[test]
    fn test_lstm_empty_sequence() {
        let lstm = LSTM::new(5, 10, 2);

        let output = lstm.forward(&[]).unwrap();

        assert_eq!(output.rows(), 10);
        assert_eq!(output.cols(), 1);
    }

    #[test]
    fn test_price_lstm() {
        let model = PriceLSTM::new(5, 10, 2, 1);

        let sequence: Vec<Matrix> = (0..10).map(|_| Matrix::zeros(5, 1)).collect();

        let prediction = model.predict(&sequence).unwrap();

        assert_eq!(prediction.rows(), 1);
        assert_eq!(prediction.cols(), 1);
    }

    #[test]
    fn test_lstm_state() {
        let state = LSTMState::zeros(15);
        assert_eq!(state.hidden.rows(), 15);
        assert_eq!(state.cell.rows(), 15);
    }

    #[test]
    fn test_multi_layer_lstm() {
        let lstm = LSTM::new(8, 16, 3);

        assert_eq!(lstm.num_layers(), 3);
        assert_eq!(lstm.hidden_size(), 16);

        let sequence: Vec<Matrix> = (0..4).map(|_| Matrix::zeros(8, 1)).collect();
        let output = lstm.forward(&sequence).unwrap();

        assert_eq!(output.rows(), 16);
    }
}
