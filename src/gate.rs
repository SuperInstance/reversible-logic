//! Reversible logic gates: Identity, NOT, CNOT, Toffoli, Fredkin, Peres.
//!
//! Every gate is represented as a permutation matrix over `u8` bits.
//! Gates are the atomic building blocks of reversible circuits.

use serde::{Deserialize, Serialize};

/// A reversible logic gate.
///
/// Each variant preserves information — the mapping from inputs to outputs
/// is a bijection (permutation). This is the fundamental requirement of
/// reversible computing (Landauer's principle: erasing information costs energy).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReversibleGate {
    /// Identity gate: passes all bits through unchanged.
    /// Arity: any ≥ 1. Represented here with explicit width.
    Identity { width: usize },
    /// NOT gate: flips a single bit. Arity: 1.
    Not,
    /// Controlled-NOT (CNOT / Feynman gate). Arity: 2.
    /// Input [c, t] → Output [c, t ⊕ c].
    Cnot,
    /// Toffoli gate (CCNOT). Arity: 3.
    /// Input [c1, c2, t] → Output [c1, c2, t ⊕ (c1 ∧ c2)].
    Toffoli,
    /// Fredkin gate (CSWAP). Arity: 3.
    /// Input [c, x, y] → Output [c, x', y'] where if c=1, swap x↔y.
    Fredkin,
    /// Peres gate. Arity: 3.
    /// Input [c1, c2, t] → Output [c1, c2⊕c1, t⊕(c1∧c2)].
    Peres,
}

impl ReversibleGate {
    /// Return the number of bits (arity) this gate operates on.
    pub fn width(&self) -> usize {
        match self {
            ReversibleGate::Identity { width } => *width,
            ReversibleGate::Not => 1,
            ReversibleGate::Cnot => 2,
            ReversibleGate::Toffoli | ReversibleGate::Fredkin | ReversibleGate::Peres => 3,
        }
    }

    /// Apply this gate to a bit vector, returning the result.
    ///
    /// # Panics
    /// Panics if `input.len() != self.width()`.
    pub fn apply(&self, input: &[u8]) -> Vec<u8> {
        assert_eq!(
            input.len(),
            self.width(),
            "Input width mismatch: expected {}, got {}",
            self.width(),
            input.len()
        );
        match self {
            ReversibleGate::Identity { .. } => input.to_vec(),
            ReversibleGate::Not => vec![input[0] ^ 1],
            ReversibleGate::Cnot => vec![input[0], input[1] ^ input[0]],
            ReversibleGate::Toffoli => vec![input[0], input[1], input[2] ^ (input[0] & input[1])],
            ReversibleGate::Fredkin => {
                if input[0] == 1 {
                    vec![input[0], input[2], input[1]]
                } else {
                    input.to_vec()
                }
            }
            ReversibleGate::Peres => vec![
                input[0],
                input[1] ^ input[0],
                input[2] ^ (input[0] & input[1]),
            ],
        }
    }

    /// Generate the full truth table for this gate.
    ///
    /// Returns a vector of (input, output) pairs, one for each possible input
    /// over `width` bits (2^width entries).
    pub fn truth_table(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let n = self.width();
        let rows = 1usize << n;
        let mut table = Vec::with_capacity(rows);
        for i in 0..rows {
            let input: Vec<u8> = (0..n).rev().map(|b| ((i >> b) & 1) as u8).collect();
            let output = self.apply(&input);
            table.push((input, output));
        }
        table
    }

    /// Return the permutation matrix of this gate as a Vec<Vec<u8>>.
    ///
    /// The matrix has dimensions 2^width × 2^width. Entry `[i][j] = 1` means
    /// input pattern `i` maps to output pattern `j`.
    pub fn permutation_matrix(&self) -> Vec<Vec<u8>> {
        let n = 1usize << self.width();
        let mut matrix = vec![vec![0u8; n]; n];
        for (input, output) in self.truth_table() {
            let i = bits_to_index(&input);
            let j = bits_to_index(&output);
            matrix[i][j] = 1;
        }
        matrix
    }

    /// Return the quantum cost of this gate.
    ///
    /// Quantum cost counts the number of elementary (1-qubit or CNOT) gates
    /// needed to decompose this gate in a quantum circuit.
    ///
    /// | Gate     | Cost |
    /// |----------|------|
    /// | Identity | 0    |
    /// | NOT      | 1    |
    /// | CNOT     | 1    |
    /// | Toffoli  | 5    |
    /// | Fredkin  | 5    |
    /// | Peres    | 4    |
    pub fn quantum_cost(&self) -> usize {
        match self {
            ReversibleGate::Identity { .. } => 0,
            ReversibleGate::Not => 1,
            ReversibleGate::Cnot => 1,
            ReversibleGate::Toffoli => 5,
            ReversibleGate::Fredkin => 5,
            ReversibleGate::Peres => 4,
        }
    }

    /// Return the human-readable name of this gate.
    pub fn name(&self) -> &'static str {
        match self {
            ReversibleGate::Identity { .. } => "Identity",
            ReversibleGate::Not => "NOT",
            ReversibleGate::Cnot => "CNOT",
            ReversibleGate::Toffoli => "Toffoli",
            ReversibleGate::Fredkin => "Fredkin",
            ReversibleGate::Peres => "Peres",
        }
    }
}

/// Convert a bit vector to a decimal index (MSB first).
pub fn bits_to_index(bits: &[u8]) -> usize {
    bits.iter()
        .fold(0usize, |acc, &b| (acc << 1) | (b as usize))
}

/// Convert a decimal index to a bit vector of given width (MSB first).
pub fn index_to_bits(index: usize, width: usize) -> Vec<u8> {
    (0..width).rev().map(|b| ((index >> b) & 1) as u8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_preserves() {
        let g = ReversibleGate::Identity { width: 3 };
        assert_eq!(g.apply(&[1, 0, 1]), vec![1, 0, 1]);
    }

    #[test]
    fn not_flips() {
        let g = ReversibleGate::Not;
        assert_eq!(g.apply(&[0]), vec![1]);
        assert_eq!(g.apply(&[1]), vec![0]);
    }

    #[test]
    fn cnot_table() {
        let g = ReversibleGate::Cnot;
        let tt = g.truth_table();
        assert_eq!(tt.len(), 4);
        assert_eq!(g.apply(&[0, 0]), vec![0, 0]);
        assert_eq!(g.apply(&[0, 1]), vec![0, 1]);
        assert_eq!(g.apply(&[1, 0]), vec![1, 1]);
        assert_eq!(g.apply(&[1, 1]), vec![1, 0]);
    }

    #[test]
    fn toffoli_is_ccnot() {
        let g = ReversibleGate::Toffoli;
        // Only flips target when both controls are 1
        assert_eq!(g.apply(&[1, 1, 0]), vec![1, 1, 1]);
        assert_eq!(g.apply(&[1, 1, 1]), vec![1, 1, 0]);
        assert_eq!(g.apply(&[1, 0, 1]), vec![1, 0, 1]);
        assert_eq!(g.apply(&[0, 1, 1]), vec![0, 1, 1]);
    }

    #[test]
    fn fredkin_swaps_on_control() {
        let g = ReversibleGate::Fredkin;
        assert_eq!(g.apply(&[1, 0, 1]), vec![1, 1, 0]);
        assert_eq!(g.apply(&[0, 0, 1]), vec![0, 0, 1]);
    }

    #[test]
    fn peres_combined() {
        let g = ReversibleGate::Peres;
        // [1,1,0] -> [1, 1⊕1=0, 0⊕1=1] = [1,0,1]
        assert_eq!(g.apply(&[1, 1, 0]), vec![1, 0, 1]);
    }

    #[test]
    fn permutation_matrix_is_permutation() {
        for gate in &[
            ReversibleGate::Not,
            ReversibleGate::Cnot,
            ReversibleGate::Toffoli,
            ReversibleGate::Fredkin,
            ReversibleGate::Peres,
        ] {
            let m = gate.permutation_matrix();
            let n = m.len();
            for row in &m {
                assert_eq!(row.iter().sum::<u8>(), 1, "Each row must sum to 1");
            }
            for col in 0..n {
                let col_sum: u8 = (0..n).map(|r| m[r][col]).sum();
                assert_eq!(col_sum, 1, "Each column must sum to 1");
            }
        }
    }

    #[test]
    fn quantum_costs() {
        assert_eq!(ReversibleGate::Identity { width: 2 }.quantum_cost(), 0);
        assert_eq!(ReversibleGate::Not.quantum_cost(), 1);
        assert_eq!(ReversibleGate::Cnot.quantum_cost(), 1);
        assert_eq!(ReversibleGate::Toffoli.quantum_cost(), 5);
        assert_eq!(ReversibleGate::Fredkin.quantum_cost(), 5);
        assert_eq!(ReversibleGate::Peres.quantum_cost(), 4);
    }

    #[test]
    fn double_apply_is_identity() {
        // Peres gate is NOT self-inverse, so we exclude it
        let gates = [
            ReversibleGate::Not,
            ReversibleGate::Cnot,
            ReversibleGate::Toffoli,
            ReversibleGate::Fredkin,
        ];
        for g in &gates {
            let tt = g.truth_table();
            for (input, _) in &tt {
                let once = g.apply(input);
                let twice = g.apply(&once);
                assert_eq!(
                    *input,
                    twice,
                    "Gate {} not self-inverse on {:?}",
                    g.name(),
                    input
                );
            }
        }
    }

    #[test]
    #[should_panic]
    fn wrong_width_panics() {
        ReversibleGate::Cnot.apply(&[0]);
    }

    #[test]
    fn bits_roundtrip() {
        for i in 0..8 {
            let bits = index_to_bits(i, 3);
            assert_eq!(bits_to_index(&bits), i);
        }
    }
}
