use super::support::*;

#[test]
fn native_template_painter_clips_mui_avatar_image_to_circular_shape() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ImageAvatar".into(),
        node_id: "ImageAvatar.node".into(),
        role: "Avatar".into(),
        component_role: "avatar".into(),
        component_variant: "circular".into(),
        has_preview_image: true,
        preview_image: solid_preview_image(MUI_AVATAR_IMAGE),
        button_style: resolved_avatar_style(MUI_AVATAR_SURFACE, MUI_SECONDARY_MAIN, None, 0.0, 0.0),
        frame: frame(4.0, 4.0, 24.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(40, 36, nodes);

    assert_eq!(pixel(&bytes, 40, 4, 4), BACKGROUND);
    assert_eq!(pixel(&bytes, 40, 5, 5), BACKGROUND);
    assert_eq!(pixel(&bytes, 40, 16, 4), MUI_AVATAR_IMAGE);
    assert_eq!(pixel(&bytes, 40, 16, 16), MUI_AVATAR_IMAGE);
}

#[test]
fn native_template_painter_draws_mui_badge_standard_bottom_left_overlay() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "ErrorBadge".into(),
        node_id: "ErrorBadge.node".into(),
        role: "Badge".into(),
        component_role: "badge".into(),
        component_variant:
            "standard error circular bottom left overlapCircular anchorOriginBottomLeftCircular"
                .into(),
        text: "Alerts".into(),
        value_text: "12".into(),
        button_style: resolved_avatar_style(
            MUI_X_SURFACE_INSET,
            MATERIAL_ACCENT,
            Some(MUI_SECONDARY_MAIN),
            1.0,
            10.0,
        ),
        corner_radius: 10.0,
        border_width: 1.0,
        frame: frame(16.0, 4.0, 64.0, 28.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(96, 48, nodes);

    assert_eq!(pixel(&bytes, 96, 20, 12), MUI_X_SURFACE_INSET);
    assert_eq!(pixel(&bytes, 96, 10, 28), BACKGROUND);
    assert_eq!(pixel(&bytes, 96, 24, 28), MUI_BADGE_ERROR);
}

#[test]
fn native_template_painter_hides_mui_badge_invisible_dot_and_consumes_badge_slot() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "HiddenBadge".into(),
            node_id: "HiddenBadge.node".into(),
            role: "Badge".into(),
            component_role: "badge".into(),
            component_variant: "dot invisible error circular bottom left".into(),
            frame: frame(16.0, 4.0, 64.0, 28.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "HiddenBadgeSlot".into(),
            node_id: "HiddenBadge.slot".into(),
            role: "Label".into(),
            component_role: "label".into(),
            component_variant: "muiBadgeSlot dot invisible error circular bottom left".into(),
            text: "x".into(),
            button_style: resolved_background(MUI_BADGE_ERROR),
            frame: frame(20.0, 20.0, 18.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 48, nodes);

    assert_eq!(pixel(&bytes, 96, 28, 28), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_chip_outlined_delete_icon_geometry() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "WarningChip".into(),
        node_id: "WarningChip.node".into(),
        role: "Chip".into(),
        component_role: "chip".into(),
        component_variant: "outlined small warning clickable deletable hasDeleteIcon".into(),
        text: "Warn".into(),
        button_style: resolved_foreground_border_style(MUI_CHIP_WARNING, MUI_CHIP_WARNING, 1.0),
        frame: frame(4.0, 4.0, 80.0, 28.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(96, 40, nodes);

    assert_eq!(pixel(&bytes, 96, 20, 6), MUI_CHIP_WARNING);
    assert!(color_near(pixel(&bytes, 96, 20, 18), MUI_CHIP_WARNING, 3));
    assert!(region_contains_color_near(
        &bytes,
        96,
        67,
        13,
        11,
        11,
        MUI_CHIP_WARNING,
        3
    ));
}

#[test]
fn native_template_painter_draws_mui_chip_avatar_and_consumes_chip_slot() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "PrimaryChip".into(),
            node_id: "PrimaryChip.node".into(),
            role: "Chip".into(),
            component_role: "chip".into(),
            component_variant: "filled medium primary hasAvatar".into(),
            text: "Build".into(),
            frame: frame(4.0, 4.0, 104.0, 36.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "PrimaryChipAvatar".into(),
            node_id: "PrimaryChip.avatar".into(),
            role: "Avatar".into(),
            component_role: "avatar".into(),
            component_variant: "chipSlotAvatar".into(),
            button_style: resolved_background(MUI_AVATAR_IMAGE),
            frame: frame(8.0, 8.0, 24.0, 24.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(120, 48, nodes);

    assert_eq!(pixel(&bytes, 120, 50, 20), MUI_CHIP_PRIMARY);
    assert_eq!(pixel(&bytes, 120, 21, 22), MUI_CHIP_PRIMARY_DARK);
}

#[test]
fn native_template_painter_sorts_template_nodes_by_mui_z_index() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "Tooltip".into(),
            node_id: "Tooltip.node".into(),
            role: "Panel".into(),
            component_role: "tooltip".into(),
            surface_variant: "tooltip".into(),
            z_index: 1500,
            frame: frame(4.0, 4.0, 32.0, 20.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "NormalPanel".into(),
            node_id: "NormalPanel.node".into(),
            role: "Panel".into(),
            component_role: "panel".into(),
            surface_variant: "primary".into(),
            z_index: 0,
            frame: frame(4.0, 4.0, 32.0, 20.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(48, 32, nodes);

    assert_eq!(pixel(&bytes, 48, 12, 12), MUI_TOOLTIP_BG);
}
