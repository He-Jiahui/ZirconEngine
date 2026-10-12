//! CPU 抽取粒子与世界 HUD 的 billboard 场景绘制层；透明颜色与历史速度分别接入 render graph。
//! GPU 粒子绘制由插件回调路径提供，本层消费已有精灵快照及相机/历史状态。
#[path = "../../build_particle_velocity_vertices/mod.rs"]
mod build_particle_velocity_vertices;
#[path = "../../particle/build_particle_vertices/mod.rs"]
mod build_particle_vertices;
#[path = "particle_renderer/mod.rs"]
mod particle_renderer;
#[path = "../../particle/particle_velocity_vertex/mod.rs"]
mod particle_velocity_vertex;
#[path = "particle_vertex/mod.rs"]
mod particle_vertex;

pub(crate) use particle_renderer::ParticleRenderer;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
