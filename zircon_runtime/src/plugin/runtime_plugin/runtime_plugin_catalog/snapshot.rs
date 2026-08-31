use std::sync::Arc;

use super::{PluginCatalogGeneration, RuntimePluginCatalog, RuntimePluginCatalogCandidate};

/// Immutable read model for one published runtime plugin catalog generation.
#[derive(Debug)]
pub struct RuntimePluginCatalogSnapshot {
    catalog: RuntimePluginCatalog,
}

impl RuntimePluginCatalogSnapshot {
    /// Seals a completed mutable catalog so consumers cannot branch its revision authority.
    pub fn from_catalog(catalog: RuntimePluginCatalog) -> Self {
        Self { catalog }
    }

    pub fn generation(&self) -> PluginCatalogGeneration {
        self.catalog.generation()
    }

    pub fn catalog(&self) -> &RuntimePluginCatalog {
        &self.catalog
    }

    /// Creates an unpublished mutable candidate rooted at this exact generation.
    pub fn stage_update(self: &Arc<Self>) -> RuntimePluginCatalogCandidate {
        RuntimePluginCatalogCandidate::from_snapshot(Arc::clone(self))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{RuntimePluginCatalog, RuntimePluginCatalogSnapshot};

    #[test]
    fn snapshot_handle_shares_one_sealed_catalog_generation() {
        let catalog = RuntimePluginCatalog::from_descriptors([]);
        let generation = catalog.generation();
        let snapshot = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(catalog));
        let cloned = Arc::clone(&snapshot);

        assert_eq!(snapshot.generation(), generation);
        assert!(Arc::ptr_eq(&snapshot, &cloned));
        assert_eq!(Arc::strong_count(&snapshot), 2);
    }
}
