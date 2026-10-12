use bytemuck::{Pod, Zeroable};

/// 后处理二维灯光聚合使用的方向光上传布局，与 `clustered_lighting.wgsl` 的元素对应。
/// 活跃数量由 `ClusterParams` 指定，未上传的尾部不得被着色器读取。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct ClusteredDirectionalLight {
    pub(in crate::graphics::scene::scene_renderer::post_process) direction: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) color_intensity: [f32; 4],
}
