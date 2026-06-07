//! Reversible circuits: ordered sequences of reversible gates.
//!
//! A circuit is a chain of gates applied in sequence. Because every gate is
//! reversible, the entire circuit is reversible — its permutation is the
//! composition of its gates' permutations.

use crate::gate::ReversibleGate;
use serde::{Deserialize, Serialize};

/// A reversible circuit: an ordered sequence of gates applied left-to-right.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReversibleCircuit {
    /// Ordered sequence of gates.
    gates: Vec<ReversibleGate>,
    /// Total bit-width of the circuit (all gates must be compatible).
    width: usize,
    /// Labels for which output lines are the "answer" vs garbage.
    answer_lines: Vec<usize>,
}

impl ReversibleCircuit {
    /// Create a circuit from a vector of gates with the given bit-width.
    ///
    /// Gates that are narrower than `width` are assumed to act on the first
    /// `gate.width()` bits (LSB-aligned), with remaining bits passed through.
    pub fn new(width: usize, gates: Vec<ReversibleGate>) -> Self {
        for g in &gates {
            assert!(
                g.width() <= width,
                "Gate width {} exceeds circuit width {}",
                g.width(),
                width
            );
        }
        ReversibleCircuit {
            gates,
            width,
            answer_lines: (0..width).collect(),
        }
    }

    /// Number of bits this circuit operates on.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Number of gates in this circuit.
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Total quantum cost (sum of individual gate costs).
    pub fn quantum_cost(&self) -> usize {
        self.gates.iter().map(|g| g.quantum_cost()).sum()
    }

    /// Reference to the gate list.
    pub fn gates(&self) -> &[ReversibleGate] {
        &self.gates
    }

    /// Apply the entire circuit to an input bit vector.
    ///
    /// For each gate, if `gate.width() < circuit.width()`, the gate is applied
    /// to the first `gate.width()` bits and remaining bits pass through.
    pub fn apply(&self, input: &[u8]) -> Vec<u8> {
        assert_eq!(input.len(), self.width, "Circuit width mismatch");
        let mut state = input.to_vec();
        for gate in &self.gates {
            let gw = gate.width();
            // Apply gate to first gw bits, pass through the rest
            let gate_output = gate.apply(&state[..gw]);
            state[..gw].copy_from_slice(&gate_output);
        }
        state
    }

    /// Mark which output lines carry the "answer" (non-garbage).
    pub fn set_answer_lines(&mut self, lines: Vec<usize>) {
        for &l in &lines {
            assert!(l < self.width, "Answer line {} out of range", l);
        }
        self.answer_lines = lines;
    }

    /// Indices of output lines that carry the answer.
    pub fn answer_lines(&self) -> &[usize] {
        &self.answer_lines
    }

    /// Number of garbage output lines (outputs that are NOT the answer).
    pub fn garbage_count(&self) -> usize {
        self.width - self.answer_lines.len()
    }

    /// Extract the answer bits from a full output vector.
    pub fn extract_answer(&self, output: &[u8]) -> Vec<u8> {
        self.answer_lines.iter().map(|&i| output[i]).collect()
    }

    /// Generate the full truth table: all 2^width input/output pairs.
    pub fn truth_table(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let rows = 1usize << self.width;
        let mut table = Vec::with_capacity(rows);
        for i in 0..rows {
            let input: Vec<u8> = (0..self.width)
                .rev()
                .map(|b| ((i >> b) & 1) as u8)
                .collect();
            let output = self.apply(&input);
            table.push((input, output));
        }
        table
    }
}

/// Fluent builder for constructing reversible circuits.
pub struct CircuitBuilder {
    width: usize,
    gates: Vec<ReversibleGate>,
    answer_lines: Option<Vec<usize>>,
}

impl CircuitBuilder {
    /// Start building a circuit with the given bit-width.
    pub fn new(width: usize) -> Self {
        CircuitBuilder {
            width,
            gates: Vec::new(),
            answer_lines: None,
        }
    }

    /// Add a gate to the circuit.
    pub fn gate(mut self, gate: ReversibleGate) -> Self {
        assert!(gate.width() <= self.width, "Gate wider than circuit");
        self.gates.push(gate);
        self
    }

    /// Add a NOT gate.
    pub fn add_not(self) -> Self {
        self.gate(ReversibleGate::Not)
    }

    /// Add a CNOT gate.
    pub fn cnot(self) -> Self {
        self.gate(ReversibleGate::Cnot)
    }

    /// Add a Toffoli gate.
    pub fn toffoli(self) -> Self {
        self.gate(ReversibleGate::Toffoli)
    }

    /// Add a Fredkin gate.
    pub fn fredkin(self) -> Self {
        self.gate(ReversibleGate::Fredkin)
    }

    /// Add a Peres gate.
    pub fn peres(self) -> Self {
        self.gate(ReversibleGate::Peres)
    }

    /// Specify which output lines carry the answer.
    pub fn answer_lines(mut self, lines: Vec<usize>) -> Self {
        self.answer_lines = Some(lines);
        self
    }

    /// Build the circuit.
    pub fn build(self) -> ReversibleCircuit {
        let mut circuit = ReversibleCircuit::new(self.width, self.gates);
        if let Some(lines) = self.answer_lines {
            circuit.set_answer_lines(lines);
        }
        circuit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_circuit_is_identity() {
        let c = ReversibleCircuit::new(3, vec![]);
        assert_eq!(c.apply(&[1, 0, 1]), vec![1, 0, 1]);
    }

    #[test]
    fn single_not_circuit() {
        let c = ReversibleCircuit::new(1, vec![ReversibleGate::Not]);
        assert_eq!(c.apply(&[0]), vec![1]);
    }

    #[test]
    fn cnot_then_not_circuit() {
        let c = CircuitBuilder::new(2).cnot().build();
        assert_eq!(c.apply(&[1, 0]), vec![1, 1]);
        assert_eq!(c.gate_count(), 1);
        assert_eq!(c.quantum_cost(), 1);
    }

    #[test]
    fn builder_toffoli_chain() {
        let c = CircuitBuilder::new(3).toffoli().toffoli().build();
        // Two Toffolis cancel
        assert_eq!(c.apply(&[1, 1, 0]), vec![1, 1, 0]);
        assert_eq!(c.gate_count(), 2);
        assert_eq!(c.quantum_cost(), 10);
    }

    #[test]
    fn garbage_tracking() {
        let mut c = ReversibleCircuit::new(3, vec![ReversibleGate::Toffoli]);
        c.set_answer_lines(vec![2]); // Only bit 2 is the answer
        assert_eq!(c.garbage_count(), 2);
        assert_eq!(c.extract_answer(&[1, 1, 1]), vec![1]);
    }

    #[test]
    fn circuit_truth_table() {
        let c = CircuitBuilder::new(2).cnot().build();
        let tt = c.truth_table();
        assert_eq!(tt.len(), 4);
    }

    #[test]
    fn double_circuit_inverse() {
        // CNOT composed with CNOT = Identity
        let c = CircuitBuilder::new(2).cnot().cnot().build();
        let tt = c.truth_table();
        for (input, output) in tt {
            assert_eq!(input, output);
        }
    }

    #[test]
    fn fredkin_circuit() {
        let c = CircuitBuilder::new(3).fredkin().build();
        assert_eq!(c.apply(&[1, 0, 1]), vec![1, 1, 0]);
    }
}
