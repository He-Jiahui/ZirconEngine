use super::support::*;

#[test]
fn native_template_painter_draws_mui_linear_progress_track_and_fill() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "LinearProgress".into(),
        node_id: "LinearProgress.node".into(),
        role: "Progress".into(),
        component_role: "progress".into(),
        component_variant: "determinate linear".into(),
        value_percent: 0.62,
        frame: frame(4.0, 8.0, 100.0, 8.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(120, 28, nodes);

    assert_eq!(pixel(&bytes, 120, 12, 12), MATERIAL_ACCENT);
    assert_eq!(pixel(&bytes, 120, 92, 12), MATERIAL_PROGRESS_TRACK);
    assert_eq!(pixel(&bytes, 120, 112, 12), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_circular_progress_ring() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "CircularProgress".into(),
        node_id: "CircularProgress.node".into(),
        role: "Progress".into(),
        component_role: "progress".into(),
        component_variant: "circular determinate".into(),
        value_percent: 0.5,
        frame: frame(4.0, 4.0, 32.0, 32.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(44, 44, nodes);

    assert_eq!(pixel(&bytes, 44, 20, 5), MATERIAL_ACCENT);
    assert_eq!(pixel(&bytes, 44, 20, 34), MATERIAL_ACCENT);
    assert_eq!(pixel(&bytes, 44, 5, 20), MATERIAL_PROGRESS_TRACK);
    assert_eq!(pixel(&bytes, 44, 20, 20), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_skeleton_variants() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "RoundedSkeleton".into(),
            node_id: "RoundedSkeleton.node".into(),
            role: "Skeleton".into(),
            component_role: "skeleton".into(),
            component_variant: "rounded".into(),
            frame: frame(4.0, 4.0, 80.0, 16.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "CircularSkeleton".into(),
            node_id: "CircularSkeleton.node".into(),
            role: "Skeleton".into(),
            component_role: "skeleton".into(),
            component_variant: "circular".into(),
            frame: frame(4.0, 28.0, 80.0, 16.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 52, nodes);

    assert_eq!(pixel(&bytes, 96, 44, 12), MATERIAL_SKELETON_BG);
    assert_eq!(pixel(&bytes, 96, 4, 36), BACKGROUND);
    assert_eq!(pixel(&bytes, 96, 44, 36), MATERIAL_SKELETON_BG);
}

#[test]
fn native_template_painter_draws_mui_skeleton_text_wave_and_hides_children() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "TextSkeleton".into(),
            node_id: "TextSkeleton.node".into(),
            role: "Skeleton".into(),
            component_role: "skeleton".into(),
            component_variant: "text wave withChildren".into(),
            frame: frame(4.0, 4.0, 100.0, 20.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "TextSkeletonChild".into(),
            node_id: "TextSkeletonChild.node".into(),
            role: "Label".into(),
            component_variant: "muiSkeletonChild".into(),
            text: "Loading".into(),
            button_style: resolved_background(MUI_AVATAR_IMAGE),
            frame: frame(12.0, 10.0, 40.0, 8.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(116, 32, nodes);

    assert_eq!(pixel(&bytes, 116, 10, 6), BACKGROUND);
    assert_eq!(pixel(&bytes, 116, 20, 14), MATERIAL_SKELETON_BG);
    assert_eq!(pixel(&bytes, 116, 44, 14), MUI_SKELETON_WAVE_ON_BG);
}

#[test]
fn native_template_painter_draws_mui_backdrop_scrim_and_invisible_variant() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "Backdrop".into(),
            node_id: "Backdrop.node".into(),
            role: "Backdrop".into(),
            component_role: "backdrop".into(),
            popup_open: true,
            frame: frame(0.0, 0.0, 32.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "InvisibleBackdrop".into(),
            node_id: "InvisibleBackdrop.node".into(),
            role: "Backdrop".into(),
            component_role: "backdrop".into(),
            component_variant: "invisible".into(),
            popup_open: true,
            frame: frame(36.0, 0.0, 32.0, 32.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test_with_background(72, 36, MID_BACKGROUND, nodes);

    assert_eq!(pixel(&bytes, 72, 16, 16), MUI_BACKDROP_ON_MID_BACKGROUND);
    assert_eq!(pixel(&bytes, 72, 52, 16), MID_BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_overlay_surface_tones() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "Tooltip".into(),
            node_id: "Tooltip.node".into(),
            role: "Panel".into(),
            component_role: "tooltip".into(),
            surface_variant: "tooltip".into(),
            corner_radius: 4.0,
            frame: frame(4.0, 4.0, 60.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "Snackbar".into(),
            node_id: "Snackbar.node".into(),
            role: "Panel".into(),
            component_role: "snackbar".into(),
            surface_variant: "snackbar".into(),
            corner_radius: 4.0,
            elevation: 6.0,
            frame: frame(4.0, 28.0, 80.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(96, 56, nodes);

    assert_eq!(pixel(&bytes, 96, 12, 12), MUI_TOOLTIP_BG);
    assert_eq!(pixel(&bytes, 96, 12, 36), MUI_SNACKBAR_BG);
}

#[test]
fn native_template_painter_draws_mui_alert_severity_surface() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "WarningAlert".into(),
        node_id: "WarningAlert.node".into(),
        role: "Alert".into(),
        component_role: "alert".into(),
        surface_variant: "alert".into(),
        validation_level: "warning".into(),
        corner_radius: 4.0,
        frame: frame(4.0, 4.0, 88.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(100, 36, nodes);

    assert_eq!(pixel(&bytes, 100, 44, 16), MATERIAL_WARNING_CONTAINER);
}

#[test]
fn native_template_painter_draws_mui_divider_middle_horizontal_line() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "HorizontalDivider".into(),
        node_id: "HorizontalDivider.node".into(),
        role: "Divider".into(),
        component_role: "divider".into(),
        component_variant: "middle horizontal".into(),
        frame: frame(4.0, 4.0, 120.0, 24.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(132, 36, nodes);

    assert_eq!(pixel(&bytes, 132, 18, 16), BACKGROUND);
    assert_eq!(pixel(&bytes, 132, 22, 16), MATERIAL_DIVIDER);
    assert_eq!(pixel(&bytes, 132, 106, 16), MATERIAL_DIVIDER);
    assert_eq!(pixel(&bytes, 132, 110, 16), BACKGROUND);
}

#[test]
fn native_template_painter_draws_mui_divider_vertical_with_children_gap() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "VerticalDivider".into(),
        node_id: "VerticalDivider.node".into(),
        role: "Divider".into(),
        component_role: "divider".into(),
        component_variant: "middle vertical flexItem withChildren".into(),
        text: " ".into(),
        frame: frame(10.0, 4.0, 24.0, 80.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(48, 92, nodes);

    assert_eq!(pixel(&bytes, 48, 22, 8), BACKGROUND);
    assert_eq!(pixel(&bytes, 48, 22, 14), MATERIAL_DIVIDER);
    assert_eq!(pixel(&bytes, 48, 22, 44), BACKGROUND);
    assert_eq!(pixel(&bytes, 48, 22, 74), MATERIAL_DIVIDER);
}
