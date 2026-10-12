use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
/// 同一精灵角点的当前/历史世界位置契约；生成阶段以稳定精灵身份匹配历史，shader 按属性 0–1 读取。
pub(in crate::graphics::scene::scene_renderer::particle) struct ParticleVelocityVertex {
    pub(in crate::graphics::scene::scene_renderer::particle) current_position: [f32; 3],
    pub(in crate::graphics::scene::scene_renderer::particle) previous_position: [f32; 3],
}
