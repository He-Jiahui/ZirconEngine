mod error;
mod migrate;
mod retired_asset_reference;

pub use error::RetiredAssetRefMigrationError;
pub use migrate::{
    migrate_retired_asset_references, migrate_retired_asset_references_with,
    migrate_retired_persisted_asset_reference_with,
    migrate_retired_persisted_asset_references_with,
};
pub use retired_asset_reference::RetiredAssetReference;
