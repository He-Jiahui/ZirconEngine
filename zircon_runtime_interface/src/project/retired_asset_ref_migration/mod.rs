//! 旧项目资产引用的兼容迁移入口。
//!
//! 批量入口只识别键恰为 uuid 与 url 的旧对象；带预算的批量入口先检查整份 JSON 的资源上限，
//! 通过后才开始重写和调用解析器。单对象入口只接收根对象，不遍历其容器。
//! 本模块不读取文件，也不确认 GUID 与资源注册表的对应关系；Runtime 调用方在
//! zircon_runtime/src/asset/migration/resolver.rs 的 MigrationResolver::resolve 中完成该解析。

#[path = "budget.rs"]
mod budget;
#[path = "error.rs"]
mod error;
#[path = "migrate.rs"]
mod migrate;
#[path = "../retired_asset_reference.rs"]
mod retired_asset_reference;

pub use budget::{
    RetiredAssetRefMigrationBudget, MAX_RETIRED_ASSET_REF_MIGRATION_DEPTH,
    MAX_RETIRED_ASSET_REF_MIGRATION_NODES, MAX_RETIRED_ASSET_REF_MIGRATION_REFERENCES,
};
pub use error::RetiredAssetRefMigrationError;
pub use migrate::{
    migrate_retired_asset_references, migrate_retired_asset_references_with,
    migrate_retired_asset_references_with_budget, migrate_retired_persisted_asset_reference_with,
    migrate_retired_persisted_asset_references_with,
    migrate_retired_persisted_asset_references_with_budget,
};
pub use retired_asset_reference::RetiredAssetReference;
