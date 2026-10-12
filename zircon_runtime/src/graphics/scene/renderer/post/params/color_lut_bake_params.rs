use bytemuck::{Pod, Zeroable};

/// 把本帧曝光、色调映射及用户 LUT 选择传给颜色 LUT 烘焙阶段。
/// 绑定模式编号与 `color_lut_bake.wgsl` 共同定义，上传布局和采样资源必须匹配。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct ColorLutBakeParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) lut_size_and_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) tonemap_lut: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) grading: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) tint_and_exposure: [f32; 4],
}
