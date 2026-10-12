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

    /// 候选保留此快照作为基线；准备不会改写已发布代，authority 仅在当前 Arc 仍是该基线时发布候选。
    /// Creates an unpublished mutable candidate rooted at this exact generation.
    pub fn stage_update(self: &Arc<Self>) -> RuntimePluginCatalogCandidate {
        RuntimePluginCatalogCandidate::from_snapshot(Arc::clone(self))
    }
}

#[cfg(test)]
#[path = "tests/snapshot.rs"]
mod tests;
