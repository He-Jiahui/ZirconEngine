use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct HybridGiCompletionParams {
    pub(super) resident_probe_count: u32,
    pub(super) pending_probe_count: u32,
    pub(super) probe_budget: u32,
    pub(super) trace_region_count: u32,
    pub(super) tracing_budget: u32,
    pub(super) evictable_probe_count: u32,
    pub(super) scene_light_seed_rgb: u32,
    pub(super) scene_light_strength_q: u32,
}
