use crate::rhi::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfaceImageUvRect, UiSurfaceRect, UiSurfaceTextStyle,
};
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

use super::*;

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
                        atlas_uv: None,
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
    assert!(items.iter().all(|item| item
        .vertices
        .iter()
        .all(|vertex| { vertex.position[0].is_finite() && vertex.position[1].is_finite() })));
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
                    atlas_uv: None,
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
fn wgpu_ui_surface_image_uvs_compose_clipped_rect_with_atlas_uv() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        Some(UiSurfaceRect::new(5.0, 5.0, 10.0, 10.0)),
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
            clip: None,
            kind: UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    width: 64,
                    height: 64,
                    upload_bytes: 0,
                    rgba: None,
                    atlas_uv: Some(UiSurfaceImageUvRect {
                        min: [0.5, 0.25],
                        max: [0.75, 0.5],
                    }),
                },
            },
        }],
    );

    let items = draw_items(&draw_list);

    let DrawItem::Image(image) = &items[0] else {
        panic!("expected clipped atlas image item");
    };
    assert_eq!(image.vertices[0].uv, [0.5625, 0.3125]);
    assert_eq!(image.vertices[5].uv, [0.6875, 0.4375]);
}

#[test]
fn wgpu_ui_surface_skips_image_with_invalid_atlas_uv() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
            clip: None,
            kind: UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    width: 64,
                    height: 64,
                    upload_bytes: 0,
                    rgba: None,
                    atlas_uv: Some(UiSurfaceImageUvRect {
                        min: [0.75, 0.25],
                        max: [0.5, 0.5],
                    }),
                },
            },
        }],
    );

    assert!(draw_items(&draw_list).is_empty());
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
            font_family: None,
            font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
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
            font_family: None,
            font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
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
