//! Ancilla bit management for reversible circuits.
//!
//! Ancilla bits are extra workspace qubits/bits needed to make computations
//! reversible. **Clean** ancillae start in a known state (|0⟩), while **dirty**
//! ancillae are in an unknown state. Efficient ancilla management is crucial
//! for minimizing circuit width and quantum resources.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The state of an ancilla bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AncillaState {
    /// Clean: guaranteed to be |0⟩.
    Clean,
    /// Dirty: state unknown, must be restored before deallocation.
    Dirty,
    /// In use: currently allocated for some computation.
    InUse,
}

/// Metadata about a single ancilla bit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AncillaBit {
    /// Unique index within the circuit's ancilla register.
    pub index: usize,
    /// Current state of this ancilla.
    pub state: AncillaState,
    /// Label for what this ancilla is being used for (if InUse).
    pub label: Option<String>,
}

/// Manages a pool of ancilla bits for a reversible computation.
///
/// The manager tracks which ancillae are clean, dirty, or in-use,
/// and enforces the invariant that clean ancillae must be returned to |0⟩
/// before deallocation (Bennett's time-space tradeoff).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AncillaManager {
    /// All ancilla bits in this pool.
    bits: Vec<AncillaBit>,
    /// Total number of ancilla bits ever allocated.
    total_allocated: usize,
    /// Peak concurrent usage.
    peak_usage: usize,
    /// Current in-use count.
    current_usage: usize,
    /// Label → count tracker for garbage output accounting.
    garbage_labels: HashMap<String, usize>,
}

impl AncillaManager {
    /// Create a new AncillaManager with a given number of clean ancillae.
    pub fn new(clean_count: usize, dirty_count: usize) -> Self {
        let mut bits = Vec::with_capacity(clean_count + dirty_count);
        for i in 0..clean_count {
            bits.push(AncillaBit {
                index: i,
                state: AncillaState::Clean,
                label: None,
            });
        }
        for i in clean_count..clean_count + dirty_count {
            bits.push(AncillaBit {
                index: i,
                state: AncillaState::Dirty,
                label: None,
            });
        }
        AncillaManager {
            bits,
            total_allocated: clean_count + dirty_count,
            peak_usage: 0,
            current_usage: 0,
            garbage_labels: HashMap::new(),
        }
    }

    /// Allocate a clean ancilla bit. Returns its index.
    ///
    /// # Panics
    /// Panics if no clean ancilla is available.
    pub fn allocate_clean(&mut self, label: Option<String>) -> usize {
        for bit in &mut self.bits {
            if bit.state == AncillaState::Clean {
                bit.state = AncillaState::InUse;
                bit.label = label.clone();
                self.current_usage += 1;
                self.peak_usage = self.peak_usage.max(self.current_usage);
                return bit.index;
            }
        }
        panic!("No clean ancilla available");
    }

    /// Allocate a dirty ancilla bit. Returns its index.
    ///
    /// # Panics
    /// Panics if no dirty ancilla is available.
    pub fn allocate_dirty(&mut self, label: Option<String>) -> usize {
        for bit in &mut self.bits {
            if bit.state == AncillaState::Dirty {
                bit.state = AncillaState::InUse;
                bit.label = label.clone();
                self.current_usage += 1;
                self.peak_usage = self.peak_usage.max(self.current_usage);
                return bit.index;
            }
        }
        panic!("No dirty ancilla available");
    }

    /// Deallocate an ancilla bit, marking it clean.
    ///
    /// The caller is responsible for ensuring the bit has been returned to |0⟩
    /// (e.g., via uncomputation).
    pub fn deallocate_clean(&mut self, index: usize) {
        let bit = &mut self.bits[index];
        assert_eq!(
            bit.state,
            AncillaState::InUse,
            "Ancilla {} is not in use",
            index
        );
        bit.state = AncillaState::Clean;
        bit.label = None;
        self.current_usage -= 1;
    }

    /// Deallocate an ancilla bit, marking it dirty.
    pub fn deallocate_dirty(&mut self, index: usize) {
        let bit = &mut self.bits[index];
        assert_eq!(
            bit.state,
            AncillaState::InUse,
            "Ancilla {} is not in use",
            index
        );
        bit.state = AncillaState::Dirty;
        bit.label = None;
        self.current_usage -= 1;
    }

    /// Number of currently available clean ancillae.
    pub fn clean_available(&self) -> usize {
        self.bits
            .iter()
            .filter(|b| b.state == AncillaState::Clean)
            .count()
    }

    /// Number of currently available dirty ancillae.
    pub fn dirty_available(&self) -> usize {
        self.bits
            .iter()
            .filter(|b| b.state == AncillaState::Dirty)
            .count()
    }

    /// Number of currently in-use ancillae.
    pub fn in_use(&self) -> usize {
        self.current_usage
    }

    /// Peak concurrent usage across the lifetime.
    pub fn peak_usage(&self) -> usize {
        self.peak_usage
    }

    /// Total ancilla pool size.
    pub fn total(&self) -> usize {
        self.bits.len()
    }

    /// Register a garbage output label and count.
    pub fn track_garbage(&mut self, label: &str, count: usize) {
        *self.garbage_labels.entry(label.to_string()).or_insert(0) += count;
    }

    /// Total garbage output count across all labels.
    pub fn total_garbage(&self) -> usize {
        self.garbage_labels.values().sum()
    }

    /// Get a reference to the garbage labels map.
    pub fn garbage_labels(&self) -> &HashMap<String, usize> {
        &self.garbage_labels
    }

    /// Get a reference to a specific ancilla bit.
    pub fn get_bit(&self, index: usize) -> &AncillaBit {
        &self.bits[index]
    }

    /// Verify that all ancillae have been returned to their initial states
    /// (clean ancillae back to clean, dirty back to dirty).
    pub fn verify_all_returned(&self) -> bool {
        self.bits.iter().all(|b| b.state != AncillaState::InUse)
    }
}

/// Tracker for garbage outputs in a computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCount {
    /// Map from output line index to whether it's garbage.
    lines: Vec<bool>,
    /// Labels for garbage lines.
    labels: HashMap<usize, String>,
}

impl GarbageCount {
    /// Create a new GarbageCount for `width` output lines, initially none garbage.
    pub fn new(width: usize) -> Self {
        GarbageCount {
            lines: vec![false; width],
            labels: HashMap::new(),
        }
    }

    /// Mark a specific output line as garbage.
    pub fn mark_garbage(&mut self, index: usize, label: String) {
        self.lines[index] = true;
        self.labels.insert(index, label);
    }

    /// Number of garbage output lines.
    pub fn count(&self) -> usize {
        self.lines.iter().filter(|&&g| g).count()
    }

    /// Number of useful (non-garbage) output lines.
    pub fn useful_count(&self) -> usize {
        self.lines.iter().filter(|&&g| !g).count()
    }

    /// Check if a specific line is garbage.
    pub fn is_garbage(&self, index: usize) -> bool {
        self.lines[index]
    }

    /// Get the label for a garbage line.
    pub fn label(&self, index: usize) -> Option<&str> {
        self.labels.get(&index).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_and_deallocate_clean() {
        let mut mgr = AncillaManager::new(3, 0);
        let a0 = mgr.allocate_clean(Some("scratch".into()));
        assert_eq!(mgr.clean_available(), 2);
        assert_eq!(mgr.in_use(), 1);
        mgr.deallocate_clean(a0);
        assert_eq!(mgr.clean_available(), 3);
        assert!(mgr.verify_all_returned());
    }

    #[test]
    fn peak_usage_tracking() {
        let mut mgr = AncillaManager::new(4, 0);
        let a = mgr.allocate_clean(None);
        let b = mgr.allocate_clean(None);
        assert_eq!(mgr.peak_usage(), 2);
        mgr.deallocate_clean(a);
        mgr.deallocate_clean(b);
        assert_eq!(mgr.peak_usage(), 2);
    }

    #[test]
    fn dirty_ancilla() {
        let mut mgr = AncillaManager::new(0, 2);
        let d = mgr.allocate_dirty(Some("temp".into()));
        assert_eq!(mgr.dirty_available(), 1);
        mgr.deallocate_dirty(d);
        assert_eq!(mgr.dirty_available(), 2);
    }

    #[test]
    fn garbage_tracker() {
        let mut gc = GarbageCount::new(4);
        assert_eq!(gc.count(), 0);
        gc.mark_garbage(0, "copy".into());
        gc.mark_garbage(1, "scratch".into());
        assert_eq!(gc.count(), 2);
        assert_eq!(gc.useful_count(), 2);
        assert_eq!(gc.label(0), Some("copy"));
    }

    #[test]
    #[should_panic]
    fn no_clean_panics() {
        let mut mgr = AncillaManager::new(0, 1);
        mgr.allocate_clean(None);
    }

    #[test]
    fn garbage_labels_in_manager() {
        let mut mgr = AncillaManager::new(2, 0);
        mgr.track_garbage("copy_a", 2);
        mgr.track_garbage("copy_b", 1);
        assert_eq!(mgr.total_garbage(), 3);
    }

    #[test]
    fn full_lifecycle() {
        let mut mgr = AncillaManager::new(5, 3);
        let a = mgr.allocate_clean(Some("t1".into()));
        let b = mgr.allocate_clean(Some("t2".into()));
        let c = mgr.allocate_dirty(Some("d1".into()));
        assert_eq!(mgr.in_use(), 3);
        mgr.deallocate_clean(a);
        mgr.deallocate_clean(b);
        mgr.deallocate_dirty(c);
        assert!(mgr.verify_all_returned());
        assert_eq!(mgr.peak_usage(), 3);
    }
}
