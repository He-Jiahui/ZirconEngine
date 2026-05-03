use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
use crate::core::framework::render::RenderPluginRendererOutputs;

impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn store_plugin_renderer_outputs(
        &mut self,
        outputs: RenderPluginRendererOutputs,
    ) {
        *self.plugin_renderer_outputs_mut() = outputs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs,
        RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
        RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn stores_neutral_plugin_renderer_outputs() {
        let mut outputs = SceneRendererAdvancedPluginOutputs::default();
        let plugin_outputs = RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                page_table_entries: vec![1, 2, 3],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            hybrid_gi: RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![7, 9],
                ..RenderHybridGiReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        };

        outputs.store_plugin_renderer_outputs(plugin_outputs.clone());

        assert_eq!(outputs.plugin_renderer_outputs(), &plugin_outputs);
        assert!(outputs.has_virtual_geometry_gpu_readback());
    }

    #[test]
    fn detects_completion_only_virtual_geometry_readback_outputs() {
        let mut outputs = SceneRendererAdvancedPluginOutputs::default();
        outputs.store_plugin_renderer_outputs(RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                completed_page_assignments: vec![RenderVirtualGeometryPageAssignmentRecord {
                    page_id: 42,
                    physical_slot: 3,
                }],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        });

        assert!(outputs.has_virtual_geometry_gpu_readback());
    }

    #[test]
    fn detects_node_cluster_cull_page_request_readback_outputs() {
        let mut outputs = SceneRendererAdvancedPluginOutputs::default();
        outputs.store_plugin_renderer_outputs(RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                    page_request_ids: vec![300, 301],
                    ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                },
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        });

        assert!(outputs.has_virtual_geometry_gpu_readback());
    }

    #[test]
    fn stores_and_takes_particle_gpu_readback_outputs_without_clearing_other_outputs() {
        let mut outputs = SceneRendererAdvancedPluginOutputs::default();
        let particle_outputs = RenderParticleGpuReadbackOutputs {
            alive_count: 5,
            spawned_total: 3,
            debug_flags: 7,
            per_emitter_spawned: vec![2, 1],
            indirect_draw_args: [6, 5, 0, 0],
        };

        outputs.store_plugin_renderer_outputs(RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                page_table_entries: vec![9],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            particles: particle_outputs.clone(),
            ..RenderPluginRendererOutputs::default()
        });

        assert!(outputs.has_particle_gpu_readback());
        assert!(outputs.has_virtual_geometry_gpu_readback());
        assert_eq!(
            outputs.plugin_renderer_outputs().particles,
            particle_outputs
        );

        let taken = outputs.take_particle_gpu_readback_outputs();

        assert_eq!(taken, particle_outputs);
        assert!(!outputs.has_particle_gpu_readback());
        assert!(outputs.has_virtual_geometry_gpu_readback());
    }
}
