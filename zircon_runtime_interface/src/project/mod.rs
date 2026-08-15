mod asset_ref;
mod manifest_summary;
mod persisted_asset_reference;
mod project_name;
mod rel_path;
mod retired_asset_ref_migration;
pub mod session_lock;
mod template_pack;

pub use asset_ref::{AssetRef, AssetRefError};
pub use manifest_summary::{
    load_project_manifest_value_from_toml_str, validate_engine_version_req, ProjectManifestSummary,
    ProjectManifestSummaryError, PROJECT_MANIFEST_FORMAT_VERSION,
};
pub use persisted_asset_reference::{PersistedAssetReference, PersistedAssetReferenceError};
pub use project_name::{validate_project_name, ProjectNameError};
pub use rel_path::{RelPath, RelPathError};
pub use retired_asset_ref_migration::{
    migrate_retired_asset_references, migrate_retired_asset_references_with,
    migrate_retired_persisted_asset_reference_with,
    migrate_retired_persisted_asset_references_with, RetiredAssetRefMigrationError,
    RetiredAssetReference,
};
pub use template_pack::{
    render_project_template, ProjectTemplateId, ProjectTemplatePackError, RenderedProjectTemplate,
    RenderedProjectTemplateEntry,
};

#[cfg(test)]
mod tests;
