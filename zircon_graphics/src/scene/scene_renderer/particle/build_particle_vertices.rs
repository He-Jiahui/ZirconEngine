use zircon_math::Vec4;

use crate::types::EditorOrRuntimeFrame;

use super::particle_vertex::ParticleVertex;

pub(in crate::scene::scene_renderer::particle) fn build_particle_vertices(
    frame: &EditorOrRuntimeFrame,
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
        let top_left = sprite.position - right * half + up * half;
        let top_right = sprite.position + right * half + up * half;
        let bottom_left = sprite.position - right * half - up * half;
        let bottom_right = sprite.position + right * half - up * half;
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
