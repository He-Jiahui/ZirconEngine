use super::support::*;

#[test]
fn native_template_painter_applies_mui_transition_opacity() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "FadeTooltip".into(),
        node_id: "FadeTooltip.node".into(),
        role: "Panel".into(),
        component_role: "tooltip".into(),
        surface_variant: "tooltip".into(),
        transition_kind: "fade".into(),
        transition_progress: 0.5,
        frame: frame(4.0, 4.0, 32.0, 20.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(48, 32, nodes);

    assert_eq!(pixel(&bytes, 48, 12, 12), MUI_TOOLTIP_BG_FADE_HALF_ON_BLACK);
}

#[test]
fn native_template_painter_draws_mui_x_data_grid_chrome() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "DataGrid".into(),
        node_id: "DataGrid.node".into(),
        role: "DataGrid".into(),
        component_role: "mui-x-data-grid".into(),
        selected: true,
        checked: true,
        focused: true,
        corner_radius: 10.0,
        border_width: 1.0,
        frame: frame(4.0, 4.0, 96.0, 38.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 52, nodes);

    assert_eq!(pixel(&bytes, 112, 50, 10), MUI_X_GRID_HEADER);
    assert_eq!(pixel(&bytes, 112, 20, 22), MUI_X_GRID_SELECTED_ROW);
    assert_eq!(pixel(&bytes, 112, 20, 27), MUI_X_GRID_ROW);
}

#[test]
fn native_template_painter_draws_mui_x_tree_view_items() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "MaterialTreeView".into(),
        node_id: "MaterialTreeView.node".into(),
        role: "MaterialTreeView".into(),
        component_role: "mui-x-tree-view".into(),
        selected: true,
        checked: true,
        expanded: true,
        popup_open: true,
        focused: true,
        corner_radius: 10.0,
        frame: frame(4.0, 4.0, 96.0, 38.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 52, nodes);

    assert_eq!(pixel(&bytes, 112, 50, 40), MUI_X_TREE_SURFACE);
    assert_eq!(pixel(&bytes, 112, 20, 12), MUI_X_GRID_SELECTED_ROW);
    assert_eq!(pixel(&bytes, 112, 26, 24), MUI_X_GRID_HEADER);
    assert_eq!(pixel(&bytes, 112, 12, 11), MUI_X_TREE_MARKER);
}

#[test]
fn native_template_painter_draws_mui_x_root_custom_surface_color() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "CustomDataGrid".into(),
        node_id: "CustomDataGrid.node".into(),
        role: "DataGrid".into(),
        component_role: "panel".into(),
        button_style: resolved_background(MUI_X_CUSTOM_SURFACE),
        frame: frame(4.0, 4.0, 96.0, 38.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 52, nodes);

    assert_eq!(pixel(&bytes, 112, 50, 10), MUI_X_GRID_HEADER);
    assert_eq!(pixel(&bytes, 112, 50, 38), MUI_X_CUSTOM_SURFACE);
}

#[test]
fn native_template_painter_draws_mui_x_date_time_picker_field_and_popup() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "DateTimePickers".into(),
        node_id: "DateTimePickers.node".into(),
        role: "DateTimePickers".into(),
        component_role: "mui-x-date-time-pickers".into(),
        component_variant: "desktop".into(),
        selected: true,
        popup_open: true,
        focused: true,
        corner_radius: 10.0,
        border_width: 1.0,
        frame: frame(4.0, 4.0, 96.0, 50.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 64, nodes);

    assert_eq!(pixel(&bytes, 112, 48, 12), MUI_X_SURFACE_INSET);
    assert_eq!(pixel(&bytes, 112, 91, 12), MUI_X_PICKER_SECONDARY);
    assert_eq!(pixel(&bytes, 112, 20, 44), MUI_X_GRID_ROW);
    assert_eq!(pixel(&bytes, 112, 50, 42), MUI_X_PICKER_SECONDARY);
}
