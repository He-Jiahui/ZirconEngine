mod binding_param_resolver;
mod binding_program;
mod cache;
mod compile;
mod component_instance_expander;
mod component_props;
mod control_scope;
mod node_expander;
mod package;
mod prototype_instancer;
mod shape_validator;
mod style_apply;
mod ui_document_compiler;
mod ui_style_resolver;
mod value_normalizer;

pub use cache::{
    compile_cache_key_from_compiler, UiAssetCompileCache, UiAssetCompileCacheEvictionReport,
    UiCompileCacheOutcome, UiCompiledArtifactKey, UiCompiledArtifactStore,
    UiCompiledArtifactStoreEvictionReport,
};
pub use package::{
    compiled_asset_package_manifest_from_artifact_bytes, UiRuntimeCompiledAssetArtifact,
};
pub use ui_document_compiler::{UiCompiledDocument, UiDocumentCompiler};
pub use ui_style_resolver::UiStyleResolver;

pub(crate) use binding_param_resolver::{
    resolve_binding_params as resolve_component_binding_params,
    typed_component_params as validate_typed_component_params,
};
pub(crate) use binding_program::compile_binding_program;
pub(crate) use value_normalizer::{
    resolve_value as resolve_component_param_value,
    resolve_value_map as resolve_component_param_value_map,
};
