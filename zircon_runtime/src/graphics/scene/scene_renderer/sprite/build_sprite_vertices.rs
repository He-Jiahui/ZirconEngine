use crate::core::framework::render::{
    RenderPhase, RenderPhaseMeshSource, RenderQueueValue, RenderSpriteImageMode, RenderSpriteRect,
    RenderSpriteScalingMode, RenderSpriteSliceScaleMode, RenderSpriteSlicer,
};
use crate::core::math::{Mat4, Vec2, Vec3};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::types::ViewportRenderFrame;

use super::sprite_vertex::SpriteVertex;

const MAX_SPRITE_IMAGE_SLICES: usize = 1_000;
const MIN_TILE_EXTENT: f32 = 1.0;
const MIN_STRETCH_VALUE: f32 = 0.001;

pub(super) fn visit_stage_sprites(
    frame: &ViewportRenderFrame,
    stage: RenderPassStage,
    mut visit: impl FnMut(usize, &crate::core::framework::render::RenderSpriteSnapshot, Vec2),
) {
    let phase = match stage {
        RenderPassStage::Opaque2d => RenderPhase::Opaque2d,
        RenderPassStage::AlphaMask2d => RenderPhase::AlphaMask2d,
        RenderPassStage::Transparent2d => RenderPhase::Transparent2d,
        RenderPassStage::Transparent3d => RenderPhase::Transparent3d,
        _ => return,
    };
    let mut phase_items = frame
        .extract
        .sprites
        .phase_queue
        .items_for_phase(phase)
        .filter_map(|item| match item.mesh_source {
            RenderPhaseMeshSource::SpriteIndex(index) => Some(index),
            RenderPhaseMeshSource::MeshIndex(_) => None,
        });
    let first_phase_item = phase_items.next();
    let camera_layers = frame.extract.view.selected_camera_layers();

    if let Some(first_phase_item) = first_phase_item {
        for index in std::iter::once(first_phase_item).chain(phase_items) {
            let Some((sprite, size)) = sprite_vertex_source(frame, index, &camera_layers) else {
                continue;
            };
            visit(index, sprite, size);
        }
    } else {
        for (index, sprite) in frame.sprites().iter().enumerate() {
            if RenderQueueValue::from_alpha_mode(&sprite.material_alpha_mode)
                .phase(frame.extract.view.core_pipeline)
                != phase
            {
                continue;
            }
            let Some((sprite, size)) = sprite_vertex_source(frame, index, &camera_layers) else {
                continue;
            };
            visit(index, sprite, size);
        }
    }
}

fn sprite_vertex_source<'a>(
    frame: &'a ViewportRenderFrame,
    index: usize,
    camera_layers: &crate::core::framework::render::RenderLayerSet,
) -> Option<(
    &'a crate::core::framework::render::RenderSpriteSnapshot,
    Vec2,
)> {
    frame.sprites().get(index).and_then(|sprite| {
        if !camera_layers.intersects(&sprite.common.layer_mask) {
            return None;
        }
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
        Some((sprite, size))
    })
}

pub(super) fn append_sprite_image_vertices(
    sprite: &crate::core::framework::render::RenderSpriteSnapshot,
    size: Vec2,
    vertices: &mut Vec<SpriteVertex>,
) {
    let base_rect = sprite_base_rect(sprite);
    let transform = sprite.transform.matrix();
    visit_sprite_image_slices(sprite.image_mode, base_rect, size, |slice| {
        append_sprite_quad_vertices(sprite, size, slice, &transform, vertices);
    });
}

pub(crate) fn build_sprite_vertices(
    frame: &ViewportRenderFrame,
    stage: RenderPassStage,
) -> Vec<(usize, Vec<SpriteVertex>)> {
    let mut projected = Vec::new();
    visit_stage_sprites(frame, stage, |index, sprite, size| {
        let mut vertices = Vec::new();
        append_sprite_image_vertices(sprite, size, &mut vertices);
        if !vertices.is_empty() {
            projected.push((index, vertices));
        }
    });
    projected
}

fn sprite_base_rect(
    sprite: &crate::core::framework::render::RenderSpriteSnapshot,
) -> RenderSpriteRect {
    sprite.rect.unwrap_or(RenderSpriteRect {
        min: Vec2::ZERO,
        max: Vec2::ONE,
    })
}

fn append_sprite_quad_vertices(
    sprite: &crate::core::framework::render::RenderSpriteSnapshot,
    sprite_size: Vec2,
    slice: SpriteImageSlice,
    transform: &Mat4,
    vertices: &mut Vec<SpriteVertex>,
) {
    let anchor = sprite.anchor.normalized;
    let anchor_offset = Vec2::new(-anchor.x * sprite_size.x, -anchor.y * sprite_size.y);
    let min = anchor_offset + slice.offset - slice.draw_size * 0.5;
    let max = anchor_offset + slice.offset + slice.draw_size * 0.5;
    let position = |x: f32, y: f32| transform.transform_point3(Vec3::new(x, y, 0.0));
    let (atlas_min, atlas_max) = sprite
        .atlas_region
        .map(|region| (region.min, region.max))
        .unwrap_or((Vec2::ZERO, Vec2::ONE));
    let (atlas_min, atlas_max) = if atlas_min.is_finite() && atlas_max.is_finite() {
        (atlas_min, atlas_max)
    } else {
        (Vec2::ZERO, Vec2::ONE)
    };
    let base_rect = sprite_base_rect(sprite);
    let mut uv_min = remap_texture_point(slice.texture_rect.min, base_rect, atlas_min, atlas_max);
    let mut uv_max = remap_texture_point(slice.texture_rect.max, base_rect, atlas_min, atlas_max);
    if !uv_min.is_finite() || !uv_max.is_finite() {
        uv_min = atlas_min;
        uv_max = atlas_max;
    }
    if sprite.flip_x {
        std::mem::swap(&mut uv_min.x, &mut uv_max.x);
    }
    if sprite.flip_y {
        std::mem::swap(&mut uv_min.y, &mut uv_max.y);
    }
    let top_left = position(min.x, max.y);
    let top_right = position(max.x, max.y);
    let bottom_left = position(min.x, min.y);
    let bottom_right = position(max.x, min.y);
    vertices.extend([
        SpriteVertex::new(top_left, Vec2::new(uv_min.x, uv_max.y), sprite.color),
        SpriteVertex::new(bottom_left, Vec2::new(uv_min.x, uv_min.y), sprite.color),
        SpriteVertex::new(top_right, Vec2::new(uv_max.x, uv_max.y), sprite.color),
        SpriteVertex::new(top_right, Vec2::new(uv_max.x, uv_max.y), sprite.color),
        SpriteVertex::new(bottom_left, Vec2::new(uv_min.x, uv_min.y), sprite.color),
        SpriteVertex::new(bottom_right, Vec2::new(uv_max.x, uv_min.y), sprite.color),
    ]);
}

fn remap_texture_point(
    point: Vec2,
    base_rect: RenderSpriteRect,
    atlas_min: Vec2,
    atlas_max: Vec2,
) -> Vec2 {
    let base_size = base_rect.max - base_rect.min;
    if base_size.x.abs() <= f32::EPSILON || base_size.y.abs() <= f32::EPSILON {
        return atlas_min;
    }
    let normalized = (point - base_rect.min) / base_size;
    atlas_min + (atlas_max - atlas_min) * normalized
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SpriteImageSlice {
    texture_rect: RenderSpriteRect,
    draw_size: Vec2,
    offset: Vec2,
}

fn visit_sprite_image_slices(
    image_mode: RenderSpriteImageMode,
    base_rect: RenderSpriteRect,
    draw_size: Vec2,
    mut visit: impl FnMut(SpriteImageSlice),
) {
    let mut emitted = 0;
    match image_mode {
        RenderSpriteImageMode::Stretch => emit_sprite_image_slice(
            SpriteImageSlice {
                texture_rect: base_rect,
                draw_size,
                offset: draw_size * 0.5,
            },
            &mut emitted,
            &mut visit,
        ),
        RenderSpriteImageMode::Scale(scaling_mode) => {
            if let Some(slice) = scaled_image_slice(scaling_mode, base_rect, draw_size) {
                emit_sprite_image_slice(slice, &mut emitted, &mut visit);
            }
        }
        RenderSpriteImageMode::Tiled {
            tile_x,
            tile_y,
            stretch_value,
        } => visit_tile_slices(
            SpriteImageSlice {
                texture_rect: base_rect,
                draw_size,
                offset: draw_size * 0.5,
            },
            stretch_value,
            tile_x,
            tile_y,
            &mut emitted,
            &mut visit,
        ),
        RenderSpriteImageMode::Sliced(slicer) => {
            visit_sliced_image_slices(slicer, base_rect, draw_size, &mut emitted, &mut visit)
        }
    }
}

fn emit_sprite_image_slice(
    slice: SpriteImageSlice,
    emitted: &mut usize,
    visit: &mut impl FnMut(SpriteImageSlice),
) {
    if *emitted >= MAX_SPRITE_IMAGE_SLICES {
        return;
    }
    *emitted += 1;
    visit(slice);
}

fn scaled_image_slice(
    scaling_mode: RenderSpriteScalingMode,
    base_rect: RenderSpriteRect,
    draw_size: Vec2,
) -> Option<SpriteImageSlice> {
    let base_size = base_rect.max - base_rect.min;
    if !valid_positive_size(base_size) || !valid_positive_size(draw_size) {
        return None;
    }
    match scaling_mode {
        RenderSpriteScalingMode::FitCenter
        | RenderSpriteScalingMode::FitStart
        | RenderSpriteScalingMode::FitEnd => {
            let scale = (draw_size.x / base_size.x).min(draw_size.y / base_size.y);
            let scaled_size = base_size * scale;
            let offset = fit_offset(scaling_mode, draw_size, scaled_size);
            Some(SpriteImageSlice {
                texture_rect: base_rect,
                draw_size: scaled_size,
                offset,
            })
        }
        RenderSpriteScalingMode::FillCenter
        | RenderSpriteScalingMode::FillStart
        | RenderSpriteScalingMode::FillEnd => Some(SpriteImageSlice {
            texture_rect: fill_texture_rect(scaling_mode, base_rect, draw_size),
            draw_size,
            offset: draw_size * 0.5,
        }),
    }
}

fn fit_offset(scaling_mode: RenderSpriteScalingMode, draw_size: Vec2, scaled_size: Vec2) -> Vec2 {
    match scaling_mode {
        RenderSpriteScalingMode::FitStart => {
            Vec2::new(scaled_size.x * 0.5, draw_size.y - scaled_size.y * 0.5)
        }
        RenderSpriteScalingMode::FitEnd => {
            Vec2::new(draw_size.x - scaled_size.x * 0.5, scaled_size.y * 0.5)
        }
        _ => draw_size * 0.5,
    }
}

fn fill_texture_rect(
    scaling_mode: RenderSpriteScalingMode,
    base_rect: RenderSpriteRect,
    draw_size: Vec2,
) -> RenderSpriteRect {
    let base_size = base_rect.max - base_rect.min;
    let base_aspect = base_size.x / base_size.y;
    let draw_aspect = draw_size.x / draw_size.y;
    if base_aspect > draw_aspect {
        let crop_width = base_size.y * draw_aspect;
        let min_x = aligned_min(
            scaling_mode,
            base_rect.min.x,
            base_rect.max.x,
            crop_width,
            false,
        );
        RenderSpriteRect::new(
            Vec2::new(min_x, base_rect.min.y),
            Vec2::new(min_x + crop_width, base_rect.max.y),
        )
    } else {
        let crop_height = base_size.x / draw_aspect;
        let min_y = aligned_min(
            scaling_mode,
            base_rect.min.y,
            base_rect.max.y,
            crop_height,
            true,
        );
        RenderSpriteRect::new(
            Vec2::new(base_rect.min.x, min_y),
            Vec2::new(base_rect.max.x, min_y + crop_height),
        )
    }
}

fn aligned_min(
    scaling_mode: RenderSpriteScalingMode,
    min: f32,
    max: f32,
    extent: f32,
    vertical_axis: bool,
) -> f32 {
    let overflow = max - min - extent;
    match scaling_mode {
        RenderSpriteScalingMode::FillStart | RenderSpriteScalingMode::FitStart => {
            if vertical_axis {
                max - extent
            } else {
                min
            }
        }
        RenderSpriteScalingMode::FillEnd | RenderSpriteScalingMode::FitEnd => {
            if vertical_axis {
                min
            } else {
                max - extent
            }
        }
        _ => min + overflow * 0.5,
    }
}

fn visit_sliced_image_slices(
    slicer: RenderSpriteSlicer,
    base_rect: RenderSpriteRect,
    draw_size: Vec2,
    emitted: &mut usize,
    visit: &mut impl FnMut(SpriteImageSlice),
) {
    let base_size = base_rect.max - base_rect.min;
    if !valid_positive_size(base_size) || !valid_positive_size(draw_size) {
        return;
    }
    let left = slicer.border.left.max(0.0);
    let right = slicer.border.right.max(0.0);
    let top = slicer.border.top.max(0.0);
    let bottom = slicer.border.bottom.max(0.0);
    if left + right >= base_size.x || top + bottom >= base_size.y {
        emit_sprite_image_slice(
            SpriteImageSlice {
                texture_rect: base_rect,
                draw_size,
                offset: draw_size * 0.5,
            },
            emitted,
            visit,
        );
        return;
    }

    let max_corner_scale = slicer.max_corner_scale.max(0.0);
    let corner_scale = (draw_size.x / base_size.x)
        .min(draw_size.y / base_size.y)
        .min(max_corner_scale);
    let left_draw = left * corner_scale;
    let right_draw = right * corner_scale;
    let top_draw = top * corner_scale;
    let bottom_draw = bottom * corner_scale;
    let center_draw_width = (draw_size.x - left_draw - right_draw).max(0.0);
    let center_draw_height = (draw_size.y - top_draw - bottom_draw).max(0.0);

    let x_tex = [
        base_rect.min.x,
        base_rect.min.x + left,
        base_rect.max.x - right,
        base_rect.max.x,
    ];
    let y_tex = [
        base_rect.min.y,
        base_rect.min.y + bottom,
        base_rect.max.y - top,
        base_rect.max.y,
    ];
    let x_draw = [0.0, left_draw, left_draw + center_draw_width, draw_size.x];
    let y_draw = [
        0.0,
        bottom_draw,
        bottom_draw + center_draw_height,
        draw_size.y,
    ];

    for y in 0..3 {
        for x in 0..3 {
            let texture_rect = RenderSpriteRect::new(
                Vec2::new(x_tex[x], y_tex[y]),
                Vec2::new(x_tex[x + 1], y_tex[y + 1]),
            );
            let draw_min = Vec2::new(x_draw[x], y_draw[y]);
            let draw_max = Vec2::new(x_draw[x + 1], y_draw[y + 1]);
            let draw_size = draw_max - draw_min;
            if !valid_positive_size(draw_size) {
                continue;
            }
            let slice = SpriteImageSlice {
                texture_rect,
                draw_size,
                offset: (draw_min + draw_max) * 0.5,
            };
            match slice_scale_mode_for_grid_cell(slicer, x, y) {
                RenderSpriteSliceScaleMode::Stretch => {
                    emit_sprite_image_slice(slice, emitted, visit)
                }
                RenderSpriteSliceScaleMode::Tile { stretch_value } => {
                    let tile_x = x == 1;
                    let tile_y = y == 1;
                    visit_tile_slices(slice, stretch_value, tile_x, tile_y, emitted, visit);
                }
            }
            if *emitted >= MAX_SPRITE_IMAGE_SLICES {
                break;
            }
        }
        if *emitted >= MAX_SPRITE_IMAGE_SLICES {
            break;
        }
    }
}

fn slice_scale_mode_for_grid_cell(
    slicer: RenderSpriteSlicer,
    x: usize,
    y: usize,
) -> RenderSpriteSliceScaleMode {
    match (x, y) {
        (1, 1) => slicer.center_scale_mode,
        (1, _) | (_, 1) => slicer.sides_scale_mode,
        _ => RenderSpriteSliceScaleMode::Stretch,
    }
}

fn visit_tile_slices(
    slice: SpriteImageSlice,
    stretch_value: f32,
    tile_x: bool,
    tile_y: bool,
    emitted: &mut usize,
    visit: &mut impl FnMut(SpriteImageSlice),
) {
    if !tile_x && !tile_y {
        emit_sprite_image_slice(slice, emitted, visit);
        return;
    }
    let stretch_value = stretch_value.max(MIN_STRETCH_VALUE);
    let texture_size = slice.texture_rect.max - slice.texture_rect.min;
    if !valid_positive_size(texture_size) || !valid_positive_size(slice.draw_size) {
        return;
    }
    let tile_size = Vec2::new(
        if tile_x {
            (texture_size.x * stretch_value)
                .max(MIN_TILE_EXTENT)
                .min(slice.draw_size.x)
        } else {
            slice.draw_size.x
        },
        if tile_y {
            (texture_size.y * stretch_value)
                .max(MIN_TILE_EXTENT)
                .min(slice.draw_size.y)
        } else {
            slice.draw_size.y
        },
    );
    let base_min = slice.offset - slice.draw_size * 0.5;
    let mut y = 0.0;
    while y < slice.draw_size.y && *emitted < MAX_SPRITE_IMAGE_SLICES {
        let height = tile_size.y.min(slice.draw_size.y - y);
        let mut x = 0.0;
        while x < slice.draw_size.x && *emitted < MAX_SPRITE_IMAGE_SLICES {
            let width = tile_size.x.min(slice.draw_size.x - x);
            let draw_size = Vec2::new(width, height);
            let ratio = draw_size / tile_size;
            emit_sprite_image_slice(
                SpriteImageSlice {
                    texture_rect: RenderSpriteRect::new(
                        slice.texture_rect.min,
                        slice.texture_rect.min + texture_size * ratio,
                    ),
                    draw_size,
                    offset: base_min + Vec2::new(x, y) + draw_size * 0.5,
                },
                emitted,
                visit,
            );
            x += width;
        }
        y += height;
    }
}

fn valid_positive_size(size: Vec2) -> bool {
    size.is_finite() && size.x > f32::EPSILON && size.y > f32::EPSILON
}

#[cfg(test)]
fn sprite_image_vertices(
    sprite: &crate::core::framework::render::RenderSpriteSnapshot,
    size: Vec2,
) -> Vec<SpriteVertex> {
    let mut vertices = Vec::new();
    append_sprite_image_vertices(sprite, size, &mut vertices);
    vertices
}

#[cfg(test)]
fn sprite_image_slices(
    image_mode: RenderSpriteImageMode,
    base_rect: RenderSpriteRect,
    draw_size: Vec2,
) -> Vec<SpriteImageSlice> {
    let mut slices = Vec::new();
    visit_sprite_image_slices(image_mode, base_rect, draw_size, |slice| slices.push(slice));
    slices
}

#[cfg(test)]
mod tests;
