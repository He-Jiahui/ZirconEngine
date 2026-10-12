//! 将文档和共享原型编译成可重复实例化的模板；导入解析由宿主先完成，实际树安装与绑定执行由构建层负责。
#[path = "binding_param_resolver.rs"]
mod binding_param_resolver;
#[path = "binding_program.rs"]
mod binding_program;
#[path = "cache/mod.rs"]
mod cache;
#[path = "compile.rs"]
mod compile;
#[path = "component_instance_expander.rs"]
mod component_instance_expander;
#[path = "component_props.rs"]
mod component_props;
#[path = "control_scope.rs"]
mod control_scope;
#[path = "node_expander.rs"]
mod node_expander;
#[path = "package/mod.rs"]
mod package;
#[path = "prototype_instancer.rs"]
mod prototype_instancer;
#[path = "shape_validator.rs"]
mod shape_validator;
#[path = "style_apply.rs"]
mod style_apply;
#[path = "ui_document_compiler.rs"]
mod ui_document_compiler;
#[path = "ui_style_resolver.rs"]
mod ui_style_resolver;
#[path = "value_normalizer.rs"]
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
