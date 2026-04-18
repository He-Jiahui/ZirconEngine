use crate::SceneRenderer;

use super::super::gpu_completion::{HybridGiGpuCompletion, VirtualGeometryGpuCompletion};

pub(super) struct GpuCompletions {
    pub(super) hybrid_gi_completion: Option<HybridGiGpuCompletion>,
    pub(super) virtual_geometry_completion: Option<VirtualGeometryGpuCompletion>,
}

pub(super) fn collect_gpu_completions(renderer: &mut SceneRenderer) -> GpuCompletions {
    GpuCompletions {
        hybrid_gi_completion: collect_hybrid_gi_completion(renderer),
        virtual_geometry_completion: collect_virtual_geometry_completion(renderer),
    }
}

fn collect_hybrid_gi_completion(renderer: &mut SceneRenderer) -> Option<HybridGiGpuCompletion> {
    renderer
        .take_last_hybrid_gi_gpu_readback()
        .map(|readback| HybridGiGpuCompletion {
            cache_entries: readback.cache_entries,
            completed_probe_ids: readback.completed_probe_ids,
            completed_trace_region_ids: readback.completed_trace_region_ids,
            probe_irradiance_rgb: readback.probe_irradiance_rgb,
            probe_trace_lighting_rgb: readback.probe_trace_lighting_rgb,
        })
}

fn collect_virtual_geometry_completion(
    renderer: &mut SceneRenderer,
) -> Option<VirtualGeometryGpuCompletion> {
    renderer
        .take_last_virtual_geometry_gpu_readback()
        .map(|readback| VirtualGeometryGpuCompletion {
            page_table_entries: readback.page_table_entries,
            completed_page_assignments: readback.completed_page_assignments,
            completed_page_replacements: readback.completed_page_replacements,
        })
}
