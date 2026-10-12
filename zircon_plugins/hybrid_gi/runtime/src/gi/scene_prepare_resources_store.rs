use super::HybridGiScenePrepareResourcesSnapshot;
use zircon_runtime::core::framework::render::RenderHybridGiProbeTraceDiagnosticRecord;

impl HybridGiScenePrepareResourcesSnapshot {
    pub(crate) fn store_texture_slot_rgba_samples(
        &mut self,
        atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
        capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    ) {
        self.atlas_slot_rgba_samples = atlas_slot_rgba_samples;
        self.capture_slot_rgba_samples = capture_slot_rgba_samples;
    }

    pub(crate) fn store_surface_cache_depth_samples(
        &mut self,
        surface_cache_depth_rgba_samples: Vec<(u32, [u8; 4])>,
    ) {
        self.surface_cache_depth_rgba_samples = surface_cache_depth_rgba_samples;
    }

    pub(crate) fn store_voxel_resource_samples(
        &mut self,
        voxel_clipmap_rgba_samples: Vec<(u32, [u8; 4])>,
        voxel_clipmap_occupancy_masks: Vec<(u32, u64)>,
        voxel_clipmap_cell_rgba_samples: Vec<(u32, u32, [u8; 4])>,
        voxel_clipmap_cell_occupancy_counts: Vec<(u32, u32, u32)>,
        voxel_clipmap_cell_dominant_node_ids: Vec<(u32, u32, u64)>,
        voxel_clipmap_cell_dominant_rgba_samples: Vec<(u32, u32, [u8; 4])>,
    ) {
        self.voxel_clipmap_rgba_samples = voxel_clipmap_rgba_samples;
        self.voxel_clipmap_occupancy_masks = voxel_clipmap_occupancy_masks;
        self.voxel_clipmap_cell_rgba_samples = voxel_clipmap_cell_rgba_samples;
        self.voxel_clipmap_cell_occupancy_counts = voxel_clipmap_cell_occupancy_counts;
        self.voxel_clipmap_cell_dominant_node_ids = voxel_clipmap_cell_dominant_node_ids;
        self.voxel_clipmap_cell_dominant_rgba_samples = voxel_clipmap_cell_dominant_rgba_samples;
    }

    pub(crate) fn store_probe_trace_tiles(
        &mut self,
        probe_trace_tiles: Vec<(u32, u32, u32, u32)>,
        probe_trace_dispatch: [u32; 3],
    ) {
        self.probe_trace_tiles = probe_trace_tiles;
        self.probe_trace_dispatch = probe_trace_dispatch;
    }

    pub(crate) fn store_probe_trace_diagnostics(
        &mut self,
        diagnostics: Vec<RenderHybridGiProbeTraceDiagnosticRecord>,
    ) {
        self.probe_trace_diagnostics = diagnostics;
    }
}
