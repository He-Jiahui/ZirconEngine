use super::support::*;

#[test]
fn native_template_painter_draws_mui_timeline_dot_connector_and_separator_geometry() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "TimelineSeparator".into(),
            node_id: "TimelineSeparator.node".into(),
            role: "TimelineSeparator".into(),
            component_role: "timeline-separator".into(),
            surface_variant: "elevated".into(),
            frame: frame(4.0, 4.0, 18.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "TimelineDot".into(),
            node_id: "TimelineDot.node".into(),
            role: "TimelineDot".into(),
            component_role: "timeline-dot".into(),
            component_variant: "outlined secondary".into(),
            text_tone: "secondary".into(),
            frame: frame(8.0, 4.0, 10.0, 10.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "TimelineConnector".into(),
            node_id: "TimelineConnector.node".into(),
            role: "TimelineConnector".into(),
            component_role: "timeline-connector".into(),
            button_style: resolved_background(MATERIAL_ACCENT),
            frame: frame(12.0, 16.0, 2.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(36, 40, nodes);

    assert_eq!(pixel(&bytes, 36, 5, 20), BACKGROUND);
    assert_eq!(pixel(&bytes, 36, 13, 4), MUI_SECONDARY_MAIN);
    assert_eq!(pixel(&bytes, 36, 13, 9), BACKGROUND);
    assert_eq!(pixel(&bytes, 36, 12, 24), MATERIAL_ACCENT);
    assert_eq!(pixel(&bytes, 36, 15, 24), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_text_field_variants_without_hiding_value_text() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "OutlinedTextField".into(),
            node_id: "OutlinedTextField.node".into(),
            role: "TextField".into(),
            component_role: "input-field".into(),
            component_variant: "outlined".into(),
            value_text: "Atlas".into(),
            frame: frame(4.0, 4.0, 96.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "FilledTextField".into(),
            node_id: "FilledTextField.node".into(),
            role: "TextField".into(),
            component_role: "input-field".into(),
            component_variant: "filled focused".into(),
            focused: true,
            value_text: "Focused".into(),
            frame: frame(4.0, 44.0, 96.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "StandardTextField".into(),
            node_id: "StandardTextField.node".into(),
            role: "TextField".into(),
            component_role: "input-field".into(),
            component_variant: "standard error".into(),
            validation_level: "error".into(),
            value_text: "Error".into(),
            frame: frame(4.0, 84.0, 96.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(112, 124, nodes);

    assert_eq!(pixel(&bytes, 112, 52, 4), MATERIAL_BORDER);
    assert_eq!(pixel(&bytes, 112, 52, 20), BACKGROUND);
    assert!(
        region_changed(&bytes, 112, 10, 12, 48, 12),
        "outlined text field should still draw its editable value text"
    );
    assert_eq!(
        pixel(&bytes, 112, 12, 52),
        MUI_FIELD_FILLED_BACKGROUND_ON_BLACK
    );
    assert_eq!(pixel(&bytes, 112, 52, 74), MATERIAL_FOCUS_RING);
    assert_ne!(pixel(&bytes, 112, 12, 100), BACKGROUND);
    assert_eq!(pixel(&bytes, 112, 52, 114), MATERIAL_ERROR);
}

#[test]
fn native_template_painter_draws_mui_svg_icon_from_name_without_preview() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "AddCircleIcon".into(),
        node_id: "AddCircleIcon.node".into(),
        role: "SvgIcon".into(),
        component_role: "svg-icon".into(),
        icon_name: "AddCircle".into(),
        button_style: resolved_foreground(MUI_SECONDARY_MAIN),
        frame: frame(4.0, 4.0, 40.0, 40.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(52, 52, nodes);

    assert_eq!(pixel(&bytes, 52, 5, 5), BACKGROUND);
    assert!(
        contains_pixel(&bytes, MUI_SECONDARY_MAIN),
        "SvgIcon should load the local MUI module and use the resolved foreground tint"
    );
}

#[test]
fn native_template_painter_draws_missing_mui_icon_fallback() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "MissingIcon".into(),
        node_id: "MissingIcon.node".into(),
        role: "Icon".into(),
        component_role: "icon".into(),
        icon_name: "missing_zircon_mui_icon".into(),
        button_style: resolved_foreground(MUI_CHIP_WARNING),
        frame: frame(4.0, 4.0, 32.0, 32.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(44, 44, nodes);

    assert!(
        contains_pixel(&bytes, MUI_CHIP_WARNING),
        "missing Icon nodes should produce a visible tinted fallback instead of a blank slot"
    );
}

#[test]
fn native_template_painter_draws_mui_avatar_rounded_fallback_shape() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "RoundedAvatar".into(),
        node_id: "RoundedAvatar.node".into(),
        role: "Avatar".into(),
        component_role: "avatar".into(),
        component_variant: "rounded colorDefault".into(),
        text: "ZR".into(),
        button_style: resolved_avatar_style(
            MUI_AVATAR_SURFACE,
            MUI_SECONDARY_MAIN,
            Some(MUI_SECONDARY_MAIN),
            1.0,
            4.0,
        ),
        corner_radius: 4.0,
        border_width: 1.0,
        frame: frame(4.0, 4.0, 24.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(40, 36, nodes);

    assert_eq!(pixel(&bytes, 40, 4, 4), BACKGROUND);
    assert_eq!(pixel(&bytes, 40, 8, 4), MUI_SECONDARY_MAIN);
    assert_eq!(pixel(&bytes, 40, 27, 16), MUI_SECONDARY_MAIN);
    assert_eq!(pixel(&bytes, 40, 26, 16), MUI_AVATAR_SURFACE);
}
