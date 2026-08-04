mod builtin;
mod compile;
#[cfg(test)]
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
#[cfg(test)]
mod shadow_atlas_required_external_tests;
#[cfg(test)]
mod typed_optional_external_tests;

pub use compile_with_asset_context::RenderPipelineAssetContext;
