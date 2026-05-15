use bytemuck::{Pod, Zeroable};
use glyphon::TextBounds;

use crate::rhi::{UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceRect};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct SolidVertex {
    pub(super) position: [f32; 2],
    pub(super) color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
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
    pub(super) vertices: Vec<SolidVertex>,
}

#[derive(Clone, Debug)]
pub(super) struct ImageItem {
    pub(super) order: DrawItemOrder,
    pub(super) rect: UiSurfaceRect,
    pub(super) resource_key: String,
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
    let mut items = Vec::new();
    for (command_index, command) in ordered_commands(draw_list) {
        match &command.kind {
            UiSurfaceCommandKind::Quad {
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
                    *color,
                    *corner_radius,
                    command,
                    draw_list,
                );
            }
            UiSurfaceCommandKind::Border {
                color,
                width,
                corner_radius,
            } => {
                if *corner_radius > 0.0 {
                    push_rounded_border_item(
                        &mut items,
                        DrawItemOrder {
                            z_index: command.z_index,
                            command_index,
                            sub_index: 0,
                        },
                        *color,
                        *width,
                        *corner_radius,
                        command,
                        draw_list,
                    );
                } else {
                    for (sub_index, rect) in
                        border_rects(command.frame, *width).into_iter().enumerate()
                    {
                        push_solid_item(
                            &mut items,
                            DrawItemOrder {
                                z_index: command.z_index,
                                command_index,
                                sub_index,
                            },
                            rect,
                            *color,
                            0.0,
                            command,
                            draw_list,
                        );
                    }
                }
            }
            UiSurfaceCommandKind::Image { payload } => {
                let Some(rect) = primitive_effective_rect(command, command.frame, draw_list) else {
                    continue;
                };
                items.push(DrawItem::Image(ImageItem {
                    order: DrawItemOrder {
                        z_index: command.z_index,
                        command_index,
                        sub_index: 0,
                    },
                    rect,
                    resource_key: payload.resource_key.clone(),
                    vertices: image_vertices(command.frame, rect, draw_list.surface_size),
                }));
            }
            UiSurfaceCommandKind::Text { .. } => {
                let Some(rect) = command_effective_rect(command, draw_list) else {
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
            _ => {}
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
) {
    let Some(rect) = primitive_effective_rect(command, frame, draw_list) else {
        return;
    };
    items.push(DrawItem::Solid(SolidItem {
        order,
        rect,
        vertices: solid_vertices(rect, color, draw_list.surface_size, corner_radius),
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
) {
    let Some(rect) = primitive_effective_rect(command, command.frame, draw_list) else {
        return;
    };
    items.push(DrawItem::Solid(SolidItem {
        order,
        rect,
        vertices: rounded_border_vertices(
            rect,
            color,
            draw_list.surface_size,
            width,
            corner_radius,
        ),
    }));
}

#[cfg(test)]
fn solid_items(draw_list: &UiSurfaceDrawList) -> Vec<SolidItem> {
    draw_items(draw_list)
        .into_iter()
        .filter_map(|item| match item {
            DrawItem::Solid(item) => Some(item),
            DrawItem::Image(_) => None,
            DrawItem::Text(_) => None,
        })
        .collect()
}

fn solid_vertices(
    frame: UiSurfaceRect,
    color: [u8; 4],
    size: (u32, u32),
    corner_radius: f32,
) -> Vec<SolidVertex> {
    let corner_radius = clamped_corner_radius(frame, corner_radius);
    if corner_radius > 0.0 {
        return rounded_rect_vertices(frame, color, size, corner_radius);
    }
    let positions = quad_positions(frame, size);
    let color = normalized_color(color);
    vec![
        SolidVertex {
            position: positions[0],
            color,
        },
        SolidVertex {
            position: positions[1],
            color,
        },
        SolidVertex {
            position: positions[2],
            color,
        },
        SolidVertex {
            position: positions[2],
            color,
        },
        SolidVertex {
            position: positions[1],
            color,
        },
        SolidVertex {
            position: positions[3],
            color,
        },
    ]
}

fn rounded_rect_vertices(
    frame: UiSurfaceRect,
    color: [u8; 4],
    size: (u32, u32),
    corner_radius: f32,
) -> Vec<SolidVertex> {
    let points = rounded_rect_points(frame, corner_radius);
    let center = ndc_position(
        frame.x + frame.width * 0.5,
        frame.y + frame.height * 0.5,
        size,
    );
    let color = normalized_color(color);
    let mut vertices = Vec::with_capacity(points.len() * 3);
    for index in 0..points.len() {
        let next = (index + 1) % points.len();
        vertices.push(SolidVertex {
            position: center,
            color,
        });
        vertices.push(SolidVertex {
            position: ndc_position(points[index].0, points[index].1, size),
            color,
        });
        vertices.push(SolidVertex {
            position: ndc_position(points[next].0, points[next].1, size),
            color,
        });
    }
    vertices
}

fn rounded_border_vertices(
    frame: UiSurfaceRect,
    color: [u8; 4],
    size: (u32, u32),
    width: f32,
    corner_radius: f32,
) -> Vec<SolidVertex> {
    let width = width.max(1.0).min(frame.width.min(frame.height) * 0.5);
    let inner = UiSurfaceRect::new(
        frame.x + width,
        frame.y + width,
        (frame.width - width * 2.0).max(0.0),
        (frame.height - width * 2.0).max(0.0),
    );
    if inner.width <= 0.0 || inner.height <= 0.0 {
        return rounded_rect_vertices(frame, color, size, corner_radius);
    }
    let outer_points = rounded_rect_points(frame, clamped_corner_radius(frame, corner_radius));
    let inner_points = rounded_rect_points(
        inner,
        clamped_corner_radius(inner, (corner_radius - width).max(0.0)),
    );
    let color = normalized_color(color);
    let mut vertices = Vec::with_capacity(outer_points.len() * 6);
    for index in 0..outer_points.len() {
        let next = (index + 1) % outer_points.len();
        let outer_a = ndc_position(outer_points[index].0, outer_points[index].1, size);
        let outer_b = ndc_position(outer_points[next].0, outer_points[next].1, size);
        let inner_a = ndc_position(inner_points[index].0, inner_points[index].1, size);
        let inner_b = ndc_position(inner_points[next].0, inner_points[next].1, size);
        vertices.extend([
            SolidVertex {
                position: outer_a,
                color,
            },
            SolidVertex {
                position: outer_b,
                color,
            },
            SolidVertex {
                position: inner_a,
                color,
            },
            SolidVertex {
                position: inner_a,
                color,
            },
            SolidVertex {
                position: outer_b,
                color,
            },
            SolidVertex {
                position: inner_b,
                color,
            },
        ]);
    }
    vertices
}

fn rounded_rect_points(frame: UiSurfaceRect, corner_radius: f32) -> Vec<(f32, f32)> {
    const SEGMENTS: usize = 6;
    let radius = clamped_corner_radius(frame, corner_radius);
    let left = frame.x;
    let top = frame.y;
    let right = frame.x + frame.width;
    let bottom = frame.y + frame.height;
    let centers = [
        (right - radius, top + radius, -90.0_f32, 0.0_f32),
        (right - radius, bottom - radius, 0.0_f32, 90.0_f32),
        (left + radius, bottom - radius, 90.0_f32, 180.0_f32),
        (left + radius, top + radius, 180.0_f32, 270.0_f32),
    ];
    let mut points = Vec::with_capacity(SEGMENTS * centers.len() + centers.len());
    for (cx, cy, start, end) in centers {
        for step in 0..=SEGMENTS {
            let t = step as f32 / SEGMENTS as f32;
            let angle = (start + (end - start) * t).to_radians();
            points.push((cx + radius * angle.cos(), cy + radius * angle.sin()));
        }
    }
    points
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

fn ndc_position(x: f32, y: f32, size: (u32, u32)) -> [f32; 2] {
    let width = size.0.max(1) as f32;
    let height = size.1.max(1) as f32;
    [(x / width) * 2.0 - 1.0, 1.0 - (y / height) * 2.0]
}

fn image_vertices(
    frame: UiSurfaceRect,
    visible_rect: UiSurfaceRect,
    size: (u32, u32),
) -> [ImageVertex; 6] {
    let positions = quad_positions(visible_rect, size);
    let u0 = ((visible_rect.x - frame.x) / frame.width.max(1.0)).clamp(0.0, 1.0);
    let v0 = ((visible_rect.y - frame.y) / frame.height.max(1.0)).clamp(0.0, 1.0);
    let u1 =
        ((visible_rect.x + visible_rect.width - frame.x) / frame.width.max(1.0)).clamp(0.0, 1.0);
    let v1 =
        ((visible_rect.y + visible_rect.height - frame.y) / frame.height.max(1.0)).clamp(0.0, 1.0);
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
    primitive_effective_rect(command, command.frame, draw_list)
}

fn primitive_effective_rect(
    command: &UiSurfaceCommand,
    primitive_frame: UiSurfaceRect,
    draw_list: &UiSurfaceDrawList,
) -> Option<UiSurfaceRect> {
    let surface = UiSurfaceRect::new(
        0.0,
        0.0,
        draw_list.surface_size.0 as f32,
        draw_list.surface_size.1 as f32,
    );
    let mut rect = primitive_frame.intersection(surface)?;
    if let Some(clip) = command.clip {
        rect = rect.intersection(clip)?;
    }
    if let Some(damage) = draw_list.damage {
        rect = rect.intersection(damage)?;
    }
    Some(rect)
}

trait RectExt {
    fn intersection(self, other: UiSurfaceRect) -> Option<UiSurfaceRect>;
}

impl RectExt for UiSurfaceRect {
    fn intersection(self, other: UiSurfaceRect) -> Option<UiSurfaceRect> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        (right > left && bottom > top)
            .then(|| UiSurfaceRect::new(left, top, right - left, bottom - top))
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
mod tests {
    use crate::rhi::{
        UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
        UiSurfaceRect, UiSurfaceTextStyle,
    };

    use super::*;

    #[test]
    fn wgpu_ui_surface_generates_border_items_inside_damage() {
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            Some(UiSurfaceRect::new(0.0, 0.0, 50.0, 50.0)),
            vec![UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0),
                clip: None,
                kind: UiSurfaceCommandKind::Border {
                    color: [255, 0, 0, 255],
                    width: 2.0,
                    corner_radius: 0.0,
                },
            }],
        );

        let items = solid_items(&draw_list);

        assert_eq!(items.len(), 4);
        assert!(items.iter().all(|item| item.rect.width > 0.0));
        assert!(items.iter().all(|item| item.rect.height > 0.0));
    }

    #[test]
    fn wgpu_ui_surface_damage_and_clip_trim_solid_item_geometry() {
        let command = UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(10.0, 10.0, 30.0, 30.0),
            clip: Some(UiSurfaceRect::new(20.0, 20.0, 20.0, 20.0)),
            kind: UiSurfaceCommandKind::Quad {
                color: [255, 255, 255, 255],
                corner_radius: 0.0,
            },
        };
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            Some(UiSurfaceRect::new(25.0, 25.0, 50.0, 50.0)),
            vec![command.clone()],
        );

        let items = solid_items(&draw_list);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].rect, UiSurfaceRect::new(25.0, 25.0, 15.0, 15.0));
    }

    #[test]
    fn wgpu_ui_surface_draw_items_sort_by_stable_z_order() {
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            None,
            vec![
                UiSurfaceCommand {
                    z_index: 20,
                    frame: UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [20, 0, 0, 255],
                        corner_radius: 0.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 10,
                    frame: UiSurfaceRect::new(10.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Image {
                        payload: UiSurfaceImagePayload {
                            resource_key: "viewport".to_string(),
                            width: 2,
                            height: 2,
                            upload_bytes: 16,
                            rgba: Some(vec![255; 16]),
                        },
                    },
                },
                UiSurfaceCommand {
                    z_index: 20,
                    frame: UiSurfaceRect::new(30.0, 0.0, 10.0, 10.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [30, 0, 0, 255],
                        corner_radius: 0.0,
                    },
                },
            ],
        );

        let items = draw_items(&draw_list);

        assert!(matches!(items[0], DrawItem::Image(_)));
        let DrawItem::Solid(first_solid) = &items[1] else {
            panic!("expected first z=20 command to remain before second z=20 command");
        };
        let DrawItem::Solid(second_solid) = &items[2] else {
            panic!("expected second z=20 command to remain after first z=20 command");
        };
        assert_eq!(first_solid.vertices[0].color, [20.0 / 255.0, 0.0, 0.0, 1.0]);
        assert_eq!(
            second_solid.vertices[0].color,
            [30.0 / 255.0, 0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn wgpu_ui_surface_generates_rounded_solid_vertices_for_quad_and_border() {
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            None,
            vec![
                UiSurfaceCommand {
                    z_index: 0,
                    frame: UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: [255, 255, 255, 255],
                        corner_radius: 8.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 1,
                    frame: UiSurfaceRect::new(40.0, 10.0, 20.0, 20.0),
                    clip: None,
                    kind: UiSurfaceCommandKind::Border {
                        color: [255, 0, 0, 255],
                        width: 2.0,
                        corner_radius: 8.0,
                    },
                },
            ],
        );

        let items = solid_items(&draw_list);

        assert_eq!(items.len(), 2);
        assert!(items[0].vertices.len() > 6);
        assert!(items[1].vertices.len() > 6);
        assert!(items
            .iter()
            .all(|item| item.vertices.iter().all(|vertex| {
                vertex.position[0].is_finite() && vertex.position[1].is_finite()
            })));
    }

    #[test]
    fn wgpu_ui_surface_image_uvs_follow_clipped_rect() {
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            Some(UiSurfaceRect::new(5.0, 5.0, 10.0, 10.0)),
            vec![UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                clip: None,
                kind: UiSurfaceCommandKind::Image {
                    payload: UiSurfaceImagePayload {
                        resource_key: "viewport".to_string(),
                        width: 2,
                        height: 2,
                        upload_bytes: 16,
                        rgba: Some(vec![255; 16]),
                    },
                },
            }],
        );

        let items = draw_items(&draw_list);

        let DrawItem::Image(image) = &items[0] else {
            panic!("expected clipped image item");
        };
        assert_eq!(image.rect, UiSurfaceRect::new(5.0, 5.0, 10.0, 10.0));
        assert_eq!(image.vertices[0].uv, [0.25, 0.25]);
        assert_eq!(image.vertices[5].uv, [0.75, 0.75]);
    }

    #[test]
    fn wgpu_ui_surface_text_bounds_clip_to_damage_and_command_clip() {
        let command = UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(10.0, 10.0, 30.0, 30.0),
            clip: Some(UiSurfaceRect::new(20.0, 20.0, 20.0, 20.0)),
            kind: UiSurfaceCommandKind::Text {
                text: "Status".to_string(),
                color: [255, 255, 255, 255],
                font_size: 12.0,
                line_height: 14.0,
                style: UiSurfaceTextStyle::Regular,
            },
        };
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            Some(UiSurfaceRect::new(25.0, 25.0, 50.0, 50.0)),
            vec![command.clone()],
        );

        let clip = command_effective_rect(&command, &draw_list).unwrap();
        let bounds = text_bounds_from_rect(clip);

        assert_eq!(clip, UiSurfaceRect::new(25.0, 25.0, 15.0, 15.0));
        assert_eq!(bounds.left, 25);
        assert_eq!(bounds.top, 25);
        assert_eq!(bounds.right, 40);
        assert_eq!(bounds.bottom, 40);
    }

    #[test]
    fn wgpu_ui_surface_text_skips_disjoint_damage() {
        let command = UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0),
            clip: Some(UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0)),
            kind: UiSurfaceCommandKind::Text {
                text: "Hidden".to_string(),
                color: [255, 255, 255, 255],
                font_size: 12.0,
                line_height: 14.0,
                style: UiSurfaceTextStyle::Regular,
            },
        };
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            Some(UiSurfaceRect::new(50.0, 50.0, 10.0, 10.0)),
            vec![command.clone()],
        );

        assert_eq!(command_effective_rect(&command, &draw_list), None);
    }
}
