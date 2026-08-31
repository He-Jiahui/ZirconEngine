use zircon_runtime::graphics::RenderFeatureDescriptor;

mod capability;
mod plugin;

pub use capability::{EDITOR_CAPABILITY, RUNTIME_CAPABILITIES, RUNTIME_CAPABILITY};
pub use plugin::{
    feature_manifest, plugin_feature_registration, runtime_plugin_feature,
    RenderingSsaoRuntimeFeature,
};

pub const FEATURE_ID: &str = "rendering.ssao";
pub const FEATURE_NAME: &str = "screen_space_ambient_occlusion";
pub const EXECUTOR_ID: &str = "compute.generic";

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    zircon_runtime::graphics::screen_space_ambient_occlusion_render_feature_descriptor()
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::framework::render::PostProcessGraphResourceNames;
    use zircon_runtime::graphics::{FrameHistoryBinding, FrameHistorySlot};
    use zircon_runtime::render_graph::{QueueLane, RenderGraphComputeDispatchExtent};

    #[test]
    fn ssao_feature_registers_history_binding() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert_eq!(report.manifest.id, FEATURE_ID);
        assert!(!report.manifest.enabled_by_default);
        assert_eq!(
            report.extensions.render_features()[0].history_bindings,
            vec![FrameHistoryBinding::read_write(
                FrameHistorySlot::AmbientOcclusion
            )]
        );
        let pass = &report.extensions.render_features()[0].stage_passes[0];
        let workload = pass
            .compute_workload
            .as_ref()
            .expect("ssao async compute pass should declare workload");
        assert_eq!(pass.queue, QueueLane::AsyncCompute);
        assert_eq!(workload.pipeline_label, "zircon-ssao-pipeline");
        assert_eq!(workload.workgroup_size, [8, 8, 1]);
        assert_eq!(
            workload.dispatch_extent,
            RenderGraphComputeDispatchExtent::PerPixel {
                target: "ambient-occlusion".to_string(),
                local_size: [8, 8],
            }
        );
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::HISTORY_PREVIOUS_AMBIENT_OCCLUSION
                && resource.usage.persistent
        }));
        assert!(pass.resources.iter().any(|resource| {
            resource.name == PostProcessGraphResourceNames::AMBIENT_OCCLUSION
                && resource.usage.persistent
                && resource.schema
                    == Some(zircon_runtime::graphics::RenderResourceSchema::texture(
                        zircon_runtime::graphics::RenderTextureSchema::new(
                            zircon_runtime::rhi::TextureFormat::Rgba8Unorm,
                            zircon_runtime::rhi::TextureUsage::SAMPLED
                                | zircon_runtime::rhi::TextureUsage::STORAGE,
                        ),
                    ))
        }));
    }
}
