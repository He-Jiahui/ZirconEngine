use std::collections::HashMap;

use super::HighlightSet;

#[cfg(test)]
#[path = "viewport_highlight_store/tests/hash_index_tests.rs"]
mod hash_index_tests;

#[derive(Clone, Debug, PartialEq)]
pub struct ViewportHighlightSet {
    generation: u64,
    overlay_revision: u64,
    set: HighlightSet,
}

impl ViewportHighlightSet {
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn overlay_revision(&self) -> u64 {
        self.overlay_revision
    }

    pub fn set(&self) -> &HighlightSet {
        &self.set
    }
}

/// Per-runtime, per-viewport latest-value storage for editor overlay input.
///
/// The store replaces values in-place. It deliberately has no producer queue:
/// render extraction consumes the latest accepted value for its viewport.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewportHighlightStore {
    by_viewport: HashMap<u64, ViewportHighlightSet>,
}

impl ViewportHighlightStore {
    /// Returns false for stale generations or invalid render attributes.
    pub fn submit(&mut self, viewport: u64, generation: u64, set: HighlightSet) -> bool {
        if !set.attributes().is_valid() {
            return false;
        }
        if let Some(current) = self.by_viewport.get_mut(&viewport) {
            if generation < current.generation {
                return false;
            }
            if current.set != set {
                current.overlay_revision = current
                    .overlay_revision
                    .checked_add(1)
                    .expect("viewport highlight overlay revision overflow");
                current.set = set;
            }
            current.generation = generation;
            return true;
        }
        self.by_viewport.insert(
            viewport,
            ViewportHighlightSet {
                generation,
                overlay_revision: 1,
                set,
            },
        );
        true
    }

    pub fn get(&self, viewport: u64) -> Option<&ViewportHighlightSet> {
        self.by_viewport.get(&viewport)
    }
}

#[cfg(test)]
#[path = "tests/viewport_highlight_store.rs"]
mod tests;
