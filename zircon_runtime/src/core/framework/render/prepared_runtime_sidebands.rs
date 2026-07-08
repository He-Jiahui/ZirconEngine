use super::{
    RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
    RenderVirtualGeometryReadbackOutputs,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedFrame {
    pub resident_probes: Vec<RenderHybridGiPreparedProbe>,
    pub pending_updates: Vec<RenderHybridGiPreparedUpdateRequest>,
    pub scheduled_trace_region_ids: Vec<u32>,
    pub evictable_probe_ids: Vec<u32>,
    pub probe_scene_data: Vec<RenderHybridGiPreparedProbeSceneData>,
    pub probe_rt_lighting_rgb: Vec<RenderHybridGiPreparedProbeRtLighting>,
    pub trace_region_scene_data: Vec<RenderHybridGiPreparedTraceRegionSceneData>,
}

impl RenderHybridGiPreparedFrame {
    pub fn is_empty(&self) -> bool {
        self.resident_probes.is_empty()
            && self.pending_updates.is_empty()
            && self.scheduled_trace_region_ids.is_empty()
            && self.evictable_probe_ids.is_empty()
            && self.probe_scene_data.is_empty()
            && self.probe_rt_lighting_rgb.is_empty()
            && self.trace_region_scene_data.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedProbe {
    pub probe_id: u32,
    pub slot: u32,
    pub ray_budget: u32,
    pub irradiance_rgb: [u8; 3],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedUpdateRequest {
    pub probe_id: u32,
    pub ray_budget: u32,
    pub generation: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedProbeSceneData {
    pub probe_id: u32,
    pub position_x_q: u32,
    pub position_y_q: u32,
    pub position_z_q: u32,
    pub radius_q: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedProbeRtLighting {
    pub probe_id: u32,
    pub rt_lighting_rgb: [u8; 3],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedTraceRegionSceneData {
    pub region_id: u32,
    pub center_x_q: u32,
    pub center_y_q: u32,
    pub center_z_q: u32,
    pub radius_q: u32,
    pub coverage_q: u32,
    pub rt_lighting_rgb: [u8; 3],
}

/// Neutral sideband data prepared by runtime providers before the renderer runs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderPreparedRuntimeSidebands {
    pub plugin_renderer_outputs: RenderPluginRendererOutputs,
    pub hybrid_gi_prepared_frame: Option<RenderHybridGiPreparedFrame>,
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
            hybrid_gi_prepared_frame: None,
            hybrid_gi_evictable_probe_ids,
            virtual_geometry_evictable_page_ids,
        }
    }

    pub fn with_hybrid_gi_prepared_frame(
        mut self,
        prepared_frame: Option<RenderHybridGiPreparedFrame>,
    ) -> Self {
        self.hybrid_gi_prepared_frame = prepared_frame;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.plugin_renderer_outputs.is_empty()
            && self
                .hybrid_gi_prepared_frame
                .as_ref()
                .map(RenderHybridGiPreparedFrame::is_empty)
                .unwrap_or(true)
            && self.hybrid_gi_evictable_probe_ids.is_empty()
            && self.virtual_geometry_evictable_page_ids.is_empty()
    }

    pub fn hybrid_gi_prepared_frame(&self) -> Option<&RenderHybridGiPreparedFrame> {
        self.hybrid_gi_prepared_frame.as_ref()
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

    pub(crate) fn take_hybrid_gi_readback_outputs(&mut self) -> RenderHybridGiReadbackOutputs {
        std::mem::take(&mut self.plugin_renderer_outputs.hybrid_gi)
    }

    pub(crate) fn take_virtual_geometry_readback_outputs(
        &mut self,
    ) -> RenderVirtualGeometryReadbackOutputs {
        std::mem::take(&mut self.plugin_renderer_outputs.virtual_geometry)
    }

    pub(crate) fn take_particle_readback_outputs(&mut self) -> RenderParticleGpuReadbackOutputs {
        std::mem::take(&mut self.plugin_renderer_outputs.particles)
    }

    pub fn hybrid_gi_evictable_probe_ids(&self) -> &[u32] {
        &self.hybrid_gi_evictable_probe_ids
    }

    pub fn virtual_geometry_evictable_page_ids(&self) -> &[u32] {
        &self.virtual_geometry_evictable_page_ids
    }

    pub(crate) fn take_hybrid_gi_evictable_probe_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.hybrid_gi_evictable_probe_ids)
    }

    pub(crate) fn take_virtual_geometry_evictable_page_ids(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.virtual_geometry_evictable_page_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_runtime_sidebands_report_empty_only_without_payloads() {
        assert!(RenderPreparedRuntimeSidebands::default().is_empty());
        assert!(RenderHybridGiPreparedFrame::default().is_empty());
        assert!(!RenderHybridGiPreparedFrame {
            probe_rt_lighting_rgb: vec![RenderHybridGiPreparedProbeRtLighting {
                probe_id: 3,
                rt_lighting_rgb: [8, 16, 24],
            }],
            ..RenderHybridGiPreparedFrame::default()
        }
        .is_empty());

        let sidebands = RenderPreparedRuntimeSidebands::new(
            RenderPluginRendererOutputs::default(),
            vec![5],
            Vec::new(),
        );

        assert!(!sidebands.is_empty());
        assert_eq!(sidebands.hybrid_gi_evictable_probe_ids(), &[5]);
    }
}
