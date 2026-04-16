use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct PostProcessParams {
    pub(super) viewport_and_clusters: [u32; 4],
    pub(super) feature_flags: [u32; 4],
    pub(super) hybrid_gi_counts: [u32; 4],
    pub(super) blends: [f32; 4],
    pub(super) grading: [f32; 4],
    pub(super) tint_and_probe: [f32; 4],
    pub(super) hybrid_gi_color_and_intensity: [f32; 4],
    pub(super) baked_color_and_intensity: [f32; 4],
}
