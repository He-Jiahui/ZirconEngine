//! Materialized extension catalog for one immutable plugin generation.

use crate::core::asset::AssetTypeRegistry;
use crate::core::editor_extension::EditorExtensionRegistry;

#[derive(Clone, Debug)]
pub struct EditorExtensionCatalogReport {
    pub catalog_generation: u64,
    /// Manager generation for an active phase view; None for a full catalog candidate.
    pub active_manager_generation: Option<u64>,
    pub registry: EditorExtensionRegistry,
    pub asset_types: AssetTypeRegistry,
    pub diagnostics: Vec<String>,
}

impl EditorExtensionCatalogReport {
    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
