# reversible-logic

**Reversible computing: logic gates and circuits that preserve information.**

A Rust library for constructing, analyzing, and synthesizing reversible logic circuits — computations where no information is lost. Every operation is a bijection: the input can always be recovered from the output.

This is the foundation of lossless computation, quantum circuit design, and energy-efficient computing as described by Landauer's principle (1961): erasing a bit of information necessarily dissipates *kT* ln 2 of energy.

---

## Table of Contents

1. [Theory](#theory)
2. [Module Overview](#module-overview)
3. [Design Decisions](#design-decisions)
4. [Installation](#installation)
5. [Examples](#examples)
   - [Example 1: Gate Operations](#example-1-gate-operations)
   - [Example 2: Circuit Construction](#example-2-circuit-construction)
   - [Example 3: Synthesis from Truth Tables](#example-3-synthesis-from-truth-tables)
6. [ASCII Circuit Diagrams](#ascii-circuit-diagrams)
7. [API Reference](#api-reference)
8. [References](#references)

---

## Theory

### Reversible Gates

A logic gate is **reversible** if its mapping from input to output is a bijection — every input pattern maps to a unique output pattern, and vice versa. Formally, a gate operating on *n* bits implements a permutation σ ∈ S₂ⁿ on the set {0, 1, ..., 2ⁿ-1}.

**Landauer's Principle** (Landauer, 1961): Any logically irreversible operation must dissipate at least *kT* ln 2 joules of energy per bit erased. Reversible gates avoid this cost by construction.

**Formal Notation:**

- A reversible gate *G* with *n* inputs/outputs is a bijection:
  ```
  G: {0,1}ⁿ → {0,1}ⁿ
  ```
- Its **permutation matrix** *P(G)* is a 2ⁿ × 2ⁿ binary matrix where:
  ```
  P(G)[i][j] = 1  iff  G(binary(i)) = binary(j)
  ```
- *P(G)* has exactly one 1 in each row and each column.

### The Six Gates

| Gate | Arity | Function | Self-Inverse | Quantum Cost |
|------|-------|----------|:------------:|:------------:|
| Identity | *n* | (a₁,...,aₙ) → (a₁,...,aₙ) | ✓ | 0 |
| NOT | 1 | a → ¬a | ✓ | 1 |
| CNOT (Feynman) | 2 | (c, t) → (c, t ⊕ c) | ✓ | 1 |
| Toffoli (CCNOT) | 3 | (c₁, c₂, t) → (c₁, c₂, t ⊕ c₁c₂) | ✓ | 5 |
| Fredkin (CSWAP) | 3 | (c, x, y) → (c, x', y') where swap on c=1 | ✓ | 5 |
| Peres | 3 | (c₁, c₂, t) → (c₁, c₂⊕c₁, t ⊕ c₁c₂) | ✗ | 4 |

### Permutation Groups

The set of all reversible circuits on *n* bits forms the **symmetric group** S₂ⁿ under composition. Each circuit is a permutation, and circuit composition corresponds to permutation composition:

```
(C₁ ∘ C₂)(x) = C₁(C₂(x))
```

The **cycle decomposition** of a permutation σ is:

```
σ = (a₁ a₂ ... aₖ)(b₁ b₂ ... bₘ)...
```

where each cycle (x₁ x₂ ... xᵣ) means σ(x₁)=x₂, σ(x₂)=x₃, ..., σ(xᵣ)=x₁.

The **sign** (parity) of σ is:

```
sgn(σ) = (-1)^(number of transpositions)
```

A permutation is *even* if it can be written as a product of an even number of transpositions, *odd* otherwise.

### Bennett's Time-Space Tradeoff

Bennett (1973) showed that any Turing machine computation can be made reversible with at most a polynomial overhead in time and space. The key technique is **uncomputation**:

```
1. Compute:    |x, 0, 0⟩ → |x, f(x), g(x)⟩
2. Copy:       |x, f(x), g(x)⟩ → |x, f(x), g(x), f(x)⟩
3. Uncompute:  |x, f(x), g(x), f(x)⟩ → |x, 0, 0, f(x)⟩
```

Step 3 runs the inverse of the computation from step 1, restoring all ancilla bits to their initial states. The garbage outputs *g(x)* are cleaned, leaving only the answer *f(x)*.

**Bennett's theorem** (Bennett, 1973): For any function *f* computable in time *T(n)* and space *S(n)*, there exists a reversible computation using *O(S(n))* space and *O(T(n)^(1+ε))* time for any ε > 0.

**Lecerf's theorem** (Lecerf, 1963): Independently established that reversible computation is possible with overhead, predating Bennett's work by a decade.

### ESOP-Based Synthesis

**Exclusive-OR Sum of Products (ESOP)** is a method for synthesizing reversible circuits from truth tables. Given a target truth table:

1. Express each output bit as an XOR of product terms (Reed-Muller form).
2. Each product term maps to a Toffoli gate.
3. XOR operations map to CNOT gates.
4. The resulting circuit is a cascade of Toffoli and CNOT gates.

This library implements synthesis using **iterative BFS** (not recursive) with a `HashSet` to track visited permutation states, searching through the space of NOT, CNOT, and Toffoli gates.

---

## Module Overview

| Module | Description | Key Types |
|--------|-------------|-----------|
| `gate` | Reversible gate definitions | `ReversibleGate` |
| `circuit` | Ordered sequences of gates | `ReversibleCircuit`, `CircuitBuilder` |
| `permutation` | Permutation theory | `Permutation`, `PermutationGroup` |
| `ancilla` | Workspace bit management | `AncillaManager`, `GarbageCount` |
| `uncompute` | Inverse circuits & Bennett's trick | `invert_circuit`, `bennett_uncompute` |
| `synthesis` | Truth table → circuit search | `synthesize`, `PositionalGate` |

---

## Design Decisions

### Why `Vec<u8>` for bit vectors?

Simplicity and concreteness. This library uses `u8` values (0 or 1) for bits and `Vec<u8>` for bit vectors. No generic traits, no type-level programming. The tradeoff is memory efficiency — a `Vec<u8>` uses 8× more memory than a packed bit vector — but the code is far easier to understand, debug, and extend.

### Why concrete types over traits?

This library prioritizes **readability and correctness** over abstraction. Every public type is concrete: `ReversibleGate` is an enum, `Permutation` wraps a `Vec<usize>`, `ReversibleCircuit` holds a `Vec<ReversibleGate>`. This makes the code approachable for students, researchers, and anyone learning about reversible computing.

### Why no external dependencies (except `serde`)?

Reversible logic is fundamentally about bit manipulation and linear algebra over GF(2). No numerics, no parsing, no I/O. `serde` is included because serialization is essential for saving/loading circuits, but everything else is implemented from scratch.

### Why iterative synthesis instead of recursive?

Recursive search is elegant but can blow the stack for large search spaces. Our BFS approach uses an explicit queue and a `HashSet` for visited states, making memory usage explicit and controllable. The tradeoff: BFS guarantees minimal gate count (shortest path) but uses more memory than DFS.

### Why positional gates in synthesis?

A `ReversibleGate::Cnot` always acts on bits [0, 1]. For synthesis, we need to apply CNOT with arbitrary (control, target) positions. `PositionalGate` decouples gate type from bit positions, enabling the synthesizer to explore all possible gate placements.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
reversible-logic = "0.1"
```

Or use `cargo add`:

```bash
cargo add reversible-logic
```

---

## Examples

### Example 1: Gate Operations

Build and query individual reversible gates. Each gate has a truth table, permutation matrix, and quantum cost.

```rust
use reversible_logic::gate::ReversibleGate;

// Create a Toffoli gate (CCNOT)
let toffoli = ReversibleGate::Toffoli;

// Apply it to various inputs
assert_eq!(toffoli.apply(&[0, 0, 0]), vec![0, 0, 0]); // No flip
assert_eq!(toffoli.apply(&[0, 1, 0]), vec![0, 1, 0]); // No flip
assert_eq!(toffoli.apply(&[1, 0, 0]), vec![1, 0, 0]); // No flip
assert_eq!(toffoli.apply(&[1, 1, 0]), vec![1, 1, 1]); // Flip! Both controls = 1
assert_eq!(toffoli.apply(&[1, 1, 1]), vec![1, 1, 0]); // Flip back

// Generate the full truth table (8 rows for a 3-bit gate)
let table = toffoli.truth_table();
assert_eq!(table.len(), 8);

// Get the 8x8 permutation matrix
let matrix = toffoli.permutation_matrix();
assert_eq!(matrix.len(), 8); // 8 rows
assert_eq!(matrix[0].len(), 8); // 8 columns

// Query quantum cost
assert_eq!(toffoli.quantum_cost(), 5);

// Gate name
assert_eq!(toffoli.name(), "Toffoli");

// Width (number of bits)
assert_eq!(toffoli.width(), 3);
```

### Example 2: Circuit Construction with Builder

Use the fluent `CircuitBuilder` to compose gates into circuits, track garbage outputs, and compute costs.

```rust
use reversible_logic::circuit::CircuitBuilder;
use reversible_logic::gate::ReversibleGate;

// Build a circuit that adds two bits using a Toffoli gate
// Bits: [a, b, carry_out]
// a and b are preserved (garbage outputs), carry_out is the answer
let mut circuit = CircuitBuilder::new(3)
    .toffoli()
    .answer_lines(vec![2]) // Only bit 2 is the answer
    .build();

// Apply the circuit
let result = circuit.apply(&[1, 1, 0]);
assert_eq!(result, vec![1, 1, 1]); // carry_out = 1

// Track garbage
assert_eq!(circuit.garbage_count(), 2); // bits 0 and 1 are garbage
assert_eq!(circuit.extract_answer(&result), vec![1]); // Just the carry

// Build a more complex circuit: XOR using two CNOTs
let xor_circuit = CircuitBuilder::new(3)
    .cnot()
    .build();

assert_eq!(xor_circuit.gate_count(), 1);
assert_eq!(xor_circuit.quantum_cost(), 1);
assert_eq!(xor_circuit.width(), 3);

// Generate the full circuit truth table
let truth_table = xor_circuit.truth_table();
assert_eq!(truth_table.len(), 8); // 2^3 rows

// The circuit is reversible: double application = identity
let double_circuit = CircuitBuilder::new(3)
    .toffoli()
    .toffoli()
    .build();
let input = vec![1, 1, 0];
assert_eq!(double_circuit.apply(&input), input); // Cancelled out
```

### Example 3: Synthesis and Uncomputation

Synthesize a circuit from a truth table, then verify it and generate its inverse for Bennett's uncomputation pattern.

```rust
use reversible_logic::synthesis::{synthesize, apply_sequence};
use reversible_logic::permutation::Permutation;
use reversible_logic::uncompute::{invert_circuit, verify_inverse, bennett_uncompute};
use reversible_logic::circuit::CircuitBuilder;

// Define a truth table: swap two bits [a,b] -> [b,a]
let swap_table = vec![
    (vec![0, 0], vec![0, 0]),
    (vec![0, 1], vec![1, 0]),
    (vec![1, 0], vec![0, 1]),
    (vec![1, 1], vec![1, 1]),
];

// Synthesize a circuit
let result = synthesize(2, &swap_table, 1000);
assert!(result.exact);
println!("Found circuit with {} gates:", result.gate_count);
for (name, positions) in &result.gate_sequence {
    println!("  {} at {:?}", name, positions);
}

// Verify the synthesized circuit matches all truth table entries
for (input, expected) in &swap_table {
    let actual = apply_sequence(input, &result.gate_sequence);
    assert_eq!(&actual, expected, "Mismatch on {:?}", input);
}

// Create a permutation from the swap
let swap_perm = Permutation::new(vec![0, 2, 1, 3]);
assert!(!swap_perm.is_identity());
let inv = swap_perm.inverse();
assert!(swap_perm.compose(&inv).is_identity());

// Verify uncomputation works for a Toffoli circuit
let circuit = CircuitBuilder::new(3)
    .toffoli()
    .cnot()
    .build();
assert!(verify_inverse(&circuit));

// Bennett's pattern: compute, copy, uncompute
let bennett = bennett_uncompute(&circuit, 2, 1);
println!("Bennett circuit has {} gates", bennett.gate_count());
```

---

## ASCII Circuit Diagrams

### CNOT (Feynman Gate)

```
Control ───●───  =  Control ───●───
           │                    │
Target  ───⊕───      Target  ──⊕⊕──
```

Input → Output:
```
[0,0] → [0,0]    [0,1] → [0,1]    [1,0] → [1,1]    [1,1] → [1,0]
```

### Toffoli (CCNOT)

```
c₁ ───●───
       │
c₂ ───●───
       │
t  ───⊕───
```

Only flips target when **both** controls are 1:
```
[1,1,0] → [1,1,1]    [1,1,1] → [1,1,0]
```

### Fredkin (CSWAP)

```
c ───●────●───
     │    │
x ───⊕──⊕⊕───  (swap x↔y when c=1)
     │    │
y ───⊕──⊕⊕───
```

### Peres Gate

```
c₁ ───●────────
       │
c₂ ───⊕──●─────    c₂ ⊕= c₁,  t ⊕= c₁·c₂
          │
t  ──────⊕─────
```

### Bennett's Uncomputation Pattern

```
   ┌─────────┐         ┌─────────┐
a ─┤         ├──●──────┤         ├──
   │         │  │      │  INVERSE │
x ─┤ FORWARD │──⊕──●───┤         ├──
   │         │     │   │ (restores │
g ─┤         ├──●──⊕───┤ garbage)  ├──
   └─────────┘  copy   └─────────┘
                         (answer preserved)
```

Step by step:
1. **Forward**: Compute `|x, 0, 0⟩ → |x, f(x), g(x)⟩`
2. **Copy**: CNOT copies answer to ancilla: `|x, f(x), g(x), 0⟩ → |x, f(x), g(x), f(x)⟩`
3. **Uncompute**: Inverse of forward: `|x, f(x), g(x), f(x)⟩ → |x, 0, 0, f(x)⟩`

Result: ancillae restored to |0⟩, answer preserved.

### Swap Circuit (3 CNOTs)

```
a ───●───────⊕───●───
     │       │   │
b ───⊕───●───●───⊕───
         │
    (3 CNOT gates: a⊕b, then b⊕(a⊕b)=a, then a⊕(a⊕b)=b)
```

This is what `synthesize` discovers for the swap truth table.

### Half Adder (Reversible)

```
a ───●───●───  (Toffoli for carry, CNOT for sum)
     │   │
b ───●───⊕───
     │
c ───⊕───────  (carry = a·b, sum = a⊕b)
```

---

## API Reference

### `gate` Module

```rust
pub enum ReversibleGate {
    Identity { width: usize },
    Not,
    Cnot,
    Toffoli,
    Fredkin,
    Peres,
}
```

**Methods:**
- `width() -> usize` — Number of bits this gate operates on
- `apply(input: &[u8]) -> Vec<u8>` — Apply gate to input bits
- `truth_table() -> Vec<(Vec<u8>, Vec<u8>)>` — All input→output pairs
- `permutation_matrix() -> Vec<Vec<u8>>` — Binary permutation matrix
- `quantum_cost() -> usize` — Decomposition cost in elementary gates
- `name() -> &'static str` — Human-readable gate name

### `circuit` Module

```rust
pub struct ReversibleCircuit { ... }
pub struct CircuitBuilder { ... }
```

**ReversibleCircuit methods:**
- `new(width, gates) -> Self` — Create from gate sequence
- `apply(input: &[u8]) -> Vec<u8>` — Run circuit on input
- `gate_count() -> usize` — Number of gates
- `quantum_cost() -> usize` — Total quantum cost
- `garbage_count() -> usize` — Non-answer output lines
- `truth_table() -> Vec<(Vec<u8>, Vec<u8>)>` — Full truth table

**CircuitBuilder** (fluent API):
- `CircuitBuilder::new(width) -> Self`
- `.gate(gate) -> Self` — Add any gate
- `.cnot() / .toffoli() / .fredkin() / .peres() / .add_not() -> Self`
- `.answer_lines(indices) -> Self` — Mark answer outputs
- `.build() -> ReversibleCircuit`

### `permutation` Module

```rust
pub struct Permutation { ... }
pub enum Parity { Even, Odd }
pub struct PermutationGroup { ... }
```

**Permutation methods:**
- `new(mapping: Vec<usize>) -> Self` — Create from image vector
- `identity(n) -> Self` — Identity permutation
- `apply(i: usize) -> usize` — Map element i
- `inverse() -> Permutation` — Compute σ⁻¹
- `compose(&other) -> Permutation` — Compute σ ∘ τ
- `cycles() -> Vec<Vec<usize>>` — Cycle decomposition
- `parity() -> Parity` — Even or odd
- `is_identity() -> bool` — Check if identity

**PermutationGroup methods:**
- `new(permutations) -> Self` — Create group from generators
- `closure() -> PermutationGroup` — Compute generated subgroup (iterative BFS)
- `contains(&Permutation) -> bool` — Membership test
- `order() -> usize` — Number of elements

### `ancilla` Module

```rust
pub struct AncillaManager { ... }
pub enum AncillaState { Clean, Dirty, InUse }
pub struct GarbageCount { ... }
```

**AncillaManager methods:**
- `new(clean_count, dirty_count) -> Self` — Create pool
- `allocate_clean(label) -> usize` — Get a |0⟩ ancilla
- `allocate_dirty(label) -> usize` — Get a dirty ancilla
- `deallocate_clean(index)` — Return ancilla to clean pool
- `deallocate_dirty(index)` — Return ancilla to dirty pool
- `clean_available() / dirty_available() / in_use() -> usize`
- `peak_usage() -> usize` — Max concurrent usage
- `verify_all_returned() -> bool` — Check all ancillae freed
- `track_garbage(label, count)` — Register garbage outputs
- `total_garbage() -> usize` — Sum of all garbage

### `uncompute` Module

```rust
pub struct UncomputeResult { ... }
```

**Functions:**
- `invert_circuit(&ReversibleCircuit) -> UncomputeResult` — Generate inverse
- `bennett_uncompute(circuit, answer_source, copy_target) -> ReversibleCircuit` — Bennett pattern
- `verify_inverse(&ReversibleCircuit) -> bool` — Verify C ∘ C⁻¹ = id
- `verify_ancilla_cleanup(circuit, ancilla_indices) -> bool` — Verify ancillae restored

### `synthesis` Module

```rust
pub struct SynthesisResult { ... }
pub struct PositionalGate { ... }
pub enum GateType { Not, Cnot, Toffoli }
```

**Functions:**
- `synthesize(width, truth_table, max_iterations) -> SynthesisResult` — BFS synthesis
- `apply_sequence(bits, sequence) -> Vec<u8>` — Apply synthesized gate sequence

**SynthesisResult fields:**
- `gate_sequence: Vec<(String, Vec<usize>)>` — Gate name + positions
- `gate_count: usize` — Number of gates
- `quantum_cost: usize` — Total cost
- `exact: bool` — Whether exact match was found
- `iterations: usize` — BFS iterations used
- `width: usize` — Bit width

---

## References

1. **Bennett, C.H.** (1973). "Logical Reversibility of Computation." *IBM Journal of Research and Development*, 17(6), 525–532. — The foundational paper showing that any computation can be made reversible with polynomial overhead. Introduced the compute-copy-uncompute pattern.

2. **Fredkin, E. & Toffoli, T.** (1982). "Conservative Logic." *International Journal of Theoretical Physics*, 21(3/4), 219–253. — Introduced the Fredkin gate (CSWAP) and established that reversible logic is universal. Showed that billiard-ball models of computation are feasible.

3. **Toffoli, T.** (1980). "Reversible Computing." In *Automata, Languages and Programming* (ICALP 1980), Springer LNCS 85, 632–644. — Introduced the Toffoli gate (CCNOT) and proved that reversible cellular automata can simulate irreversible ones.

4. **Lecerf, Y.** (1963). "Machines de Turing réversibles. Récursive insolubilité en n ∈ N de l'équation u = θⁿ, où θ est un isomorphisme de codes." *Comptes Rendus Hebdomadaires des Séances de l'Académie des Sciences*, 257, 2597–2600. — Predates Bennett by a decade; independently established reversible Turing machines.

5. **Nielsen, M.A. & Chuang, I.L.** (2010). *Quantum Computation and Quantum Information* (10th Anniversary Edition). Cambridge University Press. — The standard textbook for quantum computing; chapters 4–5 cover quantum circuits and the relationship between reversible and quantum gates.

6. **Landauer, R.** (1961). "Irreversibility and Heat Generation in the Computing Process." *IBM Journal of Research and Development*, 5(3), 183–191. — Established that erasing information necessarily dissipates energy; the physical motivation for reversible computing.

7. **Peres, A.** (1985). "Reversible Logic and Quantum Computers." *Physical Review A*, 32(6), 3266–3276. — Introduced the Peres gate and studied its quantum cost advantages over Toffoli decomposition.

8. **Maslov, D., Dueck, G.W. & Miller, D.M.** (2007). "Techniques for the Synthesis of Reversible Toffoli Networks." *ACM Journal on Emerging Technologies in Computing Systems*, 3(4), 1–25. — ESOP-based synthesis methods for reversible circuits using Toffoli, CNOT, and NOT gates.

---

## License

MIT

---

## Contributing

Contributions welcome! This library aims to be an educational resource as much as a practical tool. When adding features:

1. Add comprehensive tests (aim for >1 test per new function)
2. Document the theory behind your addition
3. Use concrete types — no generic trait hierarchies
4. Keep external dependencies to `serde` only
5. Run `cargo fmt`, `cargo clippy`, and `cargo test` before submitting

**Educate, don't sell.**
