//! 负责作者资产的加载、契约检查、编译缓存与热重载归属。
//! 校验报告、资源解析和运行时发布分别消费这些结果；编译成功本身不注册正在使用资产的表面。
#[path = "action_policy/mod.rs"]
mod action_policy;
#[path = "binding/mod.rs"]
mod binding;
#[path = "binding_reload_transaction.rs"]
mod binding_reload_transaction;
#[path = "compiler/mod.rs"]
mod compiler;
#[path = "component_contract/mod.rs"]
mod component_contract;
#[path = "dependency_index.rs"]
mod dependency_index;
#[path = "document.rs"]
mod document;
#[path = "hot_reload_executor.rs"]
mod hot_reload_executor;
#[path = "hot_reload_plan.rs"]
mod hot_reload_plan;
#[path = "invalidation/mod.rs"]
mod invalidation;
#[path = "loader.rs"]
mod loader;
#[path = "localization/mod.rs"]
mod localization;
#[path = "prototype_file_cache.rs"]
mod prototype_file_cache;
#[path = "prototype_store.rs"]
mod prototype_store;
#[path = "resource_ref/mod.rs"]
mod resource_ref;
#[path = "schema/mod.rs"]
mod schema;
#[path = "style.rs"]
mod style;
#[path = "surface_index.rs"]
mod surface_index;
#[path = "watch_invalidation.rs"]
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
