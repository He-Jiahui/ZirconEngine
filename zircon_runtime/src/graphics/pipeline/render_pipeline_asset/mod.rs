//! RenderPipelineAsset 的编译、默认资产、资源描述和 pass authoring 子模块边界。
//! 只有编译上下文在此模块根导出；具体阶段实现保持在相应子模块内。
mod attachment_initialization;
mod builtin;
mod compile;
#[cfg(test)]
#[path = "tests/compile_tests.rs"]
mod compile_tests;
mod compile_with_asset_context;
mod default_core2d;
mod default_deferred;
mod default_forward_plus;
mod descriptor_filtering;
mod graph_resources;
mod half_resolution_transparency;
mod pass_authoring;
mod plugin_render_features;
mod resource_descriptors;
mod resource_schema_catalog;
#[cfg(test)]
#[path = "tests/shadow_atlas_required_external_tests.rs"]
mod shadow_atlas_required_external_tests;
mod ssao_input_qualification;
#[cfg(test)]
#[path = "tests/typed_optional_external_tests.rs"]
mod typed_optional_external_tests;

pub use compile_with_asset_context::RenderPipelineAssetContext;
