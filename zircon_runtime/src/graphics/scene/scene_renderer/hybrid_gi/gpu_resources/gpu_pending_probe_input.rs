use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GpuPendingProbeInput {
    pub(super) probe_id: u32,
    pub(super) logical_index: u32,
    pub(super) ray_budget: u32,
    pub(super) lineage_trace_support_q: u32,
    pub(super) position_x_q: u32,
    pub(super) position_y_q: u32,
    pub(super) position_z_q: u32,
    pub(super) radius_q: u32,
    pub(super) runtime_hierarchy_irradiance_rgb: u32,
    pub(super) runtime_hierarchy_irradiance_weight_q: u32,
    pub(super) skip_scene_prepare_for_irradiance_q: u32,
    pub(super) lineage_trace_lighting_rgb: u32,
    pub(super) skip_scene_prepare_for_trace_q: u32,
    pub(super) parent_probe_id: u32,
    pub(super) resident_ancestor_probe_id: u32,
    pub(super) resident_ancestor_depth: u32,
    pub(super) resident_secondary_ancestor_probe_id: u32,
    pub(super) resident_secondary_ancestor_depth: u32,
    pub(super) resident_tertiary_ancestor_probe_id: u32,
    pub(super) resident_tertiary_ancestor_depth: u32,
    pub(super) resident_quaternary_ancestor_probe_id: u32,
    pub(super) resident_quaternary_ancestor_depth: u32,
}
