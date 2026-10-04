use std::collections::{BTreeMap, VecDeque};

use crate::core::framework::render::{
    RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot,
};
use crate::core::math::{Vec2, Vec3};
use crate::graphics::types::ViewportRenderFrame;

use super::super::particle_velocity_vertex::ParticleVelocityVertex;

const PARTICLE_VELOCITY_VERTICES_PER_SPRITE: usize = 6;

pub(in crate::graphics::scene::scene_renderer::particle) fn build_particle_velocity_vertices(
    frame: &ViewportRenderFrame,
) -> Vec<ParticleVelocityVertex> {
    if frame.extract.particles.sprites.is_empty() || frame.previous_particle_sprites().is_empty() {
        return Vec::new();
    }

    let camera = frame.effective_camera().transform;
    let right = camera.right();
    let up = camera.up();
    let camera_layers = frame.extract.view.selected_camera_layers();
    let ambiguous_anonymous_entities = frame
        .extract
        .particles
        .anonymous_stream_ambiguity_entities();
    let mut previous_by_entity =
        BTreeMap::<_, VecDeque<RenderParticlePreviousSpriteSnapshot>>::new();
    for previous in frame.previous_particle_sprites() {
        if previous.size <= f32::EPSILON {
            continue;
        }
        if previous.stable_sprite_key == 0
            && ambiguous_anonymous_entities.contains(&previous.entity)
        {
            continue;
        }
        previous_by_entity
            .entry(previous.identity())
            .or_default()
            .push_back(*previous);
    }

    let mut vertices = Vec::with_capacity(particle_velocity_vertex_capacity(
        frame.extract.particles.sprites.len(),
    ));
    for sprite in &frame.extract.particles.sprites {
        if !camera_layers.intersects(&sprite.render_layer_mask) {
            continue;
        }
        if !sprite.depth_test {
            continue;
        }
        if sprite.size <= f32::EPSILON || sprite.color.w <= f32::EPSILON {
            continue;
        }
        if sprite.stable_sprite_key == 0 && ambiguous_anonymous_entities.contains(&sprite.entity) {
            continue;
        }
        let Some(previous) = previous_by_entity
            .get_mut(&sprite.identity())
            .and_then(VecDeque::pop_front)
        else {
            continue;
        };

        let current_quad = current_particle_quad(sprite, right, up);
        let previous_quad = previous_particle_quad(previous, right, up);
        vertices.extend_from_slice(&[
            ParticleVelocityVertex::new(current_quad.top_left, previous_quad.top_left),
            ParticleVelocityVertex::new(current_quad.bottom_left, previous_quad.bottom_left),
            ParticleVelocityVertex::new(current_quad.top_right, previous_quad.top_right),
            ParticleVelocityVertex::new(current_quad.top_right, previous_quad.top_right),
            ParticleVelocityVertex::new(current_quad.bottom_left, previous_quad.bottom_left),
            ParticleVelocityVertex::new(current_quad.bottom_right, previous_quad.bottom_right),
        ]);
    }

    vertices
}

fn particle_velocity_vertex_capacity(sprite_count: usize) -> usize {
    sprite_count.saturating_mul(PARTICLE_VELOCITY_VERTICES_PER_SPRITE)
}

#[derive(Clone, Copy)]
struct ParticleQuad {
    top_left: Vec3,
    top_right: Vec3,
    bottom_left: Vec3,
    bottom_right: Vec3,
}

fn current_particle_quad(
    sprite: &RenderParticleSpriteSnapshot,
    right: Vec3,
    up: Vec3,
) -> ParticleQuad {
    particle_quad(
        sprite.position,
        sprite.size,
        sprite.aspect_ratio,
        sprite.billboard_offset,
        sprite.rotation,
        right,
        up,
    )
}

fn previous_particle_quad(
    sprite: RenderParticlePreviousSpriteSnapshot,
    right: Vec3,
    up: Vec3,
) -> ParticleQuad {
    let (right, up) = sprite
        .billboard_basis
        .map(|basis| (basis.right, basis.up))
        .unwrap_or((right, up));
    particle_quad(
        sprite.position,
        sprite.size,
        sprite.aspect_ratio,
        sprite.billboard_offset,
        sprite.rotation,
        right,
        up,
    )
}

fn particle_quad(
    position: Vec3,
    size: f32,
    aspect_ratio: f32,
    billboard_offset: Vec2,
    rotation: f32,
    right: Vec3,
    up: Vec3,
) -> ParticleQuad {
    let aspect_ratio = aspect_ratio.max(f32::EPSILON);
    let half_width = size * aspect_ratio * 0.5;
    let half_height = size * 0.5;
    let sin = rotation.sin();
    let cos = rotation.cos();
    let rotated = |x: f32, y: f32| right * (x * cos - y * sin) + up * (x * sin + y * cos);
    let center = position + rotated(billboard_offset.x, billboard_offset.y);
    ParticleQuad {
        top_left: center + rotated(-half_width, half_height),
        top_right: center + rotated(half_width, half_height),
        bottom_left: center + rotated(-half_width, -half_height),
        bottom_right: center + rotated(half_width, -half_height),
    }
}

#[cfg(test)]
#[path = "tests/build_particle_velocity_vertices.rs"]
mod tests;

#[cfg(test)]
#[path = "build_particle_velocity_vertices/tests/capacity_tests.rs"]
mod capacity_tests;
