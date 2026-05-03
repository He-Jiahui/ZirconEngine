use crate::core::math::Vec4;

use crate::graphics::types::ViewportRenderFrame;

use super::super::particle_vertex::ParticleVertex;

pub(in crate::graphics::scene::scene_renderer::particle) fn build_particle_vertices(
    frame: &ViewportRenderFrame,
) -> Vec<ParticleVertex> {
    let camera = &frame.extract.view.camera.transform;
    let right = camera.right();
    let up = camera.up();
    let mut vertices = Vec::new();

    for sprite in &frame.extract.particles.sprites {
        if sprite.size <= f32::EPSILON || sprite.color.w <= f32::EPSILON {
            continue;
        }
        let half = sprite.size * 0.5;
        let color = Vec4::new(
            sprite.color.x * sprite.intensity,
            sprite.color.y * sprite.intensity,
            sprite.color.z * sprite.intensity,
            sprite.color.w.clamp(0.0, 1.0),
        );
        let sin = sprite.rotation.sin();
        let cos = sprite.rotation.cos();
        let rotated = |x: f32, y: f32| right * (x * cos - y * sin) + up * (x * sin + y * cos);
        let top_left = sprite.position + rotated(-half, half);
        let top_right = sprite.position + rotated(half, half);
        let bottom_left = sprite.position + rotated(-half, -half);
        let bottom_right = sprite.position + rotated(half, -half);
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
