use crate::core::framework::render::{RenderPhase, RenderPhaseMeshSource};
use crate::core::math::{Vec2, Vec3};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::types::ViewportRenderFrame;

use super::sprite_vertex::SpriteVertex;

pub(in crate::graphics::scene::scene_renderer::sprite) fn build_sprite_vertices(
    frame: &ViewportRenderFrame,
    stage: RenderPassStage,
) -> Vec<(usize, Vec<SpriteVertex>)> {
    let phase = match stage {
        RenderPassStage::Opaque2d => RenderPhase::Opaque2d,
        RenderPassStage::AlphaMask2d => RenderPhase::AlphaMask2d,
        RenderPassStage::Transparent2d => RenderPhase::Transparent2d,
        _ => return Vec::new(),
    };
    let phase_items = frame
        .extract
        .sprites
        .phase_queue
        .items_for_phase(phase)
        .filter_map(|item| match item.mesh_source {
            RenderPhaseMeshSource::SpriteIndex(index) => Some(index),
            RenderPhaseMeshSource::MeshIndex(_) => None,
        })
        .collect::<Vec<_>>();
    let sprite_indices = if phase_items.is_empty() {
        frame
            .sprites()
            .iter()
            .enumerate()
            .filter_map(|(index, sprite)| {
                (RenderPhase::mesh_phase(
                    frame.extract.view.core_pipeline,
                    matches!(
                        sprite.material_alpha_mode,
                        crate::core::framework::render::RenderMaterialAlphaMode::Mask { .. }
                    ),
                    matches!(
                        sprite.material_alpha_mode,
                        crate::core::framework::render::RenderMaterialAlphaMode::Blend
                    ),
                ) == phase)
                    .then_some(index)
            })
            .collect::<Vec<_>>()
    } else {
        phase_items
    };

    sprite_indices
        .into_iter()
        .filter_map(|index| frame.sprites().get(index).map(|sprite| (index, sprite)))
        .filter_map(|(index, sprite)| {
            if !sprite.color.is_finite() || sprite.color.w <= f32::EPSILON {
                return None;
            }
            let size = sprite.custom_size.unwrap_or_else(|| {
                sprite
                    .rect
                    .map(|rect| rect.max - rect.min)
                    .unwrap_or(Vec2::ONE)
            });
            if !size.is_finite() || size.x.abs() <= f32::EPSILON || size.y.abs() <= f32::EPSILON {
                return None;
            }
            let vertices = sprite_quad_vertices(sprite, size);
            (!vertices.is_empty()).then_some((index, vertices))
        })
        .collect()
}

fn sprite_quad_vertices(
    sprite: &crate::core::framework::render::RenderSpriteSnapshot,
    size: Vec2,
) -> Vec<SpriteVertex> {
    let anchor = sprite.anchor.normalized;
    let left = -anchor.x * size.x;
    let right = (1.0 - anchor.x) * size.x;
    let bottom = -anchor.y * size.y;
    let top = (1.0 - anchor.y) * size.y;
    let transform = sprite.transform.matrix();
    let position = |x: f32, y: f32| transform.transform_point3(Vec3::new(x, y, 0.0));
    let (mut uv_min, mut uv_max) = sprite
        .atlas_region
        .map(|region| (region.min, region.max))
        .unwrap_or((Vec2::ZERO, Vec2::ONE));
    if !uv_min.is_finite() || !uv_max.is_finite() {
        uv_min = Vec2::ZERO;
        uv_max = Vec2::ONE;
    }
    if sprite.flip_x {
        std::mem::swap(&mut uv_min.x, &mut uv_max.x);
    }
    if sprite.flip_y {
        std::mem::swap(&mut uv_min.y, &mut uv_max.y);
    }
    let top_left = position(left, top);
    let top_right = position(right, top);
    let bottom_left = position(left, bottom);
    let bottom_right = position(right, bottom);
    vec![
        SpriteVertex::new(top_left, Vec2::new(uv_min.x, uv_max.y), sprite.color),
        SpriteVertex::new(bottom_left, Vec2::new(uv_min.x, uv_min.y), sprite.color),
        SpriteVertex::new(top_right, Vec2::new(uv_max.x, uv_max.y), sprite.color),
        SpriteVertex::new(top_right, Vec2::new(uv_max.x, uv_max.y), sprite.color),
        SpriteVertex::new(bottom_left, Vec2::new(uv_min.x, uv_min.y), sprite.color),
        SpriteVertex::new(bottom_right, Vec2::new(uv_max.x, uv_min.y), sprite.color),
    ]
}
