mod action_policy;
mod binding;
mod binding_reload_transaction;
mod compiler;
mod component_contract;
mod dependency_index;
mod document;
mod hot_reload_executor;
mod hot_reload_plan;
mod invalidation;
mod loader;
mod localization;
mod prototype_file_cache;
mod prototype_store;
mod resource_ref;
mod schema;
mod style;
mod surface_index;
mod watch_invalidation;

pub use action_policy::validate_document_action_policy;
pub use binding::{collect_asset_binding_report, validate_asset_bindings};
pub(crate) use binding_reload_transaction::UiBindingReloadTransaction;
pub use binding_reload_transaction::{UiBindingQuiescenceReceipt, UiBindingReloadPrepareError};
pub(crate) use compiler::{
    compile_binding_program, resolve_component_binding_params, resolve_component_param_value,
    resolve_component_param_value_map, validate_typed_component_params,
};
pub use compiler::{
    compile_cache_key_from_compiler, compiled_asset_package_manifest_from_artifact_bytes,
    UiAssetCompileCache, UiAssetCompileCacheEvictionReport, UiCompileCacheOutcome,
    UiCompiledArtifactKey, UiCompiledArtifactStore, UiCompiledArtifactStoreEvictionReport,
    UiCompiledDocument, UiDocumentCompiler, UiRuntimeCompiledAssetArtifact, UiStyleResolver,
};
pub use component_contract::component_contract_diagnostic;
pub use dependency_index::{UiAssetDependencyIndex, UiAssetDependencyQueryReport};
pub use document::{UiAssetDocumentRuntimeExt, UiAssetNodeIter, UiNodeParent};
pub use hot_reload_executor::{
    UiAssetHotReloadExecutionError, UiAssetHotReloadExecutionReport, UiAssetHotReloadExecutor,
    UiAssetSurfaceRebuildRequest, UiAssetSurfaceRebuilder, UiAssetTemplateRebuildReceipt,
};
pub use hot_reload_plan::{
    classify_ui_hot_reload_asset, UiAssetHotReloadPlan, UiAssetHotReloadSurfaceDirtyReport,
    UiHotReloadAssetKind,
};
pub use invalidation::{
    collect_invalidation_diagnostics, component_contract_fingerprint, declared_imports_fingerprint,
    document_import_fingerprints, fingerprint_document, resource_dependencies_fingerprint,
    UiInvalidationGraph, BROAD_SELECTOR_WARNING_THRESHOLD, LARGE_DOCUMENT_NODE_WARNING_THRESHOLD,
    NON_VIRTUALIZED_SCROLL_CHILD_WARNING_THRESHOLD,
};
pub use loader::UiAssetLoader;
pub use localization::{
    collect_document_localization_report, localization_table_keys_from_toml_str,
    validate_document_localization, validate_localization_report_against_catalog,
    UiLocalizationTableCatalog,
};
pub use prototype_file_cache::{UiPrototypeStoreFileCache, UiPrototypeStoreLoadOutcome};
pub use prototype_store::{UiPrototypeStore, UiPrototypeStoreBuilder};
pub use resource_ref::{
    collect_document_resource_dependencies, validate_resource_dependency_files,
    UiResolvedResourceDependency, UiResolvedUiResource, UiResourcePathResolver,
    UiResourceResolutionReport, UiResourceResolveDiagnostic, UiResourceResolveDiagnosticCode,
    UiResourceResolver, UiResourceResolverCacheInvalidationReport, UiResourceResolverSchemeMap,
};
pub use schema::UiAssetSchemaMigrator;
pub use surface_index::{
    UiAssetBindingTarget, UiAssetCompiledNodeTarget, UiAssetHotReloadNodeDirtyReport,
    UiAssetNodeHotReloadTargets, UiAssetNodeTarget, UiAssetSurfaceHotReloadApplyReport,
    UiAssetSurfaceHotReloadTargets, UiAssetSurfaceIndex,
    UiAssetSurfaceNodeResourceRegistrationReport,
};
pub use watch_invalidation::UiAssetWatchInvalidationReport;
