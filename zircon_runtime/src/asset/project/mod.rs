mod manager;
mod manifest;
mod meta;
pub(crate) mod meta_io;
mod package_asset_registry;
mod paths;
mod script_manifest;
mod shader_resource_records;

pub(crate) use manager::mint_meta_for_migration;
pub use manager::ProjectManager;
pub use manifest::{ProjectManifest, ProjectManifestError};
pub use meta::{
    AssetMetaDocument, AssetMetaEntry, AssetMetaError, AssetMetaResult, AssetSourceUnit,
    PreviewState,
};
pub use package_asset_registry::PackageAssetRegistry;
pub use paths::ProjectPaths;
pub use script_manifest::ProjectScriptManifest;
pub use shader_resource_records::{
    shader_resource_records_from_asset_root, shader_resource_records_from_asset_roots,
    ShaderResourceRecordExportError, ShaderResourceRecordExportResult,
};
