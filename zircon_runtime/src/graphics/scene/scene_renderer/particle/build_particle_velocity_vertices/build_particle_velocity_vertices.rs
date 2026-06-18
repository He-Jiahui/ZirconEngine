use std::collections::{BTreeMap, VecDeque};

use crate::core::framework::render::{
    RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot,
};
use crate::core::math::{Vec2, Vec3};
use crate::graphics::types::ViewportRenderFrame;

use super::super::particle_velocity_vertex::ParticleVelocityVertex;

pub(in crate::graphics::scene::scene_renderer::particle) fn build_particle_velocity_vertices(
    frame: &ViewportRenderFrame,
) -> Vec<ParticleVelocityVertex> {
    if frame.extract.particles.sprites.is_empty()
        || frame.extract.particles.previous_sprites.is_empty()
    {
        return Vec::new();
    }

    let camera = frame.effective_camera().transform;
    let right = camera.right();
    let up = camera.up();
    let ambiguous_anonymous_entities = frame
        .extract
        .particles
        .anonymous_stream_ambiguity_entities();
    let mut previous_by_entity =
        BTreeMap::<_, VecDeque<RenderParticlePreviousSpriteSnapshot>>::new();
    for previous in &frame.extract.particles.previous_sprites {
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

    let mut vertices = Vec::new();
    for sprite in &frame.extract.particles.sprites {
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
mod tests {
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderOverlayExtract,
        RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
    use crate::graphics::types::ViewportRenderFrame;

    use super::build_particle_velocity_vertices;

    #[test]
    fn particle_velocity_vertices_require_previous_sprite_state() {
        let frame = particle_frame(vec![particle_sprite(7, Vec3::ZERO)], Vec::new());

        assert!(build_particle_velocity_vertices(&frame).is_empty());
    }

    #[test]
    fn particle_velocity_vertices_pair_current_and_previous_quads() {
        let frame = particle_frame(
            vec![particle_sprite(7, Vec3::new(0.25, 0.0, -2.5))],
            vec![RenderParticlePreviousSpriteSnapshot {
                entity: 7,
                stable_sprite_key: 0,
                position: Vec3::new(-0.25, 0.0, -2.5),
                size: 1.0,
                aspect_ratio: 1.0,
                billboard_offset: Vec2::ZERO,
                rotation: 0.0,
                billboard_basis: None,
            }],
        );

        let vertices = build_particle_velocity_vertices(&frame);

        assert_eq!(vertices.len(), 6);
        assert_ne!(vertices[0].current_position, vertices[0].previous_position);
    }

    #[test]
    fn particle_velocity_vertices_consume_duplicate_keyed_previous_rows_once() {
        let frame = particle_frame(
            vec![
                particle_sprite_with_key(7, 31, Vec3::new(0.25, 0.0, -2.5)),
                particle_sprite_with_key(7, 31, Vec3::new(1.25, 0.0, -2.5)),
            ],
            vec![RenderParticlePreviousSpriteSnapshot {
                entity: 7,
                stable_sprite_key: 31,
                position: Vec3::new(-0.25, 0.0, -2.5),
                size: 1.0,
                aspect_ratio: 1.0,
                billboard_offset: Vec2::ZERO,
                rotation: 0.0,
                billboard_basis: None,
            }],
        );

        assert_eq!(build_particle_velocity_vertices(&frame).len(), 6);
    }

    #[test]
    fn particle_velocity_vertices_reject_ambiguous_anonymous_previous_rows() {
        let frame = particle_frame(
            vec![
                particle_sprite(7, Vec3::new(0.25, 0.0, -2.5)),
                particle_sprite(7, Vec3::new(1.25, 0.0, -2.5)),
            ],
            vec![
                RenderParticlePreviousSpriteSnapshot {
                    entity: 7,
                    stable_sprite_key: 0,
                    position: Vec3::new(-0.25, 0.0, -2.5),
                    size: 1.0,
                    aspect_ratio: 1.0,
                    billboard_offset: Vec2::ZERO,
                    rotation: 0.0,
                    billboard_basis: None,
                },
                RenderParticlePreviousSpriteSnapshot {
                    entity: 7,
                    stable_sprite_key: 0,
                    position: Vec3::new(0.75, 0.0, -2.5),
                    size: 1.0,
                    aspect_ratio: 1.0,
                    billboard_offset: Vec2::ZERO,
                    rotation: 0.0,
                    billboard_basis: None,
                },
            ],
        );

        assert!(build_particle_velocity_vertices(&frame).is_empty());
    }

    #[test]
    fn particle_velocity_vertices_match_duplicate_entity_by_stable_sprite_key() {
        let frame = particle_frame(
            vec![
                particle_sprite_with_key(7, 11, Vec3::new(0.25, 0.0, -2.5)),
                particle_sprite_with_key(7, 12, Vec3::new(1.25, 0.0, -2.5)),
            ],
            vec![RenderParticlePreviousSpriteSnapshot {
                entity: 7,
                stable_sprite_key: 12,
                position: Vec3::new(0.75, 0.0, -2.5),
                size: 1.0,
                aspect_ratio: 1.0,
                billboard_offset: Vec2::ZERO,
                rotation: 0.0,
                billboard_basis: None,
            }],
        );

        let vertices = build_particle_velocity_vertices(&frame);

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].current_position[0], 0.75);
        assert_eq!(vertices[0].previous_position[0], 0.25);
    }

    #[test]
    fn particle_velocity_vertices_use_previous_billboard_basis_when_available() {
        let frame = particle_frame(
            vec![particle_sprite_with_key(7, 12, Vec3::new(0.25, 0.0, -2.5))],
            vec![RenderParticlePreviousSpriteSnapshot {
                entity: 7,
                stable_sprite_key: 12,
                position: Vec3::new(0.25, 0.0, -2.5),
                size: 1.0,
                aspect_ratio: 1.0,
                billboard_offset: Vec2::ZERO,
                rotation: 0.0,
                billboard_basis: Some(
                    crate::core::framework::render::RenderParticleBillboardBasisSnapshot::new(
                        Vec3::Y,
                        Vec3::X,
                    ),
                ),
            }],
        );

        let vertices = build_particle_velocity_vertices(&frame);

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].previous_position[0], 0.75);
        assert_eq!(vertices[0].previous_position[1], -0.5);
    }

    #[test]
    fn particle_velocity_vertices_skip_overlay_sprites() {
        let mut overlay = particle_sprite_with_key(7, 12, Vec3::new(0.25, 0.0, -2.5));
        overlay.depth_test = false;
        let frame = particle_frame(
            vec![overlay],
            vec![RenderParticlePreviousSpriteSnapshot {
                entity: 7,
                stable_sprite_key: 12,
                position: Vec3::new(-0.25, 0.0, -2.5),
                size: 1.0,
                aspect_ratio: 1.0,
                billboard_offset: Vec2::ZERO,
                rotation: 0.0,
                billboard_basis: None,
            }],
        );

        assert!(build_particle_velocity_vertices(&frame).is_empty());
    }

    fn particle_frame(
        sprites: Vec<RenderParticleSpriteSnapshot>,
        previous_sprites: Vec<RenderParticlePreviousSpriteSnapshot>,
    ) -> ViewportRenderFrame {
        let viewport_size = UVec2::new(64, 64);
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera: ViewportCameraSnapshot {
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
                        ..ViewportCameraSnapshot::default()
                    },
                    meshes: Vec::new(),
                    directional_lights: Vec::new(),
                    point_lights: Vec::new(),
                    spot_lights: Vec::new(),
                    ambient_lights: Vec::new(),
                    rect_lights: Vec::new(),
                },
                overlays: RenderOverlayExtract::default(),
                preview: PreviewEnvironmentExtract {
                    lighting_enabled: false,
                    skybox_enabled: false,
                    fallback_skybox: FallbackSkyboxKind::None,
                    clear_color: Vec4::ZERO,
                },
                virtual_geometry_debug: None,
            },
        );
        extract.apply_viewport_size(viewport_size);
        extract.particles.sprites = sprites;
        extract.particles.previous_sprites = previous_sprites;
        ViewportRenderFrame::from_extract(extract, viewport_size)
    }

    fn particle_sprite(entity: u64, position: Vec3) -> RenderParticleSpriteSnapshot {
        particle_sprite_with_key(entity, 0, position)
    }

    fn particle_sprite_with_key(
        entity: u64,
        stable_sprite_key: u64,
        position: Vec3,
    ) -> RenderParticleSpriteSnapshot {
        RenderParticleSpriteSnapshot {
            entity,
            stable_sprite_key,
            position,
            size: 1.0,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::ZERO,
            rotation: 0.0,
            sort_order: 0,
            color: Vec4::ONE,
            intensity: 1.0,
            depth_test: true,
            material: None,
            texture: None,
        }
    }
}
