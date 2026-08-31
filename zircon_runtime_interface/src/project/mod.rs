mod activation_operation_id;
mod asset_ref;
mod canonical_descriptor_identity;
mod engine_compatibility;
mod manifest_digest;
mod manifest_digest_parse_error;
mod manifest_summary;
mod persisted_asset_reference;
mod project_guid;
mod project_guid_parse_error;
mod project_identity;
mod project_launch_intent;
mod project_launch_intent_error;
mod project_name;
mod rel_path;
mod retired_asset_ref_migration;
pub mod session_lock;
mod template_pack;

pub use activation_operation_id::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator,
    ProjectActivationOperationSequence, ProjectLaunchInstanceId,
};
pub use asset_ref::{AssetRef, AssetRefError};
pub use canonical_descriptor_identity::{
    CanonicalDescriptorIdentity, CanonicalDescriptorIdentityError,
};
pub use engine_compatibility::{
    assess_project_engine_compatibility, ProjectEngineCompatibility,
    ProjectEngineCompatibilityDisposition, ProjectEngineCompatibilityError, ProjectEngineVersion,
    ProjectEngineVersionParseError,
};
pub use manifest_digest::ProjectManifestDigest;
pub use manifest_digest_parse_error::ProjectManifestDigestParseError;
pub use manifest_summary::{
    load_project_manifest_value_from_toml_str, validate_engine_version_req, ProjectManifestSummary,
    ProjectManifestSummaryError, MAX_PROJECT_ASSET_ROOTS, MAX_PROJECT_MANIFEST_ARRAY_ITEMS,
    MAX_PROJECT_MANIFEST_BYTES, MAX_PROJECT_MANIFEST_NESTING_DEPTH,
    MAX_PROJECT_MANIFEST_TABLE_ENTRIES, PROJECT_MANIFEST_FORMAT_VERSION,
};
pub use persisted_asset_reference::{PersistedAssetReference, PersistedAssetReferenceError};
pub use project_guid::ProjectGuid;
pub use project_guid_parse_error::ProjectGuidParseError;
pub use project_identity::ProjectIdentity;
pub use project_launch_intent::{
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource, ProjectLaunchTarget,
    PROJECT_LAUNCH_INTENT_SCHEMA_VERSION_V1,
};
pub use project_launch_intent_error::ProjectLaunchIntentError;
pub use project_name::{validate_project_name, ProjectNameError};
pub use rel_path::{RelPath, RelPathError};
pub use retired_asset_ref_migration::{
    migrate_retired_asset_references, migrate_retired_asset_references_with,
    migrate_retired_asset_references_with_budget, migrate_retired_persisted_asset_reference_with,
    migrate_retired_persisted_asset_references_with,
    migrate_retired_persisted_asset_references_with_budget, RetiredAssetRefMigrationBudget,
    RetiredAssetRefMigrationError, RetiredAssetReference, MAX_RETIRED_ASSET_REF_MIGRATION_DEPTH,
    MAX_RETIRED_ASSET_REF_MIGRATION_NODES, MAX_RETIRED_ASSET_REF_MIGRATION_REFERENCES,
};
pub use template_pack::{
    render_project_template, ProjectTemplateId, ProjectTemplatePackError, RenderedProjectTemplate,
    RenderedProjectTemplateEntry,
};

#[cfg(test)]
mod tests;
