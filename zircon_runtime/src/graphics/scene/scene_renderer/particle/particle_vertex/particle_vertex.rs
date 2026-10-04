use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
/// CPU 粒子颜色管线的位置 0 / 颜色 1 契约；逐精灵三角形保留抽取层次与透明绘制顺序。
pub(in crate::graphics::scene::scene_renderer::particle) struct ParticleVertex {
    pub(in crate::graphics::scene::scene_renderer::particle) position: [f32; 3],
    pub(in crate::graphics::scene::scene_renderer::particle) color: [f32; 4],
}
