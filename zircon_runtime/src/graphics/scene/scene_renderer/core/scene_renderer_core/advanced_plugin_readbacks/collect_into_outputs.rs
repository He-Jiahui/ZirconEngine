use crate::graphics::scene::scene_renderer::core::scene_renderer::SceneRendererAdvancedPluginOutputs;
use crate::graphics::types::GraphicsError;

use super::scene_renderer_advanced_plugin_readbacks::SceneRendererAdvancedPluginReadbacks;

impl SceneRendererAdvancedPluginReadbacks {
    pub(in crate::graphics::scene::scene_renderer::core) fn collect_into_outputs(
        self,
        outputs: &mut SceneRendererAdvancedPluginOutputs,
    ) -> Result<(), GraphicsError> {
        self.collect_neutral_outputs_into(outputs);
        Ok(())
    }

    fn collect_neutral_outputs_into(mut self, outputs: &mut SceneRendererAdvancedPluginOutputs) {
        outputs.store_plugin_renderer_outputs(std::mem::take(&mut self.outputs));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs,
        RenderPluginRendererOutputs, RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn advanced_plugin_readbacks_collect_neutral_plugin_renderer_outputs() {
        let mut outputs = SceneRendererAdvancedPluginOutputs::default();
        let readbacks =
            SceneRendererAdvancedPluginReadbacks::from_outputs(RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    page_table_entries: vec![1, 2, 3],
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![7, 9],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 5,
                    spawned_total: 3,
                    debug_flags: 7,
                    per_emitter_spawned: vec![2, 1],
                    indirect_draw_args: [6, 5, 0, 0],
                },
                ..RenderPluginRendererOutputs::default()
            });

        readbacks.collect_neutral_outputs_into(&mut outputs);

        assert_eq!(
            outputs
                .plugin_renderer_outputs()
                .virtual_geometry
                .page_table_entries,
            vec![1, 2, 3]
        );
        assert_eq!(
            outputs
                .plugin_renderer_outputs()
                .hybrid_gi
                .completed_probe_ids,
            vec![7, 9]
        );
        assert_eq!(
            outputs
                .plugin_renderer_outputs()
                .particles
                .indirect_draw_args,
            [6, 5, 0, 0]
        );
    }

    #[test]
    fn cpu_only_plugin_owners_do_not_borrow_the_native_device() {
        let readback_source = include_str!("collect_into_outputs.rs");
        let readback_owner = readback_source
            .split("fn collect_into_outputs(")
            .nth(1)
            .and_then(|source| source.split("fn collect_neutral_outputs_into(").next())
            .expect("advanced-plugin readback publication owner");
        let resource_source = include_str!(
            "../advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs"
        );
        let resource_owner = resource_source
            .split("fn new(")
            .nth(1)
            .and_then(|source| {
                source
                    .split("fn register_runtime_prepare_collector(")
                    .next()
            })
            .expect("advanced-plugin CPU capability owner");
        let output_caller =
            include_str!("../../scene_renderer_runtime_outputs/store_last_runtime_outputs.rs");
        let construct_source =
            include_str!("../../scene_renderer_core_construct/construct/construct.rs");
        let construct_caller = construct_source
            .split("let advanced_plugin_resources =")
            .nth(1)
            .and_then(|source| source.split("let volumetric_fog_enabled =").next())
            .expect("advanced-plugin capability construction caller");
        let runtime_prepare_context = include_str!("../../../../../runtime_prepare_collector.rs");
        let root_context = runtime_prepare_context
            .split("pub struct RuntimePrepareCollectorContext")
            .nth(1)
            .and_then(|source| {
                source
                    .split("/// Runtime-prepare capability for CPU writes")
                    .next()
            })
            .expect("runtime-prepare root context");

        assert!(!readback_owner.contains("wgpu::Device"));
        assert!(!resource_owner.contains("wgpu::Device"));
        assert!(!output_caller.contains("renderer.backend.device"));
        assert!(!construct_caller.contains("device,"));
        assert!(runtime_prepare_context.contains("pub fn gpu_recording_context("));
        assert!(!root_context.contains("pub device:"));
        assert!(readback_owner.contains("collect_neutral_outputs_into(outputs)"));
    }
}
