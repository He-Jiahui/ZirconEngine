use super::{
    RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
    RenderVirtualGeometryReadbackOutputs,
};

/// Neutral sideband data prepared by runtime providers before the renderer runs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderPreparedRuntimeSidebands {
    pub plugin_renderer_outputs: RenderPluginRendererOutputs,
    pub hybrid_gi_evictable_probe_ids: Vec<u32>,
    pub virtual_geometry_evictable_page_ids: Vec<u32>,
}

impl RenderPreparedRuntimeSidebands {
    pub fn new(
        plugin_renderer_outputs: RenderPluginRendererOutputs,
        hybrid_gi_evictable_probe_ids: Vec<u32>,
        virtual_geometry_evictable_page_ids: Vec<u32>,
    ) -> Self {
        Self {
            plugin_renderer_outputs,
            hybrid_gi_evictable_probe_ids,
            virtual_geometry_evictable_page_ids,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.plugin_renderer_outputs.is_empty()
            && self.hybrid_gi_evictable_probe_ids.is_empty()
            && self.virtual_geometry_evictable_page_ids.is_empty()
    }

    pub fn hybrid_gi_readback_outputs(&self) -> &RenderHybridGiReadbackOutputs {
        &self.plugin_renderer_outputs.hybrid_gi
    }

    pub fn virtual_geometry_readback_outputs(&self) -> &RenderVirtualGeometryReadbackOutputs {
        &self.plugin_renderer_outputs.virtual_geometry
    }

    pub fn particle_readback_outputs(&self) -> &RenderParticleGpuReadbackOutputs {
        &self.plugin_renderer_outputs.particles
    }

    pub fn hybrid_gi_evictable_probe_ids(&self) -> &[u32] {
        &self.hybrid_gi_evictable_probe_ids
    }

    pub fn virtual_geometry_evictable_page_ids(&self) -> &[u32] {
        &self.virtual_geometry_evictable_page_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_runtime_sidebands_report_empty_only_without_payloads() {
        assert!(RenderPreparedRuntimeSidebands::default().is_empty());

        let sidebands = RenderPreparedRuntimeSidebands::new(
            RenderPluginRendererOutputs::default(),
            vec![5],
            Vec::new(),
        );

        assert!(!sidebands.is_empty());
        assert_eq!(sidebands.hybrid_gi_evictable_probe_ids(), &[5]);
    }
}
