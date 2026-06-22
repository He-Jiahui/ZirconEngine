use crate::core::framework::render::{RenderPluginRendererOutputs, RenderPreparedRuntimeSidebands};

#[derive(Default)]
pub(super) struct PreparedRuntimeSubmission {
    hybrid_gi_evictable_probe_ids: Vec<u32>,
    virtual_geometry_evictable_page_ids: Vec<u32>,
    plugin_renderer_outputs: RenderPluginRendererOutputs,
}

impl PreparedRuntimeSubmission {
    pub(super) fn new(
        hybrid_gi_evictable_probe_ids: Vec<u32>,
        virtual_geometry_evictable_page_ids: Vec<u32>,
        plugin_renderer_outputs: RenderPluginRendererOutputs,
    ) -> Self {
        Self {
            hybrid_gi_evictable_probe_ids,
            virtual_geometry_evictable_page_ids,
            plugin_renderer_outputs,
        }
    }

    pub(super) fn into_prepared_runtime_sidebands(self) -> RenderPreparedRuntimeSidebands {
        RenderPreparedRuntimeSidebands::new(
            self.plugin_renderer_outputs,
            self.hybrid_gi_evictable_probe_ids,
            self.virtual_geometry_evictable_page_ids,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs,
        RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn prepared_submission_carries_plugin_renderer_output_sideband() {
        let prepared = PreparedRuntimeSubmission::new(
            vec![5],
            vec![9],
            RenderPluginRendererOutputs {
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![11],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 4,
                    indirect_draw_args: [6, 4, 0, 0],
                    ..RenderParticleGpuReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        let sidebands = prepared.into_prepared_runtime_sidebands();

        assert_eq!(
            sidebands.hybrid_gi_readback_outputs().completed_probe_ids,
            vec![11]
        );
        assert_eq!(
            sidebands
                .virtual_geometry_readback_outputs()
                .node_cluster_cull
                .page_request_ids,
            vec![300]
        );
        assert_eq!(sidebands.particle_readback_outputs().alive_count, 4);
        assert_eq!(sidebands.hybrid_gi_evictable_probe_ids(), &[5]);
        assert_eq!(sidebands.virtual_geometry_evictable_page_ids(), &[9]);
    }

    #[test]
    fn prepared_submission_projects_neutral_runtime_sidebands() {
        let prepared = PreparedRuntimeSubmission::new(
            vec![5],
            vec![9],
            RenderPluginRendererOutputs {
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![11],
                    ..RenderHybridGiReadbackOutputs::default()
                },
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 6,
                    indirect_draw_args: [6, 6, 0, 0],
                    ..RenderParticleGpuReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        let sidebands = prepared.into_prepared_runtime_sidebands();

        assert_eq!(
            sidebands.hybrid_gi_readback_outputs().completed_probe_ids,
            vec![11]
        );
        assert_eq!(
            sidebands
                .virtual_geometry_readback_outputs()
                .node_cluster_cull
                .page_request_ids,
            vec![300]
        );
        assert_eq!(sidebands.hybrid_gi_evictable_probe_ids(), &[5]);
        assert_eq!(sidebands.virtual_geometry_evictable_page_ids(), &[9]);
        assert_eq!(sidebands.particle_readback_outputs().alive_count, 6);
    }
}
