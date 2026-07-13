use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingDecalsRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.decals";
pub const FEATURE_NAME: &str = "decals";
pub const EXECUTOR_ID: &str = "decals.projector-composite";
pub const DECAL_PROJECTOR_COMPONENT_TYPE: &str = "rendering.Component.DecalProjector";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecalProjectionMode {
    ScreenSpace,
    Deferred,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DecalProjectorDescriptor {
    pub mode: DecalProjectionMode,
    pub opacity: f32,
    pub normal_blend: f32,
    pub atlas_region: String,
}

impl Default for DecalProjectorDescriptor {
    fn default() -> Self {
        Self {
            mode: DecalProjectionMode::Deferred,
            opacity: 1.0,
            normal_blend: 0.0,
            atlas_region: "default".to_string(),
        }
    }
}

pub fn decal_projector_component_descriptor(
) -> zircon_runtime::core::framework::scene::ComponentTypeDescriptor {
    zircon_runtime::core::framework::scene::ComponentTypeDescriptor::new(
        DECAL_PROJECTOR_COMPONENT_TYPE,
        zircon_plugin_rendering_runtime::PLUGIN_ID,
        "Decal Projector",
    )
    .with_property("mode", "enum:screen_space|deferred", true)
    .with_property("opacity", "float", true)
    .with_property("normal_blend", "float", true)
    .with_property("atlas_region", "string", true)
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "decals".to_string(),
        ],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "decal-projector-composite",
            QueueLane::Graphics,
        )
        .with_executor_id(EXECUTOR_ID)
        .read_texture("scene-depth")
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new(EXECUTOR_ID, noop_render_executor)
}

fn noop_render_executor(_context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decals_feature_registers_projector_component_and_pass() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(!report.manifest.enabled_by_default);
        assert_eq!(
            report.extensions.components()[0].type_id,
            DECAL_PROJECTOR_COMPONENT_TYPE
        );
        assert_eq!(
            report.extensions.render_features()[0].stage_passes[0].pass_name,
            "decal-projector-composite"
        );
    }
}
