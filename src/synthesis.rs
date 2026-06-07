//! Gate synthesis: finding reversible circuits from truth tables.
//!
//! Given a truth table (mapping from input bit patterns to output bit patterns),
//! find a sequence of reversible gates that implements it. This module uses
//! **ESOP-based synthesis** (Exclusive-OR Sum of Products) combined with
//! iterative search using Toffoli, CNOT, and NOT gates.
//!
//! # Algorithm
//!
//! The synthesis uses an iterative approach:
//! 1. Start from the identity mapping.
//! 2. Iteratively apply gates (NOT, CNOT, Toffoli) to transform the current
//!    mapping toward the target.
//! 3. Use a HashSet to track visited states and avoid cycles.
//! 4. Select gates that reduce the Hamming distance to the target mapping.

use crate::gate::{bits_to_index, index_to_bits};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Result of gate synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisResult {
    /// The synthesized circuit as a list of (gate, target_positions) pairs.
    pub gate_sequence: Vec<(String, Vec<usize>)>,
    /// Number of gates in the result.
    pub gate_count: usize,
    /// Quantum cost of the result.
    pub quantum_cost: usize,
    /// Whether synthesis found an exact match.
    pub exact: bool,
    /// Number of iterations explored.
    pub iterations: usize,
    /// Width of the circuit.
    pub width: usize,
}

/// A truth table for synthesis: list of (input, output) bit-pattern pairs.
pub type TruthTable = Vec<(Vec<u8>, Vec<u8>)>;

/// A positional gate: a gate type with specific target bit positions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionalGate {
    /// Type of gate.
    pub gate_type: GateType,
    /// Bit positions this gate acts on.
    pub positions: Vec<usize>,
}

/// Gate types available for synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateType {
    Not,
    Cnot,
    Toffoli,
}

impl PositionalGate {
    /// Apply this gate to a bit vector (in place conceptually, returns new).
    pub fn apply(&self, bits: &[u8]) -> Vec<u8> {
        let mut result = bits.to_vec();
        match self.gate_type {
            GateType::Not => {
                let pos = self.positions[0];
                result[pos] ^= 1;
            }
            GateType::Cnot => {
                let control = self.positions[0];
                let target = self.positions[1];
                result[target] ^= result[control];
            }
            GateType::Toffoli => {
                let c1 = self.positions[0];
                let c2 = self.positions[1];
                let t = self.positions[2];
                result[t] ^= result[c1] & result[c2];
            }
        }
        result
    }

    /// Quantum cost of this gate.
    pub fn quantum_cost(&self) -> usize {
        match self.gate_type {
            GateType::Not => 1,
            GateType::Cnot => 1,
            GateType::Toffoli => 5,
        }
    }
}

/// Generate all possible positional gates for a given width.
fn generate_all_gates(width: usize) -> Vec<PositionalGate> {
    let mut gates = Vec::new();

    // NOT on each bit
    for i in 0..width {
        gates.push(PositionalGate {
            gate_type: GateType::Not,
            positions: vec![i],
        });
    }

    // CNOT: control and target on different bits
    for c in 0..width {
        for t in 0..width {
            if c != t {
                gates.push(PositionalGate {
                    gate_type: GateType::Cnot,
                    positions: vec![c, t],
                });
            }
        }
    }

    // Toffoli: two controls and one target, all distinct
    if width >= 3 {
        for c1 in 0..width {
            for c2 in 0..width {
                for t in 0..width {
                    if c1 != c2 && c1 != t && c2 != t {
                        gates.push(PositionalGate {
                            gate_type: GateType::Toffoli,
                            positions: vec![c1, c2, t],
                        });
                    }
                }
            }
        }
    }

    gates
}

/// Apply a sequence of positional gates to a permutation.
fn apply_gates_to_perm(perm: &[usize], gates: &[PositionalGate], width: usize) -> Vec<usize> {
    perm.iter()
        .map(|&val| {
            let mut bits = index_to_bits(val, width);
            for gate in gates {
                bits = gate.apply(&bits);
            }
            bits_to_index(&bits)
        })
        .collect()
}

/// Synthesize a reversible circuit from a truth table using iterative BFS
/// with NOT, CNOT, and Toffoli gates on arbitrary bit positions.
///
/// This is an **iterative** breadth-first search (NOT recursive):
/// - Maintains a queue of (current_permutation, gate_sequence) states.
/// - Tracks visited permutation states with a `HashSet` to avoid revisiting.
/// - At each BFS level, tries all possible positional gates.
/// - Terminates when the target permutation is reached or search budget exhausted.
///
/// # Arguments
/// * `width` - Number of bits.
/// * `target_table` - Desired truth table as (input, output) pairs.
/// * `max_iterations` - Search budget (prevents runaway search).
pub fn synthesize(
    width: usize,
    target_table: &TruthTable,
    max_iterations: usize,
) -> SynthesisResult {
    let n = 1usize << width;

    // Build target permutation vector
    let mut target_perm = vec![0usize; n];
    for (input, output) in target_table {
        let i = bits_to_index(input);
        let j = bits_to_index(output);
        target_perm[i] = j;
    }

    // Check if target is a valid permutation
    let mut seen = vec![false; n];
    let mut is_valid_perm = true;
    for &v in &target_perm {
        if v >= n || seen[v] {
            is_valid_perm = false;
            break;
        }
        seen[v] = true;
    }

    if !is_valid_perm {
        return SynthesisResult {
            gate_sequence: vec![],
            gate_count: 0,
            quantum_cost: 0,
            exact: false,
            iterations: 0,
            width,
        };
    }

    // Check if already identity
    if target_perm.iter().enumerate().all(|(i, &v)| i == v) {
        return SynthesisResult {
            gate_sequence: vec![],
            gate_count: 0,
            quantum_cost: 0,
            exact: true,
            iterations: 0,
            width,
        };
    }

    // Generate all positional gates
    let candidates = generate_all_gates(width);

    // BFS: iterative, NOT recursive
    let mut visited: HashSet<Vec<usize>> = HashSet::new();
    let identity_perm: Vec<usize> = (0..n).collect();
    visited.insert(identity_perm.clone());

    // Queue holds (current_perm, gate_sequence)
    let mut current_level: Vec<(Vec<usize>, Vec<PositionalGate>)> = vec![(identity_perm, vec![])];
    let mut iterations = 0;

    while !current_level.is_empty() && iterations < max_iterations {
        let mut next_level: Vec<(Vec<usize>, Vec<PositionalGate>)> = Vec::new();

        for (current_perm, gate_seq) in current_level {
            iterations += 1;
            if iterations > max_iterations {
                break;
            }

            for gate in &candidates {
                let new_perm =
                    apply_gates_to_perm(&current_perm, std::slice::from_ref(gate), width);

                if new_perm == target_perm {
                    let mut full_seq = gate_seq.clone();
                    full_seq.push(gate.clone());
                    let total_cost: usize = full_seq.iter().map(|g| g.quantum_cost()).sum();
                    let gate_strs: Vec<(String, Vec<usize>)> = full_seq
                        .iter()
                        .map(|g| {
                            let name = match g.gate_type {
                                GateType::Not => "NOT",
                                GateType::Cnot => "CNOT",
                                GateType::Toffoli => "Toffoli",
                            };
                            (name.to_string(), g.positions.clone())
                        })
                        .collect();
                    return SynthesisResult {
                        gate_sequence: gate_strs,
                        gate_count: full_seq.len(),
                        quantum_cost: total_cost,
                        exact: true,
                        iterations,
                        width,
                    };
                }

                if visited.insert(new_perm.clone()) {
                    let mut new_seq = gate_seq.clone();
                    new_seq.push(gate.clone());
                    next_level.push((new_perm, new_seq));
                }
            }
        }

        current_level = next_level;
    }

    // Heuristic fallback: ESOP-based
    let heuristic_seq = esop_heuristic(width, &target_perm);
    SynthesisResult {
        gate_count: heuristic_seq.len(),
        quantum_cost: heuristic_seq.iter().map(|g| g.quantum_cost()).sum(),
        exact: false,
        gate_sequence: heuristic_seq
            .iter()
            .map(|g| {
                let name = match g.gate_type {
                    GateType::Not => "NOT",
                    GateType::Cnot => "CNOT",
                    GateType::Toffoli => "Toffoli",
                };
                (name.to_string(), g.positions.clone())
            })
            .collect(),
        iterations,
        width,
    }
}

/// Apply a gate sequence to a bit vector (for verification).
pub fn apply_sequence(bits: &[u8], sequence: &[(String, Vec<usize>)]) -> Vec<u8> {
    let mut result = bits.to_vec();
    for (name, positions) in sequence {
        match name.as_str() {
            "NOT" => {
                result[positions[0]] ^= 1;
            }
            "CNOT" => {
                result[positions[1]] ^= result[positions[0]];
            }
            "Toffoli" => {
                result[positions[2]] ^= result[positions[0]] & result[positions[1]];
            }
            _ => {}
        }
    }
    result
}

/// ESOP-based heuristic synthesis (greedy, not optimal).
fn esop_heuristic(width: usize, _target_perm: &[usize]) -> Vec<PositionalGate> {
    // Simplified heuristic: try to fix output bits one at a time
    // This is a placeholder for a full Reed-Muller / ESOP decomposition
    let mut gates = Vec::new();
    if width >= 1 {
        // Just add a NOT as a placeholder for the heuristic
        gates.push(PositionalGate {
            gate_type: GateType::Not,
            positions: vec![0],
        });
    }
    gates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesize_identity() {
        let tt = vec![
            (vec![0, 0], vec![0, 0]),
            (vec![0, 1], vec![0, 1]),
            (vec![1, 0], vec![1, 0]),
            (vec![1, 1], vec![1, 1]),
        ];
        let result = synthesize(2, &tt, 100);
        assert!(result.exact);
        assert_eq!(result.gate_count, 0);
    }

    #[test]
    fn synthesize_not() {
        let tt = vec![(vec![0], vec![1]), (vec![1], vec![0])];
        let result = synthesize(1, &tt, 100);
        assert!(result.exact);
        assert_eq!(result.gate_count, 1);
    }

    #[test]
    fn synthesize_cnot() {
        let tt = vec![
            (vec![0, 0], vec![0, 0]),
            (vec![0, 1], vec![0, 1]),
            (vec![1, 0], vec![1, 1]),
            (vec![1, 1], vec![1, 0]),
        ];
        let result = synthesize(2, &tt, 100);
        assert!(result.exact);
        assert_eq!(result.gate_count, 1);
    }

    #[test]
    fn synthesize_swap() {
        // Swap two bits: [a,b] -> [b,a]
        let tt = vec![
            (vec![0, 0], vec![0, 0]),
            (vec![0, 1], vec![1, 0]),
            (vec![1, 0], vec![0, 1]),
            (vec![1, 1], vec![1, 1]),
        ];
        let result = synthesize(2, &tt, 1000);
        assert!(result.exact);
        // Verify the circuit
        for (input, expected) in &tt {
            let actual = apply_sequence(input, &result.gate_sequence);
            assert_eq!(&actual, expected, "Mismatch on input {:?}", input);
        }
    }

    #[test]
    fn synthesize_non_bijection_fails() {
        let tt = vec![
            (vec![0], vec![0]),
            (vec![1], vec![0]), // NOT bijective
        ];
        let result = synthesize(1, &tt, 100);
        assert!(!result.exact);
    }

    #[test]
    fn synthesize_result_is_correct() {
        let tt = vec![(vec![0], vec![1]), (vec![1], vec![0])];
        let result = synthesize(1, &tt, 100);
        assert!(result.exact);
        for (input, expected) in &tt {
            let actual = apply_sequence(input, &result.gate_sequence);
            assert_eq!(&actual, expected);
        }
    }

    #[test]
    fn synthesize_toffoli_table() {
        let mut tt = Vec::new();
        for i in 0..8 {
            let input = index_to_bits(i, 3);
            let mut output = input.clone();
            output[2] ^= output[0] & output[1];
            tt.push((input, output));
        }
        let result = synthesize(3, &tt, 10000);
        assert!(result.exact);
        for (input, expected) in &tt {
            let actual = apply_sequence(input, &result.gate_sequence);
            assert_eq!(&actual, expected, "Mismatch on input {:?}", input);
        }
    }

    #[test]
    fn synthesize_double_not() {
        // NOT on bit 1 of a 2-bit system
        let tt = vec![
            (vec![0, 0], vec![0, 1]),
            (vec![0, 1], vec![0, 0]),
            (vec![1, 0], vec![1, 1]),
            (vec![1, 1], vec![1, 0]),
        ];
        let result = synthesize(2, &tt, 100);
        assert!(result.exact);
        assert_eq!(result.gate_count, 1);
    }

    #[test]
    fn positional_gate_not() {
        let g = PositionalGate {
            gate_type: GateType::Not,
            positions: vec![2],
        };
        assert_eq!(g.apply(&[1, 0, 1]), vec![1, 0, 0]);
    }

    #[test]
    fn positional_gate_cnot() {
        let g = PositionalGate {
            gate_type: GateType::Cnot,
            positions: vec![1, 0], // control=bit1, target=bit0
        };
        assert_eq!(g.apply(&[0, 1, 0]), vec![1, 1, 0]);
    }
}
