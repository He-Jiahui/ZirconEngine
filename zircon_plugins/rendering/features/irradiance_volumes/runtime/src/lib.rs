use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutorRegistration,
    RenderPassStage, IRRADIANCE_VOLUME_BIND_EXECUTOR_ID, IRRADIANCE_VOLUME_RESOURCE,
};
use zircon_runtime::render_graph::QueueLane;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingIrradianceVolumesRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.irradiance_volumes";
pub const FEATURE_NAME: &str = "irradiance_volumes";
pub const VOLUME_BIND_PASS: &str = "irradiance.volume_bind";

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "advanced_lighting".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::DepthPrepass,
            VOLUME_BIND_PASS,
            QueueLane::Graphics,
        )
        .with_executor_id(IRRADIANCE_VOLUME_BIND_EXECUTOR_ID)
        .with_side_effects()
        .write_external_texture(IRRADIANCE_VOLUME_RESOURCE)],
    )
    .when_advanced_lighting_irradiance_volumes_enabled()
    .with_pass_read_external_texture("opaque-mesh", IRRADIANCE_VOLUME_RESOURCE)
    .with_pass_read_external_texture("alpha-mask-mesh", IRRADIANCE_VOLUME_RESOURCE)
    .with_pass_read_external_texture("deferred-gbuffer", IRRADIANCE_VOLUME_RESOURCE)
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    zircon_runtime::graphics::irradiance_volume_render_pass_executor_registrations()
}

#[cfg(test)]
mod tests;
