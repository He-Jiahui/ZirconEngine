//! Persistent, folder-backed project asset index and dependency graph.
//! 注册表由扫描/导入写入 generation，查询和依赖关系从同一快照读取。

#[path = "asset_registry_diagnostic.rs"]
mod asset_registry_diagnostic;
#[path = "asset_registry_entry.rs"]
mod asset_registry_entry;
#[path = "asset_registry_error.rs"]
mod asset_registry_error;
#[path = "asset_registry_filter.rs"]
mod asset_registry_filter;
#[path = "asset_registry_index.rs"]
mod asset_registry_index;
#[path = "deletion.rs"]
mod deletion;
#[path = "dependency_extractors/mod.rs"]
pub(crate) mod dependency_extractors;
#[path = "incremental.rs"]
mod incremental;
#[path = "inspection.rs"]
mod inspection;
#[path = "persistence.rs"]
pub(crate) mod persistence;
#[path = "query.rs"]
mod query;
#[path = "rebuild.rs"]
mod rebuild;
#[path = "relocation.rs"]
mod relocation;
#[path = "targeted.rs"]
mod targeted;

pub use asset_registry_diagnostic::AssetRegistryDiagnostic;
pub use asset_registry_entry::AssetRegistryEntry;
pub use asset_registry_error::AssetRegistryError;
pub use asset_registry_filter::AssetRegistryFilter;
pub use asset_registry_index::AssetRegistryIndex;
