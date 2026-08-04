use std::collections::BTreeMap;

use super::HighlightSet;

#[derive(Clone, Debug, PartialEq)]
pub struct ViewportHighlightSet {
    generation: u64,
    set: HighlightSet,
}

impl ViewportHighlightSet {
    pub const fn generation(&self) -> u64 {
        self.generation
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
    by_viewport: BTreeMap<u64, ViewportHighlightSet>,
}

impl ViewportHighlightStore {
    /// Returns false when a newer value for this viewport is already retained.
    pub fn submit(&mut self, viewport: u64, generation: u64, set: HighlightSet) -> bool {
        if self
            .by_viewport
            .get(&viewport)
            .is_some_and(|current| generation < current.generation)
        {
            return false;
        }

        self.by_viewport
            .insert(viewport, ViewportHighlightSet { generation, set });
        true
    }

    pub fn get(&self, viewport: u64) -> Option<&ViewportHighlightSet> {
        self.by_viewport.get(&viewport)
    }
}

#[cfg(test)]
mod tests {
    use super::ViewportHighlightStore;
    use crate::core::framework::render::{HighlightRenderAttributes, HighlightSet};

    fn set(entities: impl IntoIterator<Item = u64>) -> HighlightSet {
        HighlightSet::new(
            entities,
            HighlightRenderAttributes::outlined([0.1, 0.2, 0.3, 1.0]),
        )
    }

    #[test]
    fn rejects_stale_generation_without_cross_viewport_leakage() {
        let mut store = ViewportHighlightStore::default();
        assert!(store.submit(3, 7, set([8, 2])));
        assert!(store.submit(4, 1, set([11])));
        assert!(!store.submit(3, 6, set([99])));

        assert_eq!(store.get(3).unwrap().generation(), 7);
        assert_eq!(store.get(3).unwrap().set().entities(), &[2, 8]);
        assert_eq!(store.get(4).unwrap().set().entities(), &[11]);
    }
}
