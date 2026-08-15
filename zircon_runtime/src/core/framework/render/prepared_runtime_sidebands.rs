use super::{
    RenderHybridGiReadbackOutputs, RenderHybridGiResolvedSettings,
    RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
    RenderVirtualGeometryReadbackOutputs,
};

pub const HYBRID_GI_SOURCE_FULL_DYNAMIC: u32 = 1 << 0;
pub const HYBRID_GI_SOURCE_BAKED_BASELINE: u32 = 1 << 1;
pub const HYBRID_GI_SOURCE_DYNAMIC_DELTA: u32 = 1 << 2;
pub const RENDER_HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderHybridGiCompositePolicy {
    source_mask: u32,
    baked_baseline_weight_q8: u8,
    dynamic_weight_q8: u8,
    baked_light_set_generation: Option<u64>,
    participation_epoch: u64,
}

impl RenderHybridGiCompositePolicy {
    pub const fn full_dynamic(participation_epoch: u64) -> Self {
        Self {
            source_mask: HYBRID_GI_SOURCE_FULL_DYNAMIC,
            baked_baseline_weight_q8: 0,
            dynamic_weight_q8: u8::MAX,
            baked_light_set_generation: None,
            participation_epoch,
        }
    }

    pub const fn baked_baseline_with_dynamic_delta(
        baked_light_set_generation: u64,
        participation_epoch: u64,
    ) -> Self {
        Self {
            source_mask: HYBRID_GI_SOURCE_BAKED_BASELINE | HYBRID_GI_SOURCE_DYNAMIC_DELTA,
            baked_baseline_weight_q8: u8::MAX,
            dynamic_weight_q8: u8::MAX,
            baked_light_set_generation: Some(baked_light_set_generation),
            participation_epoch,
        }
    }

    pub const fn source_mask(self) -> u32 {
        self.source_mask
    }

    pub const fn baked_baseline_weight_q8(self) -> u8 {
        self.baked_baseline_weight_q8
    }

    pub const fn dynamic_weight_q8(self) -> u8 {
        self.dynamic_weight_q8
    }

    pub const fn baked_light_set_generation(self) -> Option<u64> {
        self.baked_light_set_generation
    }

    pub const fn participation_epoch(self) -> u64 {
        self.participation_epoch
    }

    pub const fn accepts_hybrid_gi_output(self) -> bool {
        let has_full_dynamic = self.source_mask & HYBRID_GI_SOURCE_FULL_DYNAMIC != 0;
        let has_baked_baseline = self.source_mask & HYBRID_GI_SOURCE_BAKED_BASELINE != 0;
        let has_dynamic_delta = self.source_mask & HYBRID_GI_SOURCE_DYNAMIC_DELTA != 0;

        (has_full_dynamic && !has_baked_baseline && !has_dynamic_delta)
            || (!has_full_dynamic && has_baked_baseline && has_dynamic_delta)
    }
}

impl Default for RenderHybridGiCompositePolicy {
    fn default() -> Self {
        Self::full_dynamic(0)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderHybridGiPreparedFrame {
    pub composite_policy: RenderHybridGiCompositePolicy,
    pub resolved_settings: Option<RenderHybridGiResolvedSettings>,
    pub radiance_cache_instance_id: u64,
    /// Renderer-neutral scene descriptors produced by the Hybrid GI runtime provider.
    pub scene_prepare: Option<RenderHybridGiPreparedSceneFrame>,
    pub radiance_cache_bootstrap_updates: Vec<RenderHybridGiPreparedRadianceCacheUpdate>,
    pub radiance_cache_updates: Vec<RenderHybridGiPreparedRadianceCacheUpdate>,
    pub radiance_cache_consumes: Vec<RenderHybridGiPreparedRadianceCacheConsume>,
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
        self.composite_policy == RenderHybridGiCompositePolicy::default()
            && self.resolved_settings.is_none()
            && self
                .scene_prepare
                .as_ref()
                .map(RenderHybridGiPreparedSceneFrame::is_empty)
                .unwrap_or(true)
            && self.radiance_cache_bootstrap_updates.is_empty()
            && self.radiance_cache_updates.is_empty()
            && self.radiance_cache_consumes.is_empty()
            && self.resident_probes.is_empty()
            && self.pending_updates.is_empty()
            && self.scheduled_trace_region_ids.is_empty()
            && self.evictable_probe_ids.is_empty()
            && self.probe_scene_data.is_empty()
            && self.probe_rt_lighting_rgb.is_empty()
            && self.trace_region_scene_data.is_empty()
    }
}

/// Scene-preparation data that crosses the provider-to-renderer boundary without exposing
/// plugin-private HGI types to the core renderer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderHybridGiPreparedSceneFrame {
    pub card_capture_requests: Vec<RenderHybridGiPreparedCardCaptureRequest>,
    pub surface_cache_page_contents: Vec<RenderHybridGiPreparedSurfaceCachePageContent>,
    pub voxel_clipmaps: Vec<RenderHybridGiPreparedVoxelClipmap>,
    pub voxel_cells: Vec<RenderHybridGiPreparedVoxelCell>,
    /// Resolves presentation-local card IDs back to prepared geometry instances.
    pub card_owners: Vec<RenderHybridGiPreparedCardOwner>,
}

impl RenderHybridGiPreparedSceneFrame {
    pub fn is_empty(&self) -> bool {
        self.card_capture_requests.is_empty()
            && self.surface_cache_page_contents.is_empty()
            && self.voxel_clipmaps.is_empty()
            && self.voxel_cells.is_empty()
            && self.card_owners.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedCardOwner {
    pub card_id: u32,
    pub stable_instance_key: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderHybridGiPreparedCardCaptureRequest {
    pub card_id: u32,
    pub page_id: u32,
    pub atlas_slot_id: u32,
    pub capture_slot_id: u32,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderHybridGiPreparedSurfaceCachePageContent {
    pub page_id: u32,
    pub owner_card_id: u32,
    pub atlas_slot_id: u32,
    pub capture_slot_id: u32,
    pub bounds_center: [f32; 3],
    pub bounds_radius: f32,
    pub atlas_sample_rgba: [u8; 4],
    pub capture_sample_rgba: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderHybridGiPreparedVoxelClipmap {
    pub clipmap_id: u32,
    pub center: [f32; 3],
    pub half_extent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedVoxelCell {
    pub clipmap_id: u32,
    pub cell_index: u32,
    pub occupancy_count: u32,
    pub dominant_card_id: u32,
    pub radiance_present: bool,
    pub radiance_rgb: [u8; 3],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedRadianceCacheUpdate {
    pub slot: u32,
    pub generation: u64,
    pub radiance_rgb: [u8; 3],
    pub confidence_q8: u8,
    pub reuse_committed_radiance: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedRadianceCacheConsume {
    pub probe_id: u32,
    pub generation: u64,
    pub slots: [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
    pub weights_q16: [u16; RENDER_HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderHybridGiPreparedProbe {
    pub probe_id: u32,
    pub slot: u32,
    pub stable_instance_key: u64,
    pub source_mask: u32,
    pub dynamic_weight_q8: u8,
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
        assert!(!RenderHybridGiPreparedFrame {
            radiance_cache_instance_id: 9,
            radiance_cache_updates: vec![RenderHybridGiPreparedRadianceCacheUpdate {
                slot: 3,
                generation: 7,
                radiance_rgb: [8, 16, 24],
                confidence_q8: 200,
                reuse_committed_radiance: false,
            }],
            ..RenderHybridGiPreparedFrame::default()
        }
        .is_empty());
        assert!(!RenderHybridGiPreparedFrame {
            radiance_cache_instance_id: 9,
            radiance_cache_bootstrap_updates: vec![RenderHybridGiPreparedRadianceCacheUpdate {
                slot: 3,
                generation: 7,
                radiance_rgb: [8, 16, 24],
                confidence_q8: 200,
                reuse_committed_radiance: false,
            }],
            ..RenderHybridGiPreparedFrame::default()
        }
        .is_empty());
        assert!(!RenderHybridGiPreparedFrame {
            composite_policy: RenderHybridGiCompositePolicy::baked_baseline_with_dynamic_delta(
                7, 3
            ),
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

    #[test]
    fn hybrid_gi_composite_policy_rejects_full_dynamic_baked_double_ownership() {
        let full_dynamic = RenderHybridGiCompositePolicy::full_dynamic(4);
        assert!(full_dynamic.accepts_hybrid_gi_output());
        assert_eq!(full_dynamic.source_mask(), HYBRID_GI_SOURCE_FULL_DYNAMIC);

        let baked_delta = RenderHybridGiCompositePolicy::baked_baseline_with_dynamic_delta(9, 5);
        assert!(baked_delta.accepts_hybrid_gi_output());
        assert_eq!(baked_delta.baked_light_set_generation(), Some(9));
        assert_eq!(baked_delta.participation_epoch(), 5);
        assert_eq!(
            baked_delta.source_mask(),
            HYBRID_GI_SOURCE_BAKED_BASELINE | HYBRID_GI_SOURCE_DYNAMIC_DELTA
        );
        assert_eq!(
            baked_delta.source_mask() & HYBRID_GI_SOURCE_FULL_DYNAMIC,
            0,
            "a baked baseline must never share a lobe with full-dynamic indirect light"
        );
    }
}
