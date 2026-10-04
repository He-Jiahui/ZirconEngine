// 构造时编译颜色/速度两套 WGSL；两者分别匹配 ParticleVertex 与 ParticleVelocityVertex 属性布局。
pub(in crate::graphics::scene::scene_renderer::particle) const PARTICLE_SHADER: &str =
    include_str!("../shaders/particle.wgsl");
pub(in crate::graphics::scene::scene_renderer::particle) const PARTICLE_VELOCITY_SHADER: &str =
    include_str!("../shaders/particle_velocity.wgsl");
