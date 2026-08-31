use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutorRegistration,
    RenderPassStage, LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID, LIGHT_COOKIE_ATLAS_RESOURCE,
};
use zircon_runtime::render_graph::QueueLane;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingLightCookiesRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.light_cookies";
pub const FEATURE_NAME: &str = "light_cookies";
pub const ATLAS_BUILD_PASS: &str = "cookie.atlas_build";

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
            ATLAS_BUILD_PASS,
            QueueLane::Graphics,
        )
        .with_executor_id(LIGHT_COOKIE_ATLAS_BUILD_EXECUTOR_ID)
        .write_external_texture(LIGHT_COOKIE_ATLAS_RESOURCE)],
    )
    .when_advanced_lighting_cookies_enabled()
    .with_pass_read_external_texture("opaque-mesh", LIGHT_COOKIE_ATLAS_RESOURCE)
    .with_pass_read_external_texture("alpha-mask-mesh", LIGHT_COOKIE_ATLAS_RESOURCE)
    .with_pass_read_external_texture("transparent-mesh", LIGHT_COOKIE_ATLAS_RESOURCE)
    .with_pass_read_external_texture("deferred-lighting", LIGHT_COOKIE_ATLAS_RESOURCE)
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    zircon_runtime::graphics::light_cookie_render_pass_executor_registrations()
}

#[cfg(test)]
mod tests;
