mod action_policy;
mod binding;
mod compiler;
mod component_contract;
mod document;
mod invalidation;
mod loader;
mod localization;
mod resource_ref;
mod schema;
mod style;

pub use action_policy::validate_document_action_policy;
pub use binding::{collect_asset_binding_report, validate_asset_bindings};
pub use compiler::{
    compile_cache_key_from_compiler, compiled_asset_package_manifest_from_artifact_bytes,
    UiAssetCompileCache, UiCompileCacheOutcome, UiCompiledDocument, UiDocumentCompiler,
    UiRuntimeCompiledAssetArtifact, UiStyleResolver,
};
pub use component_contract::component_contract_diagnostic;
pub use document::{UiAssetDocumentRuntimeExt, UiAssetNodeIter, UiNodeParent};
pub use invalidation::{
    collect_invalidation_diagnostics, component_contract_fingerprint, document_import_fingerprints,
    fingerprint_document, resource_dependencies_fingerprint, UiInvalidationGraph,
    BROAD_SELECTOR_WARNING_THRESHOLD, LARGE_DOCUMENT_NODE_WARNING_THRESHOLD,
    NON_VIRTUALIZED_SCROLL_CHILD_WARNING_THRESHOLD,
};
pub use loader::UiAssetLoader;
pub use localization::{
    collect_document_localization_report, localization_table_keys_from_toml_str,
    validate_document_localization, validate_localization_report_against_catalog,
    UiLocalizationTableCatalog,
};
pub use resource_ref::{
    collect_document_resource_dependencies, validate_resource_dependency_files,
    UiResourcePathResolver,
};
pub use schema::{
    UiAssetSchemaMigrator, UiAssetSchemaVersionPolicy, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
    UI_ASSET_MINIMUM_SUPPORTED_SOURCE_SCHEMA_VERSION,
};
