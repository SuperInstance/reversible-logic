//! Permutation theory for reversible computing.
//!
//! A permutation is a bijective mapping σ: {0,..,n-1} → {0,..,n-1}.
//! Reversible gates and circuits are permutations on bit patterns.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A permutation of `n` elements, stored as an image vector.
///
/// `mapping[i] = j` means σ(i) = j.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permutation {
    /// Image vector: mapping[i] is where element i maps to.
    mapping: Vec<usize>,
}

impl Permutation {
    /// Create a new permutation from an image vector.
    ///
    /// # Panics
    /// Panics if the mapping is not a valid permutation (not bijective).
    pub fn new(mapping: Vec<usize>) -> Self {
        let n = mapping.len();
        let mut seen = vec![false; n];
        for &v in &mapping {
            assert!(
                v < n,
                "Value {} out of range for permutation of size {}",
                v,
                n
            );
            assert!(!seen[v], "Duplicate value {} — not a bijection", v);
            seen[v] = true;
        }
        Permutation { mapping }
    }

    /// Identity permutation of size `n`.
    pub fn identity(n: usize) -> Self {
        Permutation {
            mapping: (0..n).collect(),
        }
    }

    /// The number of elements in this permutation's domain.
    pub fn size(&self) -> usize {
        self.mapping.len()
    }

    /// Apply this permutation to element `i`.
    pub fn apply(&self, i: usize) -> usize {
        self.mapping[i]
    }

    /// Apply this permutation to an entire slice, producing a new vector.
    pub fn apply_vec(&self, v: &[usize]) -> Vec<usize> {
        v.iter().map(|&i| self.mapping[i]).collect()
    }

    /// Compute the inverse permutation σ⁻¹.
    ///
    /// σ⁻¹(j) = i iff σ(i) = j.
    pub fn inverse(&self) -> Permutation {
        let n = self.mapping.len();
        let mut inv = vec![0usize; n];
        for (i, &j) in self.mapping.iter().enumerate() {
            inv[j] = i;
        }
        Permutation { mapping: inv }
    }

    /// Compose two permutations: (σ ∘ τ)(i) = σ(τ(i)).
    ///
    /// `self` is applied second, `other` is applied first.
    pub fn compose(&self, other: &Permutation) -> Permutation {
        assert_eq!(self.size(), other.size(), "Permutation size mismatch");
        let mapping = other.mapping.iter().map(|&i| self.mapping[i]).collect();
        Permutation { mapping }
    }

    /// Compute the cycle decomposition of this permutation.
    ///
    /// Returns cycles as vectors of element indices. Each cycle (a₁ a₂ ... aₖ)
    /// means σ(a₁)=a₂, σ(a₂)=a₃, ..., σ(aₖ)=a₁.
    pub fn cycles(&self) -> Vec<Vec<usize>> {
        let n = self.mapping.len();
        let mut visited = vec![false; n];
        let mut cycles = Vec::new();
        for start in 0..n {
            if visited[start] {
                continue;
            }
            let mut cycle = Vec::new();
            let mut current = start;
            while !visited[current] {
                visited[current] = true;
                cycle.push(current);
                current = self.mapping[current];
            }
            if cycle.len() > 1 {
                cycles.push(cycle);
            }
        }
        cycles
    }

    /// Return the sign (parity) of this permutation.
    ///
    /// A permutation is **even** if it can be written as a product of an even
    /// number of transpositions, **odd** otherwise.
    pub fn parity(&self) -> Parity {
        // A cycle of length k decomposes into k-1 transpositions.
        // Total transpositions = sum of (cycle_length - 1) for all cycles.
        let transposition_count: usize = self.cycles().iter().map(|c| c.len() - 1).sum();
        if transposition_count.is_multiple_of(2) {
            Parity::Even
        } else {
            Parity::Odd
        }
    }

    /// Return true if this is the identity permutation.
    pub fn is_identity(&self) -> bool {
        self.mapping.iter().enumerate().all(|(i, &v)| i == v)
    }
}

/// Parity (sign) of a permutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parity {
    Even,
    Odd,
}

/// A group of permutations acting on the same set size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermutationGroup {
    /// All permutations in the group.
    permutations: Vec<Permutation>,
    /// Size of the domain (all permutations must have this size).
    size: usize,
}

impl PermutationGroup {
    /// Create a new permutation group, validating that all permutations
    /// have the same size.
    pub fn new(permutations: Vec<Permutation>) -> Self {
        let size = permutations.first().map(|p| p.size()).unwrap_or(0);
        for p in &permutations {
            assert_eq!(p.size(), size, "All permutations must have same size");
        }
        PermutationGroup { permutations, size }
    }

    /// The trivial group {id} of size `n`.
    pub fn trivial(n: usize) -> Self {
        PermutationGroup {
            permutations: vec![Permutation::identity(n)],
            size: n,
        }
    }

    /// Number of permutations in this group.
    pub fn order(&self) -> usize {
        self.permutations.len()
    }

    /// Generate the closure of this set of permutations under composition
    /// (i.e., compute the full subgroup they generate). Uses iterative
    /// BFS with a HashSet to track visited states.
    pub fn closure(&self) -> PermutationGroup {
        let mut elements: HashSet<Vec<usize>> = HashSet::new();
        let mut _queue: Vec<Permutation> = self.permutations.clone();
        let mut result: Vec<Permutation> = self.permutations.clone();

        for p in &result {
            elements.insert(p.mapping.clone());
        }
        // Always include identity
        let id = Permutation::identity(self.size);
        if elements.insert(id.mapping.clone()) {
            result.push(id);
        }

        let mut idx = 0;
        while idx < result.len() {
            let a = result[idx].clone();
            for b in &self.permutations {
                for prod in [a.compose(b), b.compose(&a)] {
                    if elements.insert(prod.mapping.clone()) {
                        result.push(prod);
                    }
                }
            }
            idx += 1;
        }

        PermutationGroup {
            permutations: result,
            size: self.size,
        }
    }

    /// Check if a given permutation is in this group.
    pub fn contains(&self, p: &Permutation) -> bool {
        assert_eq!(p.size(), self.size);
        self.permutations.iter().any(|g| g.mapping == p.mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_permutation() {
        let p = Permutation::identity(4);
        assert_eq!(p.apply(0), 0);
        assert_eq!(p.apply(3), 3);
        assert!(p.is_identity());
    }

    #[test]
    fn swap_is_not_identity() {
        let p = Permutation::new(vec![1, 0, 2]);
        assert!(!p.is_identity());
    }

    #[test]
    fn inverse_roundtrip() {
        let p = Permutation::new(vec![2, 0, 1]);
        let inv = p.inverse();
        let composed = p.compose(&inv);
        assert!(composed.is_identity());
    }

    #[test]
    fn compose_associative() {
        let a = Permutation::new(vec![1, 2, 0]);
        let b = Permutation::new(vec![0, 2, 1]);
        let c = Permutation::new(vec![2, 1, 0]);
        let ab_c = a.compose(&b).compose(&c);
        let a_bc = a.compose(&b.compose(&c));
        assert_eq!(ab_c, a_bc);
    }

    #[test]
    fn cycle_decomposition() {
        // (0 1 2)(3 4)
        let p = Permutation::new(vec![1, 2, 0, 4, 3]);
        let cycles = p.cycles();
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn parity_of_transposition_is_odd() {
        let p = Permutation::new(vec![1, 0]);
        assert_eq!(p.parity(), Parity::Odd);
    }

    #[test]
    fn parity_of_identity_is_even() {
        let p = Permutation::identity(3);
        assert_eq!(p.parity(), Parity::Even);
    }

    #[test]
    fn group_closure_s3() {
        // S₃ generated by (0 1) and (0 1 2)
        let generators = PermutationGroup::new(vec![
            Permutation::new(vec![1, 0, 2]), // (0 1)
            Permutation::new(vec![1, 2, 0]), // (0 1 2)
        ]);
        let closure = generators.closure();
        assert_eq!(closure.order(), 6); // |S₃| = 6
    }

    #[test]
    #[should_panic]
    fn invalid_permutation_panics() {
        Permutation::new(vec![0, 0]); // not bijective
    }

    #[test]
    fn apply_vec() {
        let p = Permutation::new(vec![2, 0, 1]);
        assert_eq!(p.apply_vec(&[0, 1, 2]), vec![2, 0, 1]);
    }
}
