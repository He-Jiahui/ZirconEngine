use crate::core::framework::render::RenderParticleSpriteSnapshot;
use crate::core::math::Vec4;

use crate::graphics::types::ViewportRenderFrame;

use super::super::particle_vertex::ParticleVertex;

const PARTICLE_VERTICES_PER_SPRITE: usize = 6;

/// 以当前有效相机基向量展开精灵，按相机层和 depth_test 筛选，保留抽取列表的相对顺序。
/// 场景世界抽取通常先按距离排序；此处只保留传入列表的相对顺序并生成颜色四边形，不重新排序或驱动模拟。
// TODO: [CR-SCENE-PARTICLE-0001] 快照带 material/texture，但当前顶点与管线仅有位置/颜色；需确认 CPU 粒子是否承诺支持这些资源。
pub(in crate::graphics::scene::scene_renderer::particle) fn build_particle_vertices(
    frame: &ViewportRenderFrame,
    depth_test: bool,
) -> Vec<ParticleVertex> {
    let camera = frame.effective_camera().transform;
    let right = camera.right();
    let up = camera.up();
    let camera_layers = frame.extract.view.selected_camera_layers();
    // TODO: [CR-SCENE-PARTICLE-0002] 可见性筛选未验证位置、旋转、尺寸等有限性，后续顶点构造直接拷贝 f32；需明确快照准入方并阻止非有限几何进入 GPU。
    let is_renderable = |sprite: &RenderParticleSpriteSnapshot| {
        if !camera_layers.intersects(&sprite.render_layer_mask) {
            return false;
        }
        if sprite.depth_test != depth_test {
            return false;
        }
        if sprite.size <= f32::EPSILON || sprite.color.w <= f32::EPSILON {
            return false;
        }
        true
    };
    let vertex_capacity = frame
        .extract
        .particles
        .sprites
        .iter()
        .filter(|sprite| is_renderable(sprite))
        .count()
        .saturating_mul(PARTICLE_VERTICES_PER_SPRITE);
    let mut vertices = Vec::with_capacity(vertex_capacity);

    for sprite in &frame.extract.particles.sprites {
        if !is_renderable(sprite) {
            continue;
        }
        let aspect_ratio = sprite.aspect_ratio.max(f32::EPSILON);
        let half_width = sprite.size * aspect_ratio * 0.5;
        let half_height = sprite.size * 0.5;
        let color = Vec4::new(
            sprite.color.x * sprite.intensity,
            sprite.color.y * sprite.intensity,
            sprite.color.z * sprite.intensity,
            sprite.color.w.clamp(0.0, 1.0),
        );
        let sin = sprite.rotation.sin();
        let cos = sprite.rotation.cos();
        let rotated = |x: f32, y: f32| right * (x * cos - y * sin) + up * (x * sin + y * cos);
        let center =
            sprite.position + rotated(sprite.billboard_offset.x, sprite.billboard_offset.y);
        let top_left = center + rotated(-half_width, half_height);
        let top_right = center + rotated(half_width, half_height);
        let bottom_left = center + rotated(-half_width, -half_height);
        let bottom_right = center + rotated(half_width, -half_height);
        vertices.extend_from_slice(&[
            ParticleVertex::new(top_left, color),
            ParticleVertex::new(bottom_left, color),
            ParticleVertex::new(top_right, color),
            ParticleVertex::new(top_right, color),
            ParticleVertex::new(bottom_left, color),
            ParticleVertex::new(bottom_right, color),
        ]);
    }

    vertices
}

#[cfg(test)]
#[path = "tests/build_particle_vertices.rs"]
mod tests;
