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
