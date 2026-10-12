use bytemuck::{Pod, Zeroable};

/// 单次 HZB mip 构建的目标范围；首层从场景深度读取，后续层读取上一层范围。
/// 执行入口为每层选择对应参数片段，不能让同一提交内的多层共用最后一次 CPU 写入。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct HzbParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) target_size: [u32; 2],
    pub(in crate::graphics::scene::scene_renderer::post_process) target_mip_level: u32,
    pub(in crate::graphics::scene::scene_renderer::post_process) _pad0: u32,
}
