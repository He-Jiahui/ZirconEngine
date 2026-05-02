use zircon_runtime::graphics::{
    FrameHistoryBinding, FrameHistorySlot, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassExecutionContext, RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

pub const FEATURE_ID: &str = "rendering.ssao";
pub const FEATURE_NAME: &str = "screen_space_ambient_occlusion";
pub const EXECUTOR_ID: &str = "ao.ssao-evaluate";

#[derive(Clone, Debug)]
pub struct RenderingSsaoRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for RenderingSsaoRuntimeFeature {
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

pub fn runtime_plugin_feature() -> RenderingSsaoRuntimeFeature {
    RenderingSsaoRuntimeFeature
}

pub fn plugin_feature_registration() -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(&runtime_plugin_feature())
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_runtime::feature_manifest(
        zircon_plugin_rendering_runtime::RenderingFeatureKind::Ssao,
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        FEATURE_NAME,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::AmbientOcclusion,
        )],
        vec![RenderFeaturePassDescriptor::new(
            RenderPassStage::AmbientOcclusion,
            "ssao-evaluate",
            QueueLane::AsyncCompute,
        )
        .with_executor_id(EXECUTOR_ID)
        .read_texture("scene-depth")
        .write_texture("ambient-occlusion")],
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
    fn ssao_feature_registers_history_binding() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert_eq!(report.manifest.id, FEATURE_ID);
        assert!(report.manifest.enabled_by_default);
        assert_eq!(
            report.extensions.render_features()[0].history_bindings,
            vec![FrameHistoryBinding::read_write(
                FrameHistorySlot::AmbientOcclusion
            )]
        );
    }
}
