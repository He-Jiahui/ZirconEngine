use bytemuck::{Pod, Zeroable};

/// 屏幕空间后处理保留的反射贡献布局；与环境光照的世界空间反射探针布局独立。
/// 当前编码入口返回零个有效元素，调用者须以计数控制读取，不能据缓冲容量推断有效探针数。
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct GpuReflectionProbe {
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_uv_and_radius: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) color_and_intensity: [f32; 4],
}
