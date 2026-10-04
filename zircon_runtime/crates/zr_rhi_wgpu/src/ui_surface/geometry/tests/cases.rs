use zircon_runtime_interface::ui::surface::UiResolvedStyle;
use zr_rhi::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfaceImageUvRect, UiSurfaceRect, UiSurfaceTextStyle,
};

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

fn quad(z_index: i32, frame: UiSurfaceRect, color: [u8; 4]) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Quad {
            color,
            corner_radius: 0.0,
        },
    }
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

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].vertices().len(), 6);
    assert!(items[0]
        .vertices()
        .iter()
        .all(|vertex| vertex.corner_radius == 0.0 && vertex.border_width == 2.0));
}

#[test]
fn wgpu_ui_surface_fuses_matching_fill_and_border_into_one_analytic_item() {
    let frame = UiSurfaceRect::new(10.25, 12.5, 24.0, 20.0);
    let fill = [32, 40, 52, 224];
    let border = [96, 174, 255, 255];
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            UiSurfaceCommand {
                z_index: 7,
                frame,
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: fill,
                    corner_radius: 8.0,
                },
            },
            UiSurfaceCommand {
                z_index: 8,
                frame,
                clip: None,
                kind: UiSurfaceCommandKind::Border {
                    color: border,
                    width: 1.0,
                    corner_radius: 8.0,
                },
            },
        ],
    );

    let (items, stats) = draw_items_with_stats(&draw_list);
    let [DrawItem::Solid(item)] = items.as_slice() else {
        panic!("matching fill and border must become one solid item");
    };

    assert_eq!(stats.visible_command_count, 2);
    assert_eq!(stats.visible_draw_item_count, 1);
    assert_eq!(stats.draw_calls, 1);
    assert!(item.vertices().iter().all(|vertex| {
        vertex.color == normalized_color(border)
            && vertex.fill_color == normalized_color(fill)
            && vertex.corner_radius == 8.0
            && vertex.border_width == 1.0
    }));
}

#[test]
fn wgpu_ui_surface_only_fuses_fill_and_border_when_they_are_draw_order_neighbors() {
    let frame = UiSurfaceRect::new(10.25, 12.5, 24.0, 20.0);
    let fill = UiSurfaceCommand {
        z_index: 7,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Quad {
            color: [32, 40, 52, 224],
            corner_radius: 8.0,
        },
    };
    let border = UiSurfaceCommand {
        z_index: 9,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Border {
            color: [96, 174, 255, 255],
            width: 1.0,
            corner_radius: 8.0,
        },
    };
    let intervening = UiSurfaceCommand {
        z_index: 8,
        frame: UiSurfaceRect::new(12.0, 14.0, 4.0, 4.0),
        clip: None,
        kind: UiSurfaceCommandKind::Quad {
            color: [255, 255, 255, 255],
            corner_radius: 0.0,
        },
    };

    let adjacent = UiSurfaceDrawList::new((100, 100), None, vec![fill.clone(), border.clone()]);
    assert_eq!(
        solid_items(&adjacent).len(),
        1,
        "matching draw-order neighbors may fuse across distinct z values"
    );

    let separated = UiSurfaceDrawList::new((100, 100), None, vec![fill, border, intervening]);
    assert_eq!(
        solid_items(&separated).len(),
        3,
        "fusion must never cross an intervening draw item"
    );
}

#[test]
fn wgpu_ui_surface_does_not_fuse_invalid_border_widths_into_a_fill() {
    for width in [0.0, -0.5, f32::NAN, f32::INFINITY] {
        let frame = UiSurfaceRect::new(10.25, 12.5, 24.0, 20.0);
        let fill = [32, 40, 52, 224];
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            None,
            vec![
                UiSurfaceCommand {
                    z_index: 7,
                    frame,
                    clip: None,
                    kind: UiSurfaceCommandKind::Quad {
                        color: fill,
                        corner_radius: 8.0,
                    },
                },
                UiSurfaceCommand {
                    z_index: 8,
                    frame,
                    clip: None,
                    kind: UiSurfaceCommandKind::Border {
                        color: [96, 174, 255, 255],
                        width,
                        corner_radius: 8.0,
                    },
                },
            ],
        );

        let items = solid_items(&draw_list);
        let [item] = items.as_slice() else {
            panic!("invalid border width {width:?} must leave only the original fill");
        };
        assert!(item.vertices().iter().all(|vertex| {
            vertex.color == normalized_color(fill)
                && vertex.fill_color == [0.0; 4]
                && vertex.border_width == 0.0
        }));
    }
}

#[test]
fn wgpu_ui_surface_only_fuses_identical_fill_and_border_geometry() {
    let frame = UiSurfaceRect::new(10.25, 12.5, 24.0, 20.0);
    let fill = UiSurfaceCommand {
        z_index: 7,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Quad {
            color: [32, 40, 52, 224],
            corner_radius: 8.0,
        },
    };
    let mismatches = [
        (
            "frame",
            UiSurfaceCommand {
                z_index: 8,
                frame: UiSurfaceRect::new(10.5, 12.5, 24.0, 20.0),
                clip: None,
                kind: UiSurfaceCommandKind::Border {
                    color: [96, 174, 255, 255],
                    width: 1.0,
                    corner_radius: 8.0,
                },
            },
        ),
        (
            "clip",
            UiSurfaceCommand {
                z_index: 8,
                frame,
                clip: Some(UiSurfaceRect::new(10.25, 12.5, 20.0, 20.0)),
                kind: UiSurfaceCommandKind::Border {
                    color: [96, 174, 255, 255],
                    width: 1.0,
                    corner_radius: 8.0,
                },
            },
        ),
        (
            "radius",
            UiSurfaceCommand {
                z_index: 8,
                frame,
                clip: None,
                kind: UiSurfaceCommandKind::Border {
                    color: [96, 174, 255, 255],
                    width: 1.0,
                    corner_radius: 7.0,
                },
            },
        ),
    ];

    for (label, border) in mismatches {
        let draw_list = UiSurfaceDrawList::new((100, 100), None, vec![fill.clone(), border]);
        let items = solid_items(&draw_list);

        assert_eq!(
            items.len(),
            2,
            "a {label} mismatch must preserve separate source items"
        );
        assert!(items
            .iter()
            .flat_map(SolidItem::vertices)
            .all(|vertex| vertex.fill_color == [0.0; 4]));
    }
}

#[test]
fn wgpu_ui_surface_fractional_square_quad_uses_analytic_coverage() {
    let frame = UiSurfaceRect::new(10.25, 12.5, 20.0, 18.25);
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame,
            clip: None,
            kind: UiSurfaceCommandKind::Quad {
                color: [255, 255, 255, 255],
                corner_radius: 0.0,
            },
        }],
    );

    let items = solid_items(&draw_list);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].rect, UiSurfaceRect::new(9.25, 11.5, 22.0, 20.25));
    assert_eq!(items[0].vertices().len(), 6);
    assert!(items[0].vertices().iter().all(|vertex| {
        vertex.half_extent == [10.0, 9.125]
            && vertex.corner_radius == 0.0
            && vertex.border_width == 0.0
    }));
}

#[test]
fn wgpu_ui_surface_square_border_preserves_subpixel_physical_width() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(10.25, 12.5, 24.0, 20.0),
            clip: None,
            kind: UiSurfaceCommandKind::Border {
                color: [255, 255, 255, 255],
                width: 0.625,
                corner_radius: 0.0,
            },
        }],
    );

    let items = solid_items(&draw_list);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].vertices().len(), 6);
    assert!(items[0]
        .vertices()
        .iter()
        .all(|vertex| vertex.corner_radius == 0.0 && vertex.border_width == 0.625));
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
    assert_eq!(items[0].rect, UiSurfaceRect::new(24.0, 24.0, 16.0, 16.0));
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
                        resource_generation: 0,
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
    assert_eq!(
        first_solid.instance().expect("plain quad instance").color,
        [20.0 / 255.0, 0.0, 0.0, 1.0]
    );
    assert_eq!(
        second_solid.instance().expect("plain quad instance").color,
        [30.0 / 255.0, 0.0, 0.0, 1.0]
    );
}

#[test]
fn target_only_resize_keeps_projection_geometry_and_offscreen_items() {
    let mut draw_list = UiSurfaceDrawList::with_generation(
        (100, 100),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(60.0, 60.0, 20.0, 20.0),
            clip: None,
            kind: UiSurfaceCommandKind::Quad {
                color: [255, 255, 255, 255],
                corner_radius: 0.0,
            },
        }],
        7,
    );
    let before = solid_items(&draw_list)
        .pop()
        .and_then(|item| item.instance())
        .expect("projection contains the quad");

    draw_list.retarget_surface_size_preserving_projection((50, 50));
    let after = solid_items(&draw_list)
        .pop()
        .and_then(|item| item.instance())
        .expect("target clipping is deferred to the render scissor");

    assert_eq!(after.min_position, before.min_position);
    assert_eq!(after.max_position, before.max_position);
    assert_eq!(after.color, before.color);
}

#[test]
fn wgpu_ui_surface_generates_analytic_rounded_quads_for_fill_and_border() {
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
    assert_eq!(items[0].vertices().len(), 6);
    assert_eq!(items[1].vertices().len(), 6);
    assert!(items[0].vertices().iter().all(|vertex| {
        vertex.half_extent == [10.0, 10.0]
            && vertex.corner_radius == 8.0
            && vertex.border_width == 0.0
    }));
    assert!(items[1].vertices().iter().all(|vertex| {
        vertex.half_extent == [10.0, 10.0]
            && vertex.corner_radius == 8.0
            && vertex.border_width == 2.0
    }));
    assert_eq!(items[0].vertices()[0].local_position, [-11.0, -11.0]);
    assert_eq!(items[0].vertices()[5].local_position, [11.0, 11.0]);
    assert!(items.iter().all(|item| {
        item.vertices()
            .iter()
            .all(|vertex| vertex.position[0].is_finite() && vertex.position[1].is_finite())
    }));
}

#[test]
fn wgpu_ui_surface_analytic_border_preserves_subpixel_physical_width() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(10.0, 10.0, 24.0, 20.0),
            clip: None,
            kind: UiSurfaceCommandKind::Border {
                color: [255, 255, 255, 255],
                width: 0.625,
                corner_radius: 8.0,
            },
        }],
    );

    let items = solid_items(&draw_list);

    assert_eq!(items.len(), 1);
    assert!(items[0]
        .vertices()
        .iter()
        .all(|vertex| vertex.border_width == 0.625));
}

#[test]
fn wgpu_ui_surface_analytic_border_rejects_nonpositive_or_nonfinite_width() {
    for width in [0.0, -0.5, f32::NAN, f32::INFINITY] {
        let draw_list = UiSurfaceDrawList::new(
            (100, 100),
            None,
            vec![UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(10.0, 10.0, 24.0, 20.0),
                clip: None,
                kind: UiSurfaceCommandKind::Border {
                    color: [255, 255, 255, 255],
                    width,
                    corner_radius: 8.0,
                },
            }],
        );

        assert!(
            solid_items(&draw_list).is_empty(),
            "invalid analytic border width {width:?} must not become a rounded fill"
        );
    }
}

#[test]
fn wgpu_ui_surface_analytic_quad_bounds_include_the_outer_coverage_fringe() {
    let frame = UiSurfaceRect::new(10.75, 12.25, 20.0, 18.0);
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame,
            clip: None,
            kind: UiSurfaceCommandKind::Quad {
                color: [255, 255, 255, 255],
                corner_radius: 8.0,
            },
        }],
    );

    let items = solid_items(&draw_list);

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].rect, UiSurfaceRect::new(9.75, 11.25, 22.0, 20.0));
    assert_eq!(items[0].vertices()[0].local_position, [-11.0, -10.0]);
    assert_eq!(items[0].vertices()[5].local_position, [11.0, 10.0]);
    assert!(items[0].vertices().iter().all(|vertex| {
        vertex.half_extent == [10.0, 9.0]
            && vertex.corner_radius == 8.0
            && vertex.border_width == 0.0
    }));
}

#[test]
fn wgpu_ui_surface_damage_redraws_content_beneath_the_analytic_fringe() {
    let rounded_frame = UiSurfaceRect::new(10.75, 12.25, 20.0, 18.0);
    let background_frame = UiSurfaceRect::new(10.0, 14.0, 0.5, 4.0);
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        Some(rounded_frame),
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: background_frame,
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [20, 24, 28, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: rounded_frame,
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [255, 255, 255, 255],
                    corner_radius: 8.0,
                },
            },
        ],
    );

    let items = solid_items(&draw_list);

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].rect, UiSurfaceRect::new(9.75, 13.0, 1.75, 6.0));
    assert!(items[0]
        .vertices()
        .iter()
        .all(|vertex| vertex.half_extent == [0.25, 2.0]));
    assert_eq!(items[1].rect, UiSurfaceRect::new(9.75, 11.25, 22.0, 20.0));
}

#[test]
fn wgpu_ui_surface_clip_does_not_rebuild_rounded_geometry_at_the_clip_edge() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 20.0, 20.0),
                clip: Some(UiSurfaceRect::new(10.0, 0.0, 10.0, 20.0)),
                kind: UiSurfaceCommandKind::Quad {
                    color: [255, 255, 255, 255],
                    corner_radius: 8.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(30.0, 0.0, 20.0, 20.0),
                clip: Some(UiSurfaceRect::new(40.0, 0.0, 10.0, 20.0)),
                kind: UiSurfaceCommandKind::Border {
                    color: [255, 0, 0, 255],
                    width: 2.0,
                    corner_radius: 8.0,
                },
            },
        ],
    );

    let items = draw_items(&draw_list);
    let DrawItem::Solid(solid) = &items[0] else {
        panic!("expected clipped rounded solid item");
    };

    assert_eq!(solid.rect, UiSurfaceRect::new(10.0, 0.0, 10.0, 20.0));
    assert!(solid
        .vertices()
        .iter()
        .all(|vertex| vertex.position[0] >= -0.800_001));
    assert!(solid
        .vertices()
        .iter()
        .any(|vertex| (vertex.position[0] + 0.8).abs() < 0.000_001
            && (vertex.position[1] - 1.0).abs() < 0.000_001));
    assert!(solid
        .vertices()
        .iter()
        .any(|vertex| vertex.local_position[0].abs() < 0.000_01
            && (vertex.local_position[1] + 10.0).abs() < 0.000_01));
    assert!(solid.vertices().iter().all(|vertex| {
        vertex.half_extent == [10.0, 10.0]
            && vertex.corner_radius == 8.0
            && vertex.border_width == 0.0
    }));

    let DrawItem::Solid(border) = &items[1] else {
        panic!("expected clipped rounded border item");
    };
    assert_eq!(border.rect, UiSurfaceRect::new(40.0, 0.0, 10.0, 20.0));
    assert!(border
        .vertices()
        .iter()
        .all(|vertex| vertex.position[0] >= -0.200_001));
    assert!(border
        .vertices()
        .iter()
        .any(|vertex| (vertex.position[0] + 0.2).abs() < 0.000_001
            && (vertex.position[1] - 1.0).abs() < 0.000_001));
    assert!(border
        .vertices()
        .iter()
        .any(|vertex| vertex.local_position[0].abs() < 0.000_01
            && (vertex.local_position[1] + 10.0).abs() < 0.000_01));
    assert!(border.vertices().iter().all(|vertex| {
        vertex.half_extent == [10.0, 10.0]
            && vertex.corner_radius == 8.0
            && vertex.border_width == 2.0
    }));
}

#[test]
fn wgpu_ui_surface_skips_non_finite_geometry() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![quad(
            0,
            UiSurfaceRect::new(f32::NAN, 0.0, 20.0, 20.0),
            [255, 255, 255, 255],
        )],
    );

    assert!(draw_items(&draw_list).is_empty());
}

#[test]
fn wgpu_ui_surface_skips_finite_rects_with_non_finite_endpoints() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![quad(
            0,
            UiSurfaceRect::new(f32::MAX, 0.0, f32::MAX, 20.0),
            [255, 255, 255, 255],
        )],
    );

    assert!(draw_items(&draw_list).is_empty());
}

#[test]
fn wgpu_ui_surface_fractional_rounded_rect_uses_the_unclipped_geometry_path() {
    let frame = UiSurfaceRect::new(4.25, 4.5, 19.5, 18.75);
    let command = UiSurfaceCommand {
        z_index: 0,
        frame,
        clip: Some(UiSurfaceRect::new(0.0, 0.0, 30.0, 30.0)),
        kind: UiSurfaceCommandKind::Quad {
            color: [255, 255, 255, 255],
            corner_radius: 6.0,
        },
    };
    let draw_list = UiSurfaceDrawList::new((100, 100), None, vec![command.clone()]);
    let raster_frame = analytic_raster_frame(frame).expect("analytic raster frame");
    let effective = effective_rect_with_clip_status(&command, raster_frame, (100, 100), None)
        .expect("fractional rounded rect remains visible");
    let expected = solid_vertices(frame, raster_frame, [255, 255, 255, 255], (100, 100), 6.0);
    let items = solid_items(&draw_list);

    assert!(!effective.clipped);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].vertices(), expected);
}

#[test]
fn wgpu_ui_surface_clipped_analytic_quads_remain_finite_non_degenerate_and_consistent() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        Some(UiSurfaceRect::new(9.25, 2.5, 8.5, 13.75)),
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(0.5, 0.5, 20.0, 20.0),
            clip: None,
            kind: UiSurfaceCommandKind::Quad {
                color: [255, 255, 255, 255],
                corner_radius: 8.0,
            },
        }],
    );
    let items = solid_items(&draw_list);
    let triangles = items[0].vertices().chunks_exact(3).collect::<Vec<_>>();
    let signed_areas = triangles
        .iter()
        .map(|triangle| {
            let first = triangle[0].position;
            let second = triangle[1].position;
            let third = triangle[2].position;
            (second[0] - first[0]) * (third[1] - first[1])
                - (second[1] - first[1]) * (third[0] - first[0])
        })
        .collect::<Vec<_>>();

    assert!(!signed_areas.is_empty());
    assert!(triangles
        .iter()
        .flat_map(|triangle| triangle.iter())
        .all(|vertex| {
            vertex.position.into_iter().all(f32::is_finite)
                && vertex.color.into_iter().all(f32::is_finite)
                && vertex.local_position.into_iter().all(f32::is_finite)
                && vertex.half_extent.into_iter().all(f32::is_finite)
                && vertex.corner_radius.is_finite()
                && vertex.border_width.is_finite()
        }));
    assert!(signed_areas.iter().all(|area| area.abs() > 1.0e-10));
    let winding = signed_areas[0].is_sign_positive();
    assert!(signed_areas
        .iter()
        .all(|area| area.is_sign_positive() == winding));
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
                    resource_generation: 0,
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
    assert_eq!(image.rect, UiSurfaceRect::new(4.0, 4.0, 12.0, 12.0));
    assert_eq!(image.vertices[0].uv, [0.2, 0.2]);
    assert_eq!(image.vertices[5].uv, [0.8, 0.8]);
}

#[test]
fn wgpu_ui_surface_image_uvs_preserve_subpixel_frame_proportions() {
    let draw_list = UiSurfaceDrawList::new(
        (1, 1),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(-0.25, -0.25, 0.5, 0.5),
            clip: None,
            kind: UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: "subpixel-image".to_string(),
                    resource_generation: 0,
                    width: 1,
                    height: 1,
                    upload_bytes: 4,
                    rgba: Some(vec![255; 4]),
                    atlas_uv: None,
                },
            },
        }],
    );

    let items = draw_items(&draw_list);

    let DrawItem::Image(image) = &items[0] else {
        panic!("expected clipped subpixel image item");
    };
    assert_eq!(image.rect, UiSurfaceRect::new(0.0, 0.0, 0.25, 0.25));
    assert_eq!(image.vertices[0].uv, [0.5, 0.5]);
    assert_eq!(image.vertices[5].uv, [1.0, 1.0]);
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
                    resource_generation: 0,
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
    assert!(image.vertices[0]
        .uv
        .into_iter()
        .zip([0.55, 0.3])
        .all(|(actual, expected)| (actual - expected).abs() <= f32::EPSILON));
    assert!(image.vertices[5]
        .uv
        .into_iter()
        .zip([0.7, 0.45])
        .all(|(actual, expected)| (actual - expected).abs() <= f32::EPSILON));
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
                    resource_generation: 0,
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

    assert_eq!(clip, UiSurfaceRect::new(24.0, 24.0, 16.0, 16.0));
    assert_eq!(bounds.left, 24);
    assert_eq!(bounds.top, 24);
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
