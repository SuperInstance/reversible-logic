//! # reversible-logic
//!
//! Reversible computing: logic gates and circuits that preserve information.
//!
//! Every operation in this library is a bijection — no information is lost,
//! no bits are erased. This is the foundation of lossless computation and
//! a prerequisite for quantum circuits.
//!
//! # Modules
//!
//! - [`gate`] — Reversible gates (NOT, CNOT, Toffoli, Fredkin, Peres)
//! - [`circuit`] — Compositions of gates into reversible circuits
//! - [`permutation`] — Permutation theory underlying reversible operations
//! - [`ancilla`] — Ancilla bit management for workspace allocation
//! - [`uncompute`] — Uncomputation and Bennett's time-space tradeoff
//! - [`synthesis`] — Gate synthesis from truth tables (ESOP-based)
//!
//! # Quick Start
//!
//! ```
//! use reversible_logic::gate::ReversibleGate;
//!
//! let toffoli = ReversibleGate::Toffoli;
//! let output = toffoli.apply(&[1, 1, 0]);
//! assert_eq!(output, vec![1, 1, 1]);
//! ```

pub mod ancilla;
pub mod circuit;
pub mod gate;
pub mod permutation;
pub mod synthesis;
pub mod uncompute;

pub use ancilla::{AncillaManager, AncillaState, GarbageCount};
pub use circuit::{CircuitBuilder, ReversibleCircuit};
pub use gate::ReversibleGate;
pub use permutation::{Parity, Permutation, PermutationGroup};
pub use synthesis::{
    GateType, PositionalGate, SynthesisResult, TruthTable, apply_sequence, synthesize,
};
pub use uncompute::{UncomputeResult, bennett_uncompute, invert_circuit, verify_inverse};
