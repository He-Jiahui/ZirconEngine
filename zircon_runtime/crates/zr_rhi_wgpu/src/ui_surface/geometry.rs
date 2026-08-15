use bytemuck::{Pod, Zeroable};
use glyphon::TextBounds;

use zr_rhi::{
    UiSurfaceCommand, UiSurfaceDrawList, UiSurfaceImageUvRect, UiSurfacePresentStats,
    UiSurfacePresentStatsAccumulator, UiSurfaceRect, UiSurfaceResolvedCommandKind,
};

mod clipping;

use clipping::clip_solid_triangles_to_rect;

pub(super) const UI_QUAD_VERTEX_COUNT: u32 = 6;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct SolidVertex {
    pub(super) position: [f32; 2],
    pub(super) color: [f32; 4],
    pub(super) local_position: [f32; 2],
    pub(super) half_extent: [f32; 2],
    pub(super) corner_radius: f32,
    pub(super) border_width: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct SolidInstance {
    pub(super) min_position: [f32; 2],
    pub(super) max_position: [f32; 2],
    pub(super) color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct ImageVertex {
    pub(super) position: [f32; 2],
    pub(super) uv: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct DrawItemOrder {
    pub(super) z_index: i32,
    pub(super) command_index: usize,
    pub(super) sub_index: usize,
}

#[derive(Clone, Debug)]
pub(super) struct SolidItem {
    pub(super) order: DrawItemOrder,
    pub(super) rect: UiSurfaceRect,
    pub(super) geometry: SolidGeometry,
}

#[derive(Clone, Debug)]
pub(super) enum SolidGeometry {
    Vertices(Vec<SolidVertex>),
    Instance(SolidInstance),
}

impl SolidItem {
    #[cfg(test)]
    pub(super) fn vertices(&self) -> &[SolidVertex] {
        match &self.geometry {
            SolidGeometry::Vertices(vertices) => vertices,
            SolidGeometry::Instance(_) => &[],
        }
    }

    #[cfg(test)]
    pub(super) fn instance(&self) -> Option<SolidInstance> {
        match self.geometry {
            SolidGeometry::Instance(instance) => Some(instance),
            SolidGeometry::Vertices(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ImageItem {
    pub(super) order: DrawItemOrder,
    pub(super) rect: UiSurfaceRect,
    pub(super) resource_key: String,
    pub(super) resource_generation: u64,
    pub(super) vertices: [ImageVertex; 6],
}

#[derive(Clone, Debug)]
pub(super) struct TextItem {
    pub(super) order: DrawItemOrder,
    pub(super) rect: UiSurfaceRect,
    pub(super) command_index: usize,
}

#[derive(Clone, Debug)]
pub(super) enum DrawItem {
    Solid(SolidItem),
    Image(ImageItem),
    Text(TextItem),
}

impl DrawItem {
    pub(super) fn order(&self) -> DrawItemOrder {
        match self {
            DrawItem::Solid(item) => item.order,
            DrawItem::Image(item) => item.order,
            DrawItem::Text(item) => item.order,
        }
    }

    pub(super) fn rect(&self) -> UiSurfaceRect {
        match self {
            DrawItem::Solid(item) => item.rect,
            DrawItem::Image(item) => item.rect,
            DrawItem::Text(item) => item.rect,
        }
    }
}

pub(super) fn draw_items(draw_list: &UiSurfaceDrawList) -> Vec<DrawItem> {
    draw_items_with_damage(draw_list, draw_list.damage, None, None)
}

pub(super) fn draw_items_with_stats(
    draw_list: &UiSurfaceDrawList,
) -> (Vec<DrawItem>, UiSurfacePresentStats) {
    let mut stats = UiSurfacePresentStatsAccumulator::new(draw_list);
    let items = draw_items_with_damage(draw_list, draw_list.damage, Some(&mut stats), None);
    (items, stats.finish())
}

pub(super) fn full_projection_draw_items_with_stats(
    draw_list: &UiSurfaceDrawList,
) -> (
    Vec<DrawItem>,
    UiSurfacePresentStats,
    Option<UiSurfacePresentStats>,
) {
    let mut full_stats = UiSurfacePresentStatsAccumulator::new(draw_list);
    let mut damage_stats = draw_list
        .damage
        .map(|_| UiSurfacePresentStatsAccumulator::new(draw_list));
    let damage_projection = draw_list.damage.zip(damage_stats.as_mut());
    let items = draw_items_with_damage(draw_list, None, Some(&mut full_stats), damage_projection);
    (
        items,
        full_stats.finish(),
        damage_stats.map(UiSurfacePresentStatsAccumulator::finish),
    )
}

fn draw_items_with_damage<'a>(
    draw_list: &'a UiSurfaceDrawList,
    damage: Option<UiSurfaceRect>,
    mut stats: Option<&mut UiSurfacePresentStatsAccumulator<'a>>,
    mut secondary_stats: Option<(UiSurfaceRect, &mut UiSurfacePresentStatsAccumulator<'a>)>,
) -> Vec<DrawItem> {
    let mut items = Vec::new();
    for (command_index, command) in ordered_commands(draw_list) {
        if let Some(stats) = stats.as_deref_mut() {
            if draw_list.command_visible_with_damage(command, damage) {
                stats.record_visible(command, draw_list);
            }
        }
        if let Some((secondary_damage, secondary_stats)) = secondary_stats.as_mut() {
            if draw_list.command_visible_with_damage(command, Some(*secondary_damage)) {
                secondary_stats.record_visible(command, draw_list);
            }
        }
        let Some(kind) = draw_list.resolved_kind(command) else {
            continue;
        };
        match kind {
            UiSurfaceResolvedCommandKind::Quad {
                color,
                corner_radius,
            } => {
                push_solid_item(
                    &mut items,
                    DrawItemOrder {
                        z_index: command.z_index,
                        command_index,
                        sub_index: 0,
                    },
                    command.frame,
                    color,
                    corner_radius,
                    command,
                    draw_list,
                    damage,
                );
            }
            UiSurfaceResolvedCommandKind::Border {
                color,
                width,
                corner_radius,
            } => {
                if corner_radius > 0.0 {
                    push_rounded_border_item(
                        &mut items,
                        DrawItemOrder {
                            z_index: command.z_index,
                            command_index,
                            sub_index: 0,
                        },
                        color,
                        width,
                        corner_radius,
                        command,
                        draw_list,
                        damage,
                    );
                } else {
                    for (sub_index, rect) in
                        border_rects(command.frame, width).into_iter().enumerate()
                    {
                        push_solid_item(
                            &mut items,
                            DrawItemOrder {
                                z_index: command.z_index,
                                command_index,
                                sub_index,
                            },
                            rect,
                            color,
                            0.0,
                            command,
                            draw_list,
                            damage,
                        );
                    }
                }
            }
            UiSurfaceResolvedCommandKind::Image { payload } => {
                let Some(rect) = primitive_effective_rect(
                    command,
                    command.frame,
                    draw_list.projection_size(),
                    damage,
                ) else {
                    continue;
                };
                if payload
                    .atlas_uv
                    .as_ref()
                    .is_some_and(|atlas_uv| !atlas_uv.is_valid())
                {
                    continue;
                }
                items.push(DrawItem::Image(ImageItem {
                    order: DrawItemOrder {
                        z_index: command.z_index,
                        command_index,
                        sub_index: 0,
                    },
                    rect,
                    resource_key: payload.resource_key.clone(),
                    resource_generation: payload.resource_generation,
                    vertices: image_vertices(
                        command.frame,
                        rect,
                        draw_list.projection_size(),
                        payload.atlas_uv,
                    ),
                }));
            }
            UiSurfaceResolvedCommandKind::Text { .. } => {
                let Some(rect) =
                    effective_rect(command, command.frame, draw_list.projection_size(), damage)
                else {
                    continue;
                };
                items.push(DrawItem::Text(TextItem {
                    order: DrawItemOrder {
                        z_index: command.z_index,
                        command_index,
                        sub_index: 0,
                    },
                    rect,
                    command_index,
                }));
            }
            UiSurfaceResolvedCommandKind::Clip => {}
        }
    }
    items
}

pub(super) fn ordered_commands(draw_list: &UiSurfaceDrawList) -> Vec<(usize, &UiSurfaceCommand)> {
    let mut ordered = draw_list.commands.iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(index, command)| (command.z_index, *index));
    ordered
}

fn push_solid_item(
    items: &mut Vec<DrawItem>,
    order: DrawItemOrder,
    frame: UiSurfaceRect,
    color: [u8; 4],
    corner_radius: f32,
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
    damage: Option<UiSurfaceRect>,
) {
    let Some(effective) =
        effective_rect_with_clip_status(command, frame, draw_list.projection_size(), damage)
    else {
        return;
    };
    let rect = effective.rect;
    let geometry = if corner_radius.is_finite() && corner_radius > 0.0 {
        let vertices = solid_vertices(frame, color, draw_list.projection_size(), corner_radius);
        let vertices = if effective.clipped {
            clip_solid_triangles_to_rect(vertices, rect, draw_list.projection_size())
        } else {
            vertices
        };
        SolidGeometry::Vertices(vertices)
    } else {
        SolidGeometry::Instance(solid_instance(rect, color, draw_list.projection_size()))
    };
    items.push(DrawItem::Solid(SolidItem {
        order,
        rect,
        geometry,
    }));
}

fn push_rounded_border_item(
    items: &mut Vec<DrawItem>,
    order: DrawItemOrder,
    color: [u8; 4],
    width: f32,
    corner_radius: f32,
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
    damage: Option<UiSurfaceRect>,
) {
    let Some(effective) = effective_rect_with_clip_status(
        command,
        command.frame,
        draw_list.projection_size(),
        damage,
    ) else {
        return;
    };
    let rect = effective.rect;
    let vertices = rounded_border_vertices(
        command.frame,
        color,
        draw_list.projection_size(),
        width,
        corner_radius,
    );
    let vertices = if effective.clipped {
        clip_solid_triangles_to_rect(vertices, rect, draw_list.projection_size())
    } else {
        vertices
    };
    items.push(DrawItem::Solid(SolidItem {
        order,
        rect,
        geometry: SolidGeometry::Vertices(vertices),
    }));
}

fn solid_instance(frame: UiSurfaceRect, color: [u8; 4], size: (u32, u32)) -> SolidInstance {
    let positions = quad_positions(frame, size);
    SolidInstance {
        min_position: positions[2],
        max_position: positions[1],
        color: normalized_color(color),
    }
}

fn solid_vertices(
    frame: UiSurfaceRect,
    color: [u8; 4],
    size: (u32, u32),
    corner_radius: f32,
) -> Vec<SolidVertex> {
    let corner_radius = clamped_corner_radius(frame, corner_radius);
    let positions = quad_positions(frame, size);
    let color = normalized_color(color);
    let half_extent = [frame.width * 0.5, frame.height * 0.5];
    let local_positions = [
        [-half_extent[0], -half_extent[1]],
        [half_extent[0], -half_extent[1]],
        [-half_extent[0], half_extent[1]],
        [half_extent[0], half_extent[1]],
    ];
    let vertex = |index| SolidVertex {
        position: positions[index],
        color,
        local_position: local_positions[index],
        half_extent,
        corner_radius,
        border_width: 0.0,
    };
    vec![
        vertex(0),
        vertex(1),
        vertex(2),
        vertex(2),
        vertex(1),
        vertex(3),
    ]
}

fn rounded_border_vertices(
    frame: UiSurfaceRect,
    color: [u8; 4],
    size: (u32, u32),
    width: f32,
    corner_radius: f32,
) -> Vec<SolidVertex> {
    let width = width.max(1.0).min(frame.width.min(frame.height) * 0.5);
    let mut vertices = solid_vertices(frame, color, size, corner_radius);
    for vertex in &mut vertices {
        vertex.border_width = width;
    }
    vertices
}

fn clamped_corner_radius(frame: UiSurfaceRect, corner_radius: f32) -> f32 {
    if !corner_radius.is_finite() {
        return 0.0;
    }
    corner_radius
        .max(0.0)
        .min(frame.width.min(frame.height).max(0.0) * 0.5)
}

fn normalized_color(color: [u8; 4]) -> [f32; 4] {
    [
        color[0] as f32 / 255.0,
        color[1] as f32 / 255.0,
        color[2] as f32 / 255.0,
        color[3] as f32 / 255.0,
    ]
}

fn image_vertices(
    frame: UiSurfaceRect,
    visible_rect: UiSurfaceRect,
    size: (u32, u32),
    atlas_uv: Option<UiSurfaceImageUvRect>,
) -> [ImageVertex; 6] {
    let positions = quad_positions(visible_rect, size);
    let frame_width = positive_finite_extent(frame.width);
    let frame_height = positive_finite_extent(frame.height);
    let local_u0 = ((visible_rect.x - frame.x) / frame_width).clamp(0.0, 1.0);
    let local_v0 = ((visible_rect.y - frame.y) / frame_height).clamp(0.0, 1.0);
    let local_u1 = ((visible_rect.x + visible_rect.width - frame.x) / frame_width).clamp(0.0, 1.0);
    let local_v1 =
        ((visible_rect.y + visible_rect.height - frame.y) / frame_height).clamp(0.0, 1.0);
    let (u0, v0, u1, v1) = image_uv_bounds(local_u0, local_v0, local_u1, local_v1, atlas_uv);
    [
        ImageVertex {
            position: positions[0],
            uv: [u0, v0],
        },
        ImageVertex {
            position: positions[1],
            uv: [u1, v0],
        },
        ImageVertex {
            position: positions[2],
            uv: [u0, v1],
        },
        ImageVertex {
            position: positions[2],
            uv: [u0, v1],
        },
        ImageVertex {
            position: positions[1],
            uv: [u1, v0],
        },
        ImageVertex {
            position: positions[3],
            uv: [u1, v1],
        },
    ]
}

fn positive_finite_extent(extent: f32) -> f32 {
    if extent.is_finite() && extent > 0.0 {
        extent
    } else {
        1.0
    }
}

fn image_uv_bounds(
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    atlas_uv: Option<UiSurfaceImageUvRect>,
) -> (f32, f32, f32, f32) {
    let Some(atlas_uv) = atlas_uv else {
        return (u0, v0, u1, v1);
    };
    let width = atlas_uv.max[0] - atlas_uv.min[0];
    let height = atlas_uv.max[1] - atlas_uv.min[1];
    (
        atlas_uv.min[0] + u0 * width,
        atlas_uv.min[1] + v0 * height,
        atlas_uv.min[0] + u1 * width,
        atlas_uv.min[1] + v1 * height,
    )
}

fn quad_positions(frame: UiSurfaceRect, size: (u32, u32)) -> [[f32; 2]; 4] {
    let width = size.0.max(1) as f32;
    let height = size.1.max(1) as f32;
    let left = (frame.x / width) * 2.0 - 1.0;
    let right = ((frame.x + frame.width) / width) * 2.0 - 1.0;
    let top = 1.0 - (frame.y / height) * 2.0;
    let bottom = 1.0 - ((frame.y + frame.height) / height) * 2.0;
    [[left, top], [right, top], [left, bottom], [right, bottom]]
}

fn border_rects(frame: UiSurfaceRect, width: f32) -> [UiSurfaceRect; 4] {
    let width = width.max(1.0);
    [
        UiSurfaceRect::new(frame.x, frame.y, frame.width, width),
        UiSurfaceRect::new(
            frame.x,
            (frame.y + frame.height - width).max(frame.y),
            frame.width,
            width,
        ),
        UiSurfaceRect::new(frame.x, frame.y, width, frame.height),
        UiSurfaceRect::new(
            (frame.x + frame.width - width).max(frame.x),
            frame.y,
            width,
            frame.height,
        ),
    ]
}

pub(super) fn command_effective_rect(
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
) -> Option<UiSurfaceRect> {
    effective_rect(
        command,
        command.frame,
        draw_list.projection_size(),
        draw_list.damage,
    )
}

pub(super) fn full_projection_effective_rect(
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
) -> Option<UiSurfaceRect> {
    effective_rect(command, command.frame, draw_list.projection_size(), None)
}

fn primitive_effective_rect(
    command: &UiSurfaceCommand,
    primitive_frame: UiSurfaceRect,
    surface_size: (u32, u32),
    damage: Option<UiSurfaceRect>,
) -> Option<UiSurfaceRect> {
    effective_rect(command, primitive_frame, surface_size, damage)
}

fn effective_rect(
    command: &UiSurfaceCommand,
    primitive_frame: UiSurfaceRect,
    surface_size: (u32, u32),
    damage: Option<UiSurfaceRect>,
) -> Option<UiSurfaceRect> {
    effective_rect_with_clip_status(command, primitive_frame, surface_size, damage)
        .map(|effective| effective.rect)
}

#[derive(Clone, Copy)]
struct EffectiveRect {
    rect: UiSurfaceRect,
    clipped: bool,
}

fn effective_rect_with_clip_status(
    command: &UiSurfaceCommand,
    primitive_frame: UiSurfaceRect,
    surface_size: (u32, u32),
    damage: Option<UiSurfaceRect>,
) -> Option<EffectiveRect> {
    let surface = UiSurfaceRect::new(0.0, 0.0, surface_size.0 as f32, surface_size.1 as f32);
    let mut clipped = !surface.contains_rect(primitive_frame);
    let mut rect = primitive_frame.intersection(surface)?;
    if let Some(clip) = command.clip {
        clipped |= !clip.contains_rect(primitive_frame);
        rect = rect.intersection(clip)?;
    }
    if let Some(damage) = damage {
        clipped |= !damage.contains_rect(primitive_frame);
        rect = rect.intersection(damage)?;
    }
    Some(EffectiveRect { rect, clipped })
}

trait RectExt {
    fn intersection(self, other: UiSurfaceRect) -> Option<UiSurfaceRect>;
    fn contains_rect(self, other: UiSurfaceRect) -> bool;
}

impl RectExt for UiSurfaceRect {
    fn intersection(self, other: UiSurfaceRect) -> Option<UiSurfaceRect> {
        if !self.has_finite_positive_area() || !other.has_finite_positive_area() {
            return None;
        }
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        (right > left && bottom > top)
            .then(|| UiSurfaceRect::new(left, top, right - left, bottom - top))
    }

    fn contains_rect(self, other: UiSurfaceRect) -> bool {
        self.has_finite_positive_area()
            && other.has_finite_positive_area()
            && self.x <= other.x
            && self.y <= other.y
            && self.x + self.width >= other.x + other.width
            && self.y + self.height >= other.y + other.height
    }
}

pub(super) fn text_bounds_from_rect(clip: UiSurfaceRect) -> TextBounds {
    TextBounds {
        left: clip.x.max(0.0).floor() as i32,
        top: clip.y.max(0.0).floor() as i32,
        right: (clip.x + clip.width).max(0.0).ceil() as i32,
        bottom: (clip.y + clip.height).max(0.0).ceil() as i32,
    }
}

#[cfg(test)]
mod tests;
