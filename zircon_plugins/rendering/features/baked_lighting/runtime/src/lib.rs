use zircon_runtime::graphics::{
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutionContext,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

pub const FEATURE_ID: &str = "rendering.baked_lighting";
pub const FEATURE_NAME: &str = "baked_lighting";
pub const EXECUTOR_ID: &str = "lighting.baked-composite";

#[derive(Clone, Debug)]
pub struct RenderingBakedLightingRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for RenderingBakedLightingRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_render_feature(render_feature_descriptor())?;
        registry.register_render_pass_executor(render_pass_executor_registration())
    }
}

pub fn runtime_plugin_feature() -> RenderingBakedLightingRuntimeFeature {
    RenderingBakedLightingRuntimeFeature
}

pub fn plugin_feature_registration() -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(&runtime_plugin_feature())
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_runtime::feature_manifest(
        zircon_plugin_rendering_runtime::RenderingFeatureKind::BakedLighting,
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec!["lighting".to_string(), "post_process".to_string()],
        Vec::new(),
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::PostProcess,
            "baked-lighting-composite",
            QueueLane::Graphics,
        )
        .with_executor_id(EXECUTOR_ID)
        .read_texture("scene-color")
        .write_texture("scene-color")],
    )
}

pub fn render_pass_executor_registration() -> RenderPassExecutorRegistration {
    RenderPassExecutorRegistration::new(EXECUTOR_ID, noop_render_executor)
}

fn noop_render_executor(_context: &RenderPassExecutionContext) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baked_lighting_feature_registers_composite_pass() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report.manifest.enabled_by_default);
        assert_eq!(
            report.extensions.render_features()[0].stage_passes[0].pass_name,
            "baked-lighting-composite"
        );
    }
}
