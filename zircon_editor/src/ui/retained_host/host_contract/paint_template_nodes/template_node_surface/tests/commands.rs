use super::*;

#[test]
fn asset_thumbnail_name_area_surface_squares_top_edge_over_bottom_rounded_base() {
    let mut node = TemplatePaneNodeData {
        role: "Panel".into(),
        surface_variant: ASSET_THUMBNAIL_NAME_AREA_SURFACE.into(),
        corner_radius: 4.0,
        selected: true,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 96.0,
        height: 42.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 128.0,
        height: 96.0,
    };
    let mut commands = Vec::new();

    push_surface_commands(&mut commands, &node, &rect, &clip, 7, 1.0);

    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0].frame, rect);
    assert_eq!(commands[0].z_index, 7);
    assert_eq!(commands[0].corner_radius, 4.0);
    assert_eq!(commands[1].frame.x, rect.x);
    assert_eq!(commands[1].frame.y, rect.y);
    assert_eq!(commands[1].frame.width, rect.width);
    assert_eq!(commands[1].frame.height, 4.0);
    assert_eq!(commands[1].z_index, 7);
    assert_eq!(commands[1].background_color, commands[0].background_color);
    assert_eq!(commands[1].border_color, None);
    assert_eq!(commands[1].border_width, 0.0);
    assert_eq!(commands[1].corner_radius, 0.0);

    node.surface_variant = "panel".into();
    commands.clear();
    push_surface_commands(&mut commands, &node, &rect, &clip, 7, 1.0);

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].corner_radius, 4.0);
}

#[test]
fn asset_thumbnail_name_area_state_layer_keeps_square_top_edge() {
    let node = TemplatePaneNodeData {
        role: "Panel".into(),
        surface_variant: ASSET_THUMBNAIL_NAME_AREA_SURFACE.into(),
        corner_radius: 4.0,
        state_layer_enabled: true,
        hovered: true,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 96.0,
        height: 42.0,
    };
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 128.0,
        height: 96.0,
    };
    let mut commands = Vec::new();

    push_surface_commands(&mut commands, &node, &rect, &clip, 7, 1.0);

    assert_eq!(commands.len(), 4);
    assert_eq!(commands[2].frame, rect);
    assert_eq!(commands[2].z_index, 8);
    assert_eq!(commands[2].corner_radius, 4.0);
    assert_eq!(commands[3].frame.x, rect.x);
    assert_eq!(commands[3].frame.y, rect.y);
    assert_eq!(commands[3].frame.width, rect.width);
    assert_eq!(commands[3].frame.height, 4.0);
    assert_eq!(commands[3].z_index, 8);
    assert_eq!(commands[3].background_color, commands[2].background_color);
    assert_eq!(commands[3].border_color, None);
    assert_eq!(commands[3].border_width, 0.0);
    assert_eq!(commands[3].corner_radius, 0.0);
    assert_eq!(commands[3].opacity, commands[2].opacity);
}

#[test]
fn degenerate_surface_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 0.0,
        height: 42.0,
    };
    let mut commands = Vec::new();

    push_surface_commands(
        &mut commands,
        &TemplatePaneNodeData::default(),
        &rect,
        &rect,
        7,
        1.0,
    );

    assert!(commands.is_empty());
}

#[test]
fn narrow_surface_radius_stays_inside_the_surface_frame() {
    let node = TemplatePaneNodeData {
        role: "Panel".into(),
        corner_radius: 4.0,
        ..TemplatePaneNodeData::default()
    };
    let rect = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 0.5,
        height: 42.0,
    };
    let mut commands = Vec::new();

    push_surface_commands(&mut commands, &node, &rect, &rect, 7, 1.0);

    assert!(commands
        .iter()
        .all(|command| command.corner_radius <= rect.width * 0.5));
}

#[test]
fn surface_overlapping_clip_keeps_paint_commands() {
    let rect = FrameRect {
        x: 11.0,
        y: 20.0,
        width: 96.0,
        height: 42.0,
    };
    let clip = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 96.0,
        height: 42.0,
    };
    let mut commands = Vec::new();

    push_surface_commands(
        &mut commands,
        &TemplatePaneNodeData::default(),
        &rect,
        &clip,
        7,
        1.0,
    );

    assert!(!commands.is_empty());
    assert!(commands
        .iter()
        .all(|command| command.clip_frame.as_ref() == Some(&clip)));
}

#[test]
fn surface_outside_clip_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 11.0,
        y: 20.0,
        width: 96.0,
        height: 42.0,
    };
    let clip = FrameRect {
        x: 108.0,
        y: 20.0,
        width: 96.0,
        height: 42.0,
    };
    let mut commands = Vec::new();

    push_surface_commands(
        &mut commands,
        &TemplatePaneNodeData::default(),
        &rect,
        &clip,
        7,
        1.0,
    );

    assert!(commands.is_empty());
}
