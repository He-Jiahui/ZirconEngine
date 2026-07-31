mod asset_ref;
mod manifest_summary;
mod persisted_asset_reference;
mod project_name;
mod rel_path;
mod retired_asset_ref_migration;
mod template_pack;

pub use asset_ref::{AssetRef, AssetRefError};
pub use manifest_summary::{
    PROJECT_MANIFEST_FORMAT_VERSION, ProjectManifestSummary, ProjectManifestSummaryError,
    load_project_manifest_value_from_toml_str, validate_engine_version_req,
};
pub use persisted_asset_reference::{PersistedAssetReference, PersistedAssetReferenceError};
pub use project_name::{ProjectNameError, validate_project_name};
pub use rel_path::{RelPath, RelPathError};
pub use retired_asset_ref_migration::{
    RetiredAssetRefMigrationError, RetiredAssetReference, migrate_retired_asset_references,
    migrate_retired_asset_references_with, migrate_retired_persisted_asset_reference_with,
    migrate_retired_persisted_asset_references_with,
};
pub use template_pack::{
    ProjectTemplateId, ProjectTemplatePackError, RenderedProjectTemplate,
    RenderedProjectTemplateEntry, render_project_template,
};

#[cfg(test)]
mod tests;
