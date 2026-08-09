use super::*;

#[test]
fn gpu_timing_is_explicit_for_ui_surface_descriptors() {
    let descriptor = UiSurfaceDescriptor::headless("ui-profile", 32, 16);

    assert!(!descriptor.allow_gpu_timing);
    assert!(descriptor.with_gpu_timing().allow_gpu_timing);
}

#[test]
fn draw_list_stats_count_draw_upload_and_clip_commands() {
    let draw_list = UiSurfaceDrawList::new(
        (64, 32),
        Some(UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0)),
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                clip: None,
                kind: UiSurfaceCommandKind::Clip,
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [1, 2, 3, 255],
                    corner_radius: 6.0,
                },
            },
            UiSurfaceCommand {
                z_index: 2,
                frame: UiSurfaceRect::new(1.0, 1.0, 8.0, 8.0),
                clip: None,
                kind: UiSurfaceCommandKind::Border {
                    color: [4, 5, 6, 255],
                    width: 1.0,
                    corner_radius: 6.0,
                },
            },
            UiSurfaceCommand {
                z_index: 3,
                frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
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
        ],
    );

    let stats = draw_list.stats();
    assert_eq!(stats.surface_size, (64, 32));
    assert_eq!(stats.draw_calls, 3);
    assert_eq!(stats.render_pass_count, 0);
    assert_eq!(stats.retained_cache_copy_bytes, 0);
    assert_eq!(stats.visible_command_count, 3);
    assert_eq!(stats.visible_draw_item_count, 3);
    assert_eq!(stats.image_count, 1);
    assert_eq!(stats.image_upload_bytes, 16);
    assert_eq!(stats.clip_count, 1);
}

#[test]
fn compact_draw_list_uses_external_image_resources_when_commands_are_handle_only() {
    let mut image_resources = UiSurfaceImageResourceTable::default();
    image_resources.insert(
        "atlas://editor/icons".to_string(),
        UiSurfaceImageResource {
            generation: 7,
            width: 2,
            height: 2,
            upload_bytes: 16,
            rgba: vec![9; 16],
        },
    );
    let draw_list = UiSurfaceDrawList::with_generation_and_compact_styles_and_image_resources(
        (64, 32),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
            clip: None,
            kind: UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: "atlas://editor/icons".to_string(),
                    resource_generation: 7,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: None,
                    atlas_uv: None,
                },
            },
        }],
        19,
        image_resources,
    );

    assert_eq!(
        draw_list
            .image_resource("atlas://editor/icons", 7)
            .expect("external image resource")
            .rgba,
        vec![9; 16]
    );
    assert_eq!(draw_list.stats().image_upload_bytes, 16);
}

#[test]
fn draw_list_stats_skip_commands_outside_damage() {
    let draw_list = UiSurfaceDrawList::new(
        (64, 32),
        Some(UiSurfaceRect::new(40.0, 20.0, 8.0, 8.0)),
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [1, 2, 3, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(42.0, 22.0, 2.0, 2.0),
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
        ],
    );

    let stats = draw_list.stats();

    assert_eq!(stats.draw_calls, 1);
    assert_eq!(stats.visible_command_count, 1);
    assert_eq!(stats.visible_draw_item_count, 1);
    assert_eq!(stats.image_count, 1);
    assert_eq!(stats.image_upload_bytes, 16);
}

#[test]
fn draw_list_stats_skip_commands_with_non_finite_or_non_positive_rects() {
    let commands = [
        UiSurfaceRect::new(f32::NAN, 0.0, 10.0, 10.0),
        UiSurfaceRect::new(0.0, 0.0, f32::INFINITY, 10.0),
        UiSurfaceRect::new(0.0, 0.0, 10.0, -1.0),
        UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
    ]
    .into_iter()
    .enumerate()
    .map(|(z_index, frame)| UiSurfaceCommand {
        z_index: z_index as i32,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Quad {
            color: [255, 255, 255, 255],
            corner_radius: 0.0,
        },
    })
    .collect();
    let draw_list = UiSurfaceDrawList::new((64, 32), None, commands);

    let stats = draw_list.stats();

    assert_eq!(stats.visible_command_count, 1);
    assert_eq!(stats.visible_draw_item_count, 1);
}

#[test]
fn draw_list_generation_is_opt_in_for_compiled_presenters() {
    let legacy = UiSurfaceDrawList::new((64, 32), None, Vec::new());
    let versioned = UiSurfaceDrawList::with_generation((64, 32), None, Vec::new(), 9);

    assert_eq!(legacy.generation(), None);
    assert_eq!(versioned.generation(), Some(9));
}

#[test]
fn retargeted_surface_preserves_the_generation_projection_extent() {
    let mut draw_list = UiSurfaceDrawList::with_generation((320, 200), None, Vec::new(), 9);

    draw_list.retarget_surface_size_preserving_projection((160, 100));

    assert_eq!(draw_list.surface_size, (160, 100));
    assert_eq!(draw_list.projection_size(), (320, 200));
    assert_eq!(draw_list.generation(), Some(9));
    assert!(draw_list.bypasses_retained_surface_cache());
}

#[test]
fn compact_draw_list_interns_repeated_solid_and_text_styles() {
    let draw_list = UiSurfaceDrawList::with_generation_and_compact_styles(
        (128, 64),
        None,
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 8.0, 8.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [12, 34, 56, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(10.0, 0.0, 8.0, 8.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [12, 34, 56, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(0.0, 12.0, 40.0, 12.0),
                clip: None,
                kind: UiSurfaceCommandKind::Text {
                    text: "first".to_string(),
                    color: [220, 220, 220, 255],
                    font_family: Some("ui".to_string()),
                    font_weight: 400,
                    font_size: 12.0,
                    line_height: 14.0,
                    style: UiSurfaceTextStyle::Regular,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(44.0, 12.0, 40.0, 12.0),
                clip: None,
                kind: UiSurfaceCommandKind::Text {
                    text: "second".to_string(),
                    color: [220, 220, 220, 255],
                    font_family: Some("ui".to_string()),
                    font_weight: 400,
                    font_size: 12.0,
                    line_height: 14.0,
                    style: UiSurfaceTextStyle::Regular,
                },
            },
        ],
        7,
    );

    assert_eq!(draw_list.style_count(), 2);
    assert!(draw_list
        .commands
        .iter()
        .all(|command| matches!(command.kind, UiSurfaceCommandKind::Styled { .. })));
    assert!(matches!(
        draw_list.resolved_kind(&draw_list.commands[0]),
        Some(UiSurfaceResolvedCommandKind::Quad {
            color: [12, 34, 56, 255],
            corner_radius: 0.0,
        })
    ));
    assert!(matches!(
        draw_list.resolved_kind(&draw_list.commands[3]),
        Some(UiSurfaceResolvedCommandKind::Text { text: "second", .. })
    ));
    assert_eq!(draw_list.stats().visible_command_payload_bytes, 13);
    assert_eq!(draw_list.stats().visible_command_style_count, 2);
}

#[test]
fn draw_list_stats_do_not_count_cached_images_as_uploads() {
    let draw_list = UiSurfaceDrawList::new(
        (64, 32),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
            clip: None,
            kind: UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: "cached".to_string(),
                    resource_generation: 0,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: None,
                    atlas_uv: None,
                },
            },
        }],
    );

    let stats = draw_list.stats();

    assert_eq!(stats.draw_calls, 1);
    assert_eq!(stats.visible_command_count, 1);
    assert_eq!(stats.visible_draw_item_count, 1);
    assert_eq!(stats.image_count, 1);
    assert_eq!(stats.image_upload_bytes, 0);
}

#[test]
fn draw_list_stats_count_same_resource_image_upload_once() {
    let draw_list = UiSurfaceDrawList::new(
        (64, 32),
        None,
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 2.0, 2.0),
                clip: None,
                kind: UiSurfaceCommandKind::Image {
                    payload: UiSurfaceImagePayload {
                        resource_key: "atlas://editor/icons".to_string(),
                        resource_generation: 0,
                        width: 4,
                        height: 4,
                        upload_bytes: 64,
                        rgba: Some(vec![255; 64]),
                        atlas_uv: Some(UiSurfaceImageUvRect {
                            min: [0.0, 0.0],
                            max: [0.5, 0.5],
                        }),
                    },
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(4.0, 0.0, 2.0, 2.0),
                clip: None,
                kind: UiSurfaceCommandKind::Image {
                    payload: UiSurfaceImagePayload {
                        resource_key: "atlas://editor/icons".to_string(),
                        resource_generation: 0,
                        width: 4,
                        height: 4,
                        upload_bytes: 64,
                        rgba: Some(vec![255; 64]),
                        atlas_uv: Some(UiSurfaceImageUvRect {
                            min: [0.5, 0.0],
                            max: [1.0, 0.5],
                        }),
                    },
                },
            },
        ],
    );

    let stats = draw_list.stats();

    assert_eq!(stats.visible_command_count, 2);
    assert_eq!(stats.image_count, 2);
    assert_eq!(stats.image_upload_bytes, 64);
}

#[test]
fn compact_draw_list_keeps_one_owned_image_payload_for_shared_atlas_commands() {
    let image = |x, rgba| UiSurfaceCommand {
        z_index: 0,
        frame: UiSurfaceRect::new(x, 0.0, 2.0, 2.0),
        clip: None,
        kind: UiSurfaceCommandKind::Image {
            payload: UiSurfaceImagePayload {
                resource_key: "atlas://editor/icons".to_string(),
                resource_generation: 23,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(rgba),
                atlas_uv: None,
            },
        },
    };
    let draw_list = UiSurfaceDrawList::with_generation_and_compact_styles(
        (64, 32),
        None,
        vec![image(0.0, vec![4; 16]), image(4.0, vec![9; 16])],
        7,
    );

    assert_eq!(
        draw_list
            .image_resource("atlas://editor/icons", 23)
            .expect("shared atlas resource")
            .rgba,
        vec![4; 16]
    );
    assert!(draw_list.commands.iter().all(|command| matches!(
        &command.kind,
        UiSurfaceCommandKind::Image { payload } if payload.rgba.is_none()
    )));
    assert_eq!(draw_list.stats().image_upload_bytes, 16);
}

#[test]
fn compact_draw_list_keeps_distinct_generations_and_counts_both_uploads() {
    let image = |generation, rgba| UiSurfaceCommand {
        z_index: generation as i32,
        frame: UiSurfaceRect::new(generation as f32 * 4.0, 0.0, 2.0, 2.0),
        clip: None,
        kind: UiSurfaceCommandKind::Image {
            payload: UiSurfaceImagePayload {
                resource_key: "atlas://editor/icons".to_string(),
                resource_generation: generation,
                width: 2,
                height: 2,
                upload_bytes: 16,
                rgba: Some(rgba),
                atlas_uv: None,
            },
        },
    };
    let draw_list = UiSurfaceDrawList::with_generation_and_compact_styles(
        (64, 32),
        None,
        vec![image(4, vec![4; 16]), image(5, vec![5; 16])],
        8,
    );

    assert_eq!(
        draw_list
            .image_resource("atlas://editor/icons", 4)
            .expect("older atlas generation")
            .rgba,
        vec![4; 16]
    );
    assert_eq!(
        draw_list
            .image_resource("atlas://editor/icons", 5)
            .expect("newer atlas generation")
            .rgba,
        vec![5; 16]
    );
    assert_eq!(draw_list.stats().image_upload_bytes, 32);
}

#[test]
fn draw_list_stats_measure_visible_dynamic_command_payloads() {
    let draw_list = UiSurfaceDrawList::new(
        (64, 32),
        None,
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 16.0, 8.0),
                clip: None,
                kind: UiSurfaceCommandKind::Text {
                    text: "text".to_string(),
                    color: [255, 255, 255, 255],
                    font_family: Some("ui".to_string()),
                    font_weight: 400,
                    font_size: 12.0,
                    line_height: 14.0,
                    style: UiSurfaceTextStyle::Regular,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(20.0, 0.0, 8.0, 8.0),
                clip: None,
                kind: UiSurfaceCommandKind::Image {
                    payload: UiSurfaceImagePayload {
                        resource_key: "icon".to_string(),
                        resource_generation: 0,
                        width: 2,
                        height: 2,
                        upload_bytes: 16,
                        rgba: Some(vec![255; 16]),
                        atlas_uv: None,
                    },
                },
            },
        ],
    );

    let stats = draw_list.stats();

    assert_eq!(stats.visible_command_payload_bytes, 26);
}

#[test]
fn atlas_uv_rect_validates_normalized_finite_bounds() {
    assert!(UiSurfaceImageUvRect {
        min: [0.25, 0.25],
        max: [0.75, 0.75],
    }
    .is_valid());
    assert!(!UiSurfaceImageUvRect {
        min: [0.75, 0.25],
        max: [0.75, 0.75],
    }
    .is_valid());
    assert!(!UiSurfaceImageUvRect {
        min: [0.0, f32::NAN],
        max: [1.0, 1.0],
    }
    .is_valid());
}

#[test]
fn surface_descriptor_rejects_zero_size() {
    assert_eq!(
        UiSurfaceDescriptor::headless("bad", 0, 1)
            .validate()
            .unwrap_err(),
        RhiError::InvalidSurfaceDescriptor {
            label: Some("bad".to_string()),
            reason: "width and height must be greater than zero".to_string(),
        }
    );
}
