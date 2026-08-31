use bytemuck::{Pod, Zeroable};
use glyphon::TextBounds;

use zr_rhi::{
    UiSurfaceCommand, UiSurfaceDrawList, UiSurfaceImageUvRect, UiSurfacePresentStats,
    UiSurfacePresentStatsAccumulator, UiSurfaceRect, UiSurfaceResolvedCommandKind,
};

mod clipping;

use clipping::clip_solid_triangles_to_rect;

pub(super) const UI_QUAD_VERTEX_COUNT: u32 = 6;
const ANALYTIC_AA_PADDING: f32 = 1.0;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct SolidVertex {
    pub(super) position: [f32; 2],
    pub(super) color: [f32; 4],
    pub(super) local_position: [f32; 2],
    pub(super) half_extent: [f32; 2],
    pub(super) corner_radius: f32,
    pub(super) border_width: f32,
    pub(super) fill_color: [f32; 4],
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
    pub(super) fused_border_command_index: Option<usize>,
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
    let damage = damage_with_analytic_coverage(draw_list.damage, draw_list.projection_size());
    draw_items_with_damage(draw_list, damage, None, None)
}

pub(super) fn draw_items_with_stats(
    draw_list: &UiSurfaceDrawList,
) -> (Vec<DrawItem>, UiSurfacePresentStats) {
    let damage = damage_with_analytic_coverage(draw_list.damage, draw_list.projection_size());
    let mut stats = UiSurfacePresentStatsAccumulator::new(draw_list);
    let items = draw_items_with_damage(draw_list, damage, Some(&mut stats), None);
    (items, stats.finish())
}

pub(super) fn full_projection_draw_items_with_stats(
    draw_list: &UiSurfaceDrawList,
) -> (
    Vec<DrawItem>,
    UiSurfacePresentStats,
    Option<UiSurfacePresentStats>,
) {
    let damage = damage_with_analytic_coverage(draw_list.damage, draw_list.projection_size());
    let mut full_stats = UiSurfacePresentStatsAccumulator::new(draw_list);
    let mut damage_stats = damage.map(|_| UiSurfacePresentStatsAccumulator::new(draw_list));
    let damage_projection = damage.zip(damage_stats.as_mut());
    let items = draw_items_with_damage(draw_list, None, Some(&mut full_stats), damage_projection);
    let mut full_stats = full_stats.finish();
    let mut damage_stats = damage_stats.map(UiSurfacePresentStatsAccumulator::finish);
    if let Some(damage_stats) = damage_stats.as_mut() {
        let total_visibility_scans = full_stats
            .command_visibility_scan_count
            .saturating_add(damage_stats.command_visibility_scan_count);
        full_stats.command_visibility_scan_count = total_visibility_scans;
        damage_stats.command_visibility_scan_count = total_visibility_scans;
    }
    (items, full_stats, damage_stats)
}

fn draw_items_with_damage<'a>(
    draw_list: &'a UiSurfaceDrawList,
    damage: Option<UiSurfaceRect>,
    mut stats: Option<&mut UiSurfacePresentStatsAccumulator<'a>>,
    mut secondary_stats: Option<(UiSurfaceRect, &mut UiSurfacePresentStatsAccumulator<'a>)>,
) -> Vec<DrawItem> {
    let mut items = Vec::new();
    let ordered = ordered_commands(draw_list);
    let mut fused_border_index = None;
    for (ordered_index, &(command_index, command)) in ordered.iter().enumerate() {
        let primary_visible = draw_list.command_visible_with_damage(command, damage);
        if let Some(stats) = stats.as_deref_mut() {
            stats.record_command_visit();
            if primary_visible {
                stats.record_visible(command, draw_list);
            }
        }
        let secondary_visible = secondary_stats
            .as_ref()
            .is_some_and(|(secondary_damage, _)| {
                draw_list.command_visible_with_damage(command, Some(*secondary_damage))
            });
        if let Some((secondary_damage, secondary_stats)) = secondary_stats.as_mut() {
            secondary_stats.record_command_visit();
            if secondary_visible {
                secondary_stats.record_visible(command, draw_list);
            }
        }
        if fused_border_index == Some(ordered_index) {
            if primary_visible {
                if let Some(stats) = stats.as_deref_mut() {
                    stats.record_draw_item_fusion();
                }
            }
            if secondary_visible {
                if let Some((_, secondary_stats)) = secondary_stats.as_mut() {
                    secondary_stats.record_draw_item_fusion();
                }
            }
            fused_border_index = None;
            continue;
        }
        let Some(kind) = draw_list.resolved_kind(command) else {
            continue;
        };
        match kind {
            UiSurfaceResolvedCommandKind::Quad {
                color,
                corner_radius,
            } => {
                let order = DrawItemOrder {
                    z_index: command.z_index,
                    command_index,
                    sub_index: 0,
                };
                if let Some((border_command_index, border_color, width)) = matching_border(
                    draw_list,
                    &ordered,
                    ordered_index,
                    command,
                    color,
                    corner_radius,
                ) {
                    push_rounded_box_item(
                        &mut items,
                        order,
                        color,
                        border_color,
                        width,
                        corner_radius,
                        border_command_index,
                        command,
                        draw_list,
                        damage,
                    );
                    fused_border_index = Some(ordered_index + 1);
                } else {
                    push_solid_item(
                        &mut items,
                        order,
                        command.frame,
                        color,
                        corner_radius,
                        command,
                        draw_list,
                        damage,
                    );
                }
            }
            UiSurfaceResolvedCommandKind::Border {
                color,
                width,
                corner_radius,
            } => {
                push_border_item(
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
    let analytic = (corner_radius.is_finite() && corner_radius > 0.0)
        || !rect_edges_are_physical_pixel_aligned(frame);
    let raster_frame = if analytic {
        let Some(raster_frame) = analytic_raster_frame(frame) else {
            return;
        };
        raster_frame
    } else {
        frame
    };
    let Some(effective) =
        effective_rect_with_clip_status(command, raster_frame, draw_list.projection_size(), damage)
    else {
        return;
    };
    let rect = effective.rect;
    let geometry = if analytic {
        let vertices = solid_vertices(
            frame,
            raster_frame,
            color,
            draw_list.projection_size(),
            corner_radius,
        );
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
        fused_border_command_index: None,
    }));
}

fn push_border_item(
    items: &mut Vec<DrawItem>,
    order: DrawItemOrder,
    color: [u8; 4],
    width: f32,
    corner_radius: f32,
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
    damage: Option<UiSurfaceRect>,
) {
    if !width.is_finite() || width <= 0.0 {
        return;
    }
    let Some(raster_frame) = analytic_raster_frame(command.frame) else {
        return;
    };
    let Some(effective) =
        effective_rect_with_clip_status(command, raster_frame, draw_list.projection_size(), damage)
    else {
        return;
    };
    let rect = effective.rect;
    let vertices = rounded_border_vertices(
        command.frame,
        raster_frame,
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
        fused_border_command_index: None,
    }));
}

fn push_rounded_box_item(
    items: &mut Vec<DrawItem>,
    order: DrawItemOrder,
    fill_color: [u8; 4],
    border_color: [u8; 4],
    width: f32,
    corner_radius: f32,
    fused_border_command_index: usize,
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
    damage: Option<UiSurfaceRect>,
) {
    let Some(raster_frame) = analytic_raster_frame(command.frame) else {
        return;
    };
    let Some(effective) =
        effective_rect_with_clip_status(command, raster_frame, draw_list.projection_size(), damage)
    else {
        return;
    };
    let mut vertices = rounded_border_vertices(
        command.frame,
        raster_frame,
        border_color,
        draw_list.projection_size(),
        width,
        corner_radius,
    );
    let fill_color = normalized_color(fill_color);
    for vertex in &mut vertices {
        vertex.fill_color = fill_color;
    }
    let vertices = if effective.clipped {
        clip_solid_triangles_to_rect(vertices, effective.rect, draw_list.projection_size())
    } else {
        vertices
    };
    items.push(DrawItem::Solid(SolidItem {
        order,
        rect: effective.rect,
        geometry: SolidGeometry::Vertices(vertices),
        fused_border_command_index: Some(fused_border_command_index),
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
    shape_frame: UiSurfaceRect,
    raster_frame: UiSurfaceRect,
    color: [u8; 4],
    size: (u32, u32),
    corner_radius: f32,
) -> Vec<SolidVertex> {
    let corner_radius = clamped_corner_radius(shape_frame, corner_radius);
    let positions = quad_positions(raster_frame, size);
    let color = normalized_color(color);
    let half_extent = [shape_frame.width * 0.5, shape_frame.height * 0.5];
    let center = [
        shape_frame.x + half_extent[0],
        shape_frame.y + half_extent[1],
    ];
    let raster_right = raster_frame.x + raster_frame.width;
    let raster_bottom = raster_frame.y + raster_frame.height;
    let local_positions = [
        [raster_frame.x - center[0], raster_frame.y - center[1]],
        [raster_right - center[0], raster_frame.y - center[1]],
        [raster_frame.x - center[0], raster_bottom - center[1]],
        [raster_right - center[0], raster_bottom - center[1]],
    ];
    let vertex = |index| SolidVertex {
        position: positions[index],
        color,
        local_position: local_positions[index],
        half_extent,
        corner_radius,
        border_width: 0.0,
        fill_color: [0.0; 4],
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

fn matching_border(
    draw_list: &UiSurfaceDrawList,
    ordered: &[(usize, &UiSurfaceCommand)],
    ordered_index: usize,
    fill: &UiSurfaceCommand,
    fill_color: [u8; 4],
    corner_radius: f32,
) -> Option<(usize, [u8; 4], f32)> {
    if fill_color[3] == 0 {
        return None;
    }
    let &(border_command_index, border) = ordered.get(ordered_index + 1)?;
    if border.frame != fill.frame || border.clip != fill.clip {
        return None;
    }
    match draw_list.resolved_kind(border)? {
        UiSurfaceResolvedCommandKind::Border {
            color,
            width,
            corner_radius: border_radius,
        } if color[3] != 0
            && width.is_finite()
            && width > 0.0
            && border_radius.to_bits() == corner_radius.to_bits() =>
        {
            Some((border_command_index, color, width))
        }
        _ => None,
    }
}

fn rounded_border_vertices(
    shape_frame: UiSurfaceRect,
    raster_frame: UiSurfaceRect,
    color: [u8; 4],
    size: (u32, u32),
    width: f32,
    corner_radius: f32,
) -> Vec<SolidVertex> {
    let width = width.min(shape_frame.width.min(shape_frame.height) * 0.5);
    let mut vertices = solid_vertices(shape_frame, raster_frame, color, size, corner_radius);
    for vertex in &mut vertices {
        vertex.border_width = width;
    }
    vertices
}

fn analytic_raster_frame(frame: UiSurfaceRect) -> Option<UiSurfaceRect> {
    let padding = ANALYTIC_AA_PADDING;
    let raster_frame = UiSurfaceRect::new(
        frame.x - padding,
        frame.y - padding,
        frame.width + padding * 2.0,
        frame.height + padding * 2.0,
    );
    raster_frame
        .has_finite_positive_area()
        .then_some(raster_frame)
}

fn rect_edges_are_physical_pixel_aligned(frame: UiSurfaceRect) -> bool {
    [
        frame.x,
        frame.y,
        frame.x + frame.width,
        frame.y + frame.height,
    ]
    .into_iter()
    .all(|edge| edge.is_finite() && (edge - edge.round()).abs() <= f32::EPSILON)
}

pub(super) fn damage_with_analytic_coverage(
    damage: Option<UiSurfaceRect>,
    surface_size: (u32, u32),
) -> Option<UiSurfaceRect> {
    let damage = analytic_raster_frame(damage?)?;
    let surface = UiSurfaceRect::new(0.0, 0.0, surface_size.0 as f32, surface_size.1 as f32);
    damage.intersection(surface)
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

pub(super) fn command_effective_rect(
    command: &UiSurfaceCommand,
    draw_list: &UiSurfaceDrawList,
) -> Option<UiSurfaceRect> {
    let damage = damage_with_analytic_coverage(draw_list.damage, draw_list.projection_size());
    effective_rect(command, command.frame, draw_list.projection_size(), damage)
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
