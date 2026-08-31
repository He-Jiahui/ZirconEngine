use crate::core::framework::render::RenderParticleSpriteSnapshot;
use crate::core::math::Vec4;

use crate::graphics::types::ViewportRenderFrame;

use super::super::particle_vertex::ParticleVertex;

const PARTICLE_VERTICES_PER_SPRITE: usize = 6;

pub(in crate::graphics::scene::scene_renderer::particle) fn build_particle_vertices(
    frame: &ViewportRenderFrame,
    depth_test: bool,
) -> Vec<ParticleVertex> {
    let camera = frame.effective_camera().transform;
    let right = camera.right();
    let up = camera.up();
    let camera_layers = frame.extract.view.selected_camera_layers();
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
mod tests {
    use crate::core::framework::render::{
        CameraRenderDescriptor, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract,
        RenderLayerSet, RenderOverlayExtract, RenderParticleSpriteSnapshot,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
    use crate::graphics::types::ViewportRenderFrame;

    use super::build_particle_vertices;

    #[test]
    fn runtime99d_batch_exact_particle_vertex_capacity_preserves_depth_and_overlay() {
        let frame = particle_frame(vec![
            particle_sprite(7, Vec3::new(-0.25, 0.0, -2.5), true),
            particle_sprite(8, Vec3::new(0.25, 0.0, -2.5), false),
        ]);

        let depth_tested = build_particle_vertices(&frame, true);
        let overlay = build_particle_vertices(&frame, false);

        assert_eq!(depth_tested.len(), 6);
        assert_eq!(overlay.len(), 6);
        assert_eq!(depth_tested.capacity(), depth_tested.len());
        assert_eq!(overlay.capacity(), overlay.len());
        assert!(depth_tested.iter().all(|vertex| vertex.position[0] < 0.5));
        assert!(overlay.iter().all(|vertex| vertex.position[0] > -0.5));
    }

    #[test]
    fn runtime99d_batch_exact_particle_vertex_capacity_preserves_layer_filter() {
        let mut hidden = particle_sprite(7, Vec3::new(-0.25, 0.0, -2.5), true);
        hidden.render_layer_mask = RenderLayerSet::layer(1);
        let mut visible = particle_sprite(8, Vec3::new(0.25, 0.0, -2.5), true);
        visible.render_layer_mask = RenderLayerSet::layer(2);

        let frame =
            particle_frame_with_camera_layers(vec![hidden, visible], RenderLayerSet::layer(2));

        let vertices = build_particle_vertices(&frame, true);

        assert_eq!(vertices.len(), 6);
        assert!(vertices.iter().all(|vertex| vertex.position[0] > -0.5));
    }

    fn particle_frame(sprites: Vec<RenderParticleSpriteSnapshot>) -> ViewportRenderFrame {
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
                environment: crate::core::framework::render::EnvironmentExtract::default(),
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
        ViewportRenderFrame::from_extract(extract, viewport_size)
    }

    fn particle_frame_with_camera_layers(
        sprites: Vec<RenderParticleSpriteSnapshot>,
        camera_layers: RenderLayerSet,
    ) -> ViewportRenderFrame {
        let viewport_size = UVec2::new(64, 64);
        let mut camera = ViewportCameraSnapshot::default();
        camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 4.0));
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(Some(7), camera.clone());
        descriptor.culling_mask = camera_layers;
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            RenderSceneSnapshot {
                scene: RenderSceneGeometryExtract {
                    camera,
                    meshes: Vec::new(),
                    directional_lights: Vec::new(),
                    point_lights: Vec::new(),
                    spot_lights: Vec::new(),
                    ambient_lights: Vec::new(),
                    rect_lights: Vec::new(),
                },
                overlays: RenderOverlayExtract::default(),
                environment: crate::core::framework::render::EnvironmentExtract::default(),
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
        extract.select_camera_descriptor(descriptor);
        ViewportRenderFrame::from_extract(extract, viewport_size)
    }

    fn particle_sprite(
        entity: u64,
        position: Vec3,
        depth_test: bool,
    ) -> RenderParticleSpriteSnapshot {
        RenderParticleSpriteSnapshot {
            entity,
            stable_sprite_key: 1,
            position,
            size: 0.25,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::ZERO,
            rotation: 0.0,
            sort_order: 0,
            color: Vec4::ONE,
            intensity: 1.0,
            depth_test,
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            material: None,
            texture: None,
        }
    }
}
