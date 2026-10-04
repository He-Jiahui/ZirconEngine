mod capability;
mod plugin;
mod render_feature;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingVolumetricFogRuntimeFeature,
};
pub use render_feature::{
    render_feature_descriptor, render_pass_executor_registrations, FEATURE_ID, FEATURE_NAME,
    FROXEL_WORKGROUP_SIZE, INTEGRATE_EXECUTOR, INTEGRATE_PASS, INTEGRATE_PIPELINE_LABEL,
    INTEGRATE_WORKGROUP_SIZE, LIGHT_SCATTER_EXECUTOR, LIGHT_SCATTER_PASS,
    LIGHT_SCATTER_PIPELINE_LABEL, MEDIA_INJECT_EXECUTOR, MEDIA_INJECT_PASS,
    MEDIA_INJECT_PIPELINE_LABEL,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/wgpu_product_tests.rs"]
mod wgpu_product_tests;
