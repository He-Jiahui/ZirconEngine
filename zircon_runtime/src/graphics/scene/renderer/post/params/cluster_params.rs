use bytemuck::{Pod, Zeroable};

/// 二维灯光聚合阶段的逻辑视口、网格和有效方向光数量。
/// 与 `clustered_lighting.wgsl` 配套；该阶段输出每格聚合值，而非三维光源列表。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct ClusterParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport_and_clusters: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) counts: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) strengths: [f32; 4],
}
