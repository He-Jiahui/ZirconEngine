use bytemuck::{Pod, Zeroable};

/// Bloom 提取阶段的视口与亮度参数；输入原点允许从共享场景目标读取局部视口。
/// 由执行入口上传，字段布局须与 `bloom.wgsl` 的 uniform 同步。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct BloomParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) tuning: [f32; 4],
}
