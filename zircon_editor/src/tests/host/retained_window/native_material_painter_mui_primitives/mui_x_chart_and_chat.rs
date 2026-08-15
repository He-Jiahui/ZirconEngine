use super::support::*;

#[test]
fn native_template_painter_draws_mui_x_chart_plot_and_series() {
    let nodes = model_rc(vec![TemplatePaneNodeData {
        control_id: "Charts".into(),
        node_id: "Charts.node".into(),
        role: "Charts".into(),
        component_role: "mui-x-charts".into(),
        component_variant: "loading".into(),
        focused: true,
        corner_radius: 10.0,
        border_width: 1.0,
        frame: frame(4.0, 4.0, 96.0, 48.0),
        ..TemplatePaneNodeData::default()
    }]);

    let bytes = paint_template_nodes_for_test(112, 60, nodes);

    assert_eq!(pixel(&bytes, 112, 90, 20), MUI_X_CHART_PLOT_BG);
    assert_eq!(pixel(&bytes, 112, 30, 36), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 112, 50, 36), MUI_X_CHART_SUCCESS);
}

#[test]
fn native_template_painter_draws_mui_x_chart_subtype_feedback() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "LineChart".into(),
            node_id: "LineChart.node".into(),
            role: "LineChart".into(),
            component_role: "mui-x-line-chart".into(),
            focused: true,
            corner_radius: 10.0,
            border_width: 1.0,
            frame: frame(4.0, 4.0, 96.0, 48.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "PieChart".into(),
            node_id: "PieChart.node".into(),
            role: "PieChart".into(),
            component_role: "mui-x-pie-chart".into(),
            selected: true,
            checked: true,
            corner_radius: 10.0,
            border_width: 1.0,
            frame: frame(104.0, 4.0, 96.0, 48.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "Gauge".into(),
            node_id: "Gauge.node".into(),
            role: "Gauge".into(),
            component_role: "mui-x-gauge".into(),
            value_percent: 0.68,
            focused: true,
            corner_radius: 10.0,
            border_width: 1.0,
            frame: frame(4.0, 60.0, 96.0, 48.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "SparkLineChart".into(),
            node_id: "SparkLineChart.node".into(),
            role: "SparkLineChart".into(),
            component_role: "mui-x-sparkline".into(),
            hovered: true,
            corner_radius: 10.0,
            border_width: 1.0,
            frame: frame(104.0, 60.0, 96.0, 48.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(212, 116, nodes);

    assert_eq!(pixel(&bytes, 212, 72, 20), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 212, 160, 28), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 212, 144, 28), MUI_X_CHART_SUCCESS);
    assert_eq!(pixel(&bytes, 212, 152, 28), MUI_X_CHART_PLOT_BG);
    assert_eq!(pixel(&bytes, 212, 52, 72), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 212, 164, 78), MUI_X_CHART_PRIMARY);
}

#[test]
fn native_template_painter_draws_mui_x_agent_chat_and_composer() {
    let nodes = model_rc(vec![
        TemplatePaneNodeData {
            control_id: "AgentChat".into(),
            node_id: "AgentChat.node".into(),
            role: "AgentChat".into(),
            component_role: "mui-x-agent-chat".into(),
            component_variant: "streaming".into(),
            validation_level: "error".into(),
            focused: true,
            frame: frame(4.0, 4.0, 96.0, 44.0),
            ..TemplatePaneNodeData::default()
        },
        TemplatePaneNodeData {
            control_id: "ChatComposer".into(),
            node_id: "ChatComposer.node".into(),
            role: "ChatComposer".into(),
            component_role: "mui-x-chat-composer".into(),
            focused: true,
            frame: frame(4.0, 52.0, 96.0, 18.0),
            ..TemplatePaneNodeData::default()
        },
    ]);

    let bytes = paint_template_nodes_for_test(112, 76, nodes);

    assert_eq!(pixel(&bytes, 112, 50, 6), MUI_X_CHAT_ERROR_SURFACE);
    assert_eq!(pixel(&bytes, 112, 12, 12), MUI_X_CHAT_BUBBLE);
    assert_eq!(pixel(&bytes, 112, 82, 26), MUI_X_CHAT_SELECTED_BUBBLE);
    assert_eq!(pixel(&bytes, 112, 20, 43), MUI_X_CHART_PRIMARY);
    assert_eq!(pixel(&bytes, 112, 88, 61), MUI_X_CHART_PRIMARY);
}
