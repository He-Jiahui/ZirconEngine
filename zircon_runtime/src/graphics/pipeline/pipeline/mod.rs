//! RenderPipelineAsset 的编译、默认资产、资源描述和 pass authoring 子模块边界。
//! 只有编译上下文在此模块根导出；具体阶段实现保持在相应子模块内。
#[path = "attachment_init.rs"]
mod attachment_initialization;
#[path = "builtin.rs"]
mod builtin;
#[path = "compile.rs"]
mod compile;
#[cfg(test)]
#[path = "tests/compile.rs"]
mod compile_tests;
#[path = "compile_with_asset_context.rs"]
mod compile_with_asset_context;
#[path = "default_core2d.rs"]
mod default_core2d;
#[path = "default_deferred.rs"]
mod default_deferred;
#[path = "default_forward_plus.rs"]
mod default_forward_plus;
#[path = "descriptor_filtering.rs"]
mod descriptor_filtering;
#[path = "graph_resources.rs"]
mod graph_resources;
#[path = "half_resolution_transparency.rs"]
mod half_resolution_transparency;
#[path = "pass_authoring.rs"]
mod pass_authoring;
#[path = "plugin_render_features.rs"]
mod plugin_render_features;
#[path = "resource_descriptors.rs"]
mod resource_descriptors;
#[path = "resource_schema_catalog.rs"]
mod resource_schema_catalog;
#[cfg(test)]
#[path = "../tests/shadow_atlas_required_external.rs"]
mod shadow_atlas_required_external_tests;
#[path = "ssao_input_qualification.rs"]
mod ssao_input_qualification;
#[cfg(test)]
#[path = "tests/typed_optional_external.rs"]
mod typed_optional_external_tests;

pub use compile_with_asset_context::RenderPipelineAssetContext;
