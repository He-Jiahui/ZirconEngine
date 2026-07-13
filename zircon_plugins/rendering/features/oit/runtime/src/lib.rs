use zircon_runtime::core::framework::render::PostProcessGraphResourceNames;
use zircon_runtime::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::{QueueLane, RenderGraphAttachmentOps};

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingOitRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.oit";
pub const FEATURE_NAME: &str = "oit";
pub const FRAGMENT_STORE_PASS: &str = "oit.fragment_store";
pub const RESOLVE_PASS: &str = "oit.resolve";
pub const FRAGMENT_STORE_EXECUTOR: &str = "oit.fragment_store";
pub const RESOLVE_EXECUTOR: &str = "oit.resolve";

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "advanced_lighting".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                FRAGMENT_STORE_PASS,
                QueueLane::Graphics,
            )
            .with_executor_id(FRAGMENT_STORE_EXECUTOR)
            .read_texture(PostProcessGraphResourceNames::SCENE_DEPTH)
            .read_required_external_texture(PostProcessGraphResourceNames::SHADOW_ATLAS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_GRID_PARAMS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_ZBINS)
            .read_buffer(PostProcessGraphResourceNames::LIGHT_TILE_MASKS)
            .write_buffer(PostProcessGraphResourceNames::OIT_LAYERS)
            .write_buffer(PostProcessGraphResourceNames::OIT_COUNTS),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Transparent3d,
                RESOLVE_PASS,
                QueueLane::Graphics,
            )
            .with_executor_id(RESOLVE_EXECUTOR)
            .read_buffer(PostProcessGraphResourceNames::OIT_LAYERS)
            .read_buffer(PostProcessGraphResourceNames::OIT_COUNTS)
            .write_texture_with_ops(
                PostProcessGraphResourceNames::SCENE_COLOR,
                RenderGraphAttachmentOps::load_store(),
            ),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::StorageBuffers)
    .when_advanced_lighting_oit_enabled()
    .with_replaced_pass("transparent-mesh")
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    zircon_runtime::graphics::oit_render_pass_executor_registrations()
}

#[cfg(test)]
mod tests;
