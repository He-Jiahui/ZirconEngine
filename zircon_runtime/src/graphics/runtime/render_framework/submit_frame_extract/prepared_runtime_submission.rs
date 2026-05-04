use crate::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
    RenderPreparedRuntimeSidebands, RenderVirtualGeometryReadbackOutputs,
};

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

    pub(super) fn hybrid_gi_readback_outputs(&self) -> &RenderHybridGiReadbackOutputs {
        &self.plugin_renderer_outputs.hybrid_gi
    }

    pub(super) fn virtual_geometry_readback_outputs(
        &self,
    ) -> &RenderVirtualGeometryReadbackOutputs {
        &self.plugin_renderer_outputs.virtual_geometry
    }

    pub(super) fn particle_readback_outputs(&self) -> &RenderParticleGpuReadbackOutputs {
        &self.plugin_renderer_outputs.particles
    }

    pub(super) fn prepared_runtime_sidebands(&self) -> RenderPreparedRuntimeSidebands {
        RenderPreparedRuntimeSidebands::new(
            self.plugin_renderer_outputs.clone(),
            self.hybrid_gi_evictable_probe_ids.clone(),
            self.virtual_geometry_evictable_page_ids.clone(),
        )
    }

    pub(super) fn take_hybrid_gi_evictable_probe_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.hybrid_gi_evictable_probe_ids)
    }

    pub(super) fn take_virtual_geometry_evictable_page_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.virtual_geometry_evictable_page_ids)
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
        let mut prepared = PreparedRuntimeSubmission::new(
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

        assert_eq!(
            prepared.hybrid_gi_readback_outputs().completed_probe_ids,
            vec![11]
        );
        assert_eq!(
            prepared
                .virtual_geometry_readback_outputs()
                .node_cluster_cull
                .page_request_ids,
            vec![300]
        );
        assert_eq!(prepared.particle_readback_outputs().alive_count, 4);
        assert_eq!(prepared.take_hybrid_gi_evictable_probe_ids(), vec![5]);
        assert_eq!(prepared.take_virtual_geometry_evictable_page_ids(), vec![9]);
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

        let sidebands = prepared.prepared_runtime_sidebands();

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
