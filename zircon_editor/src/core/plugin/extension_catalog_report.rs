//! Materialized extension catalog for one immutable plugin generation.

use crate::core::asset::AssetTypeRegistry;
use crate::core::editor_extension::EditorExtensionRegistry;

#[derive(Clone, Debug)]
/// 一次目录或管理器代次的扩展物化结果；active_manager_generation 存在时只含当前 Active 插件，消费者需整次读取同一快照。
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
