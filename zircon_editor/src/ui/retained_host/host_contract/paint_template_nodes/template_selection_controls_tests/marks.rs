use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::{
    RADIO_CHECKED_BORDER, RADIO_CHECKED_FILL, RADIO_DOT_SIZE, SELECTION_LABEL_MUTED,
    SELECTION_MARK_IDLE_BORDER, SELECTION_MARK_IDLE_FILL, centered_square, checkbox_background,
    checkbox_border_color, control_accent_color, label_rect_after_mark, leading_mark_rect,
    radio_background, radio_border_color, radio_dot_size, selection_mark_label_color,
    workbench_selection_control_metrics_from_host,
};
use super::support::{node_with_role, resolved_background_and_border};
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

#[test]
fn checkbox_radio_marks_use_showcase_metrics_and_tones() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 140.0,
        height: 28.0,
    };
    let unchecked = node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOff");
    let mark = leading_mark_rect(&unchecked, &rect);
    let label = label_rect_after_mark(&unchecked, &rect, &mark);
    let checked_radio = TemplatePaneNodeData {
        checked: true,
        selected: true,
        ..node_with_role("Radio", "radio", "WorkbenchRadioOn")
    };
    let dot = centered_square(&mark, radio_dot_size(&checked_radio));

    assert_eq!(mark.x, 10.0);
    assert_eq!(mark.y, 6.0);
    assert_eq!(mark.width, 16.0);
    assert_eq!(mark.height, 16.0);
    assert_eq!(label.x, 35.0);
    assert_eq!(checkbox_background(&unchecked), SELECTION_MARK_IDLE_FILL);
    assert_eq!(
        checkbox_border_color(&unchecked),
        SELECTION_MARK_IDLE_BORDER
    );
    assert_eq!(
        selection_mark_label_color(&unchecked),
        SELECTION_LABEL_MUTED
    );
    assert_eq!(radio_background(&checked_radio), RADIO_CHECKED_FILL);
    assert_eq!(radio_border_color(&checked_radio), RADIO_CHECKED_BORDER);
    assert_eq!(radio_dot_size(&checked_radio), 5.0);
    assert_eq!(dot.width, RADIO_DOT_SIZE);
    assert_eq!(dot.height, RADIO_DOT_SIZE);
    assert!(dot.width < mark.width * 0.4);
}

#[test]
fn checkbox_radio_marks_consume_declared_style_and_label_colors() {
    let unchecked = TemplatePaneNodeData {
        label_color: crate::ui::retained_host::primitives::Color::from_rgb_u8(131, 141, 148),
        button_style: resolved_background_and_border([19, 25, 29, 255], [55, 65, 72, 255]),
        ..node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxOff")
    };
    let unchecked_radio = TemplatePaneNodeData {
        button_style: resolved_background_and_border([19, 25, 29, 255], [55, 65, 72, 255]),
        ..node_with_role("Radio", "radio", "WorkbenchRadioOff")
    };
    let checked = TemplatePaneNodeData {
        checked: true,
        selected: true,
        button_style: resolved_background_and_border([33, 160, 169, 255], [34, 161, 170, 255]),
        ..node_with_role("Radio", "radio", "WorkbenchRadioOn")
    };

    assert_eq!(checkbox_background(&unchecked), [19, 25, 29, 255]);
    assert_eq!(checkbox_border_color(&unchecked), [55, 65, 72, 255]);
    assert_eq!(selection_mark_label_color(&unchecked), [131, 141, 148, 255]);
    assert_eq!(radio_background(&unchecked_radio), [19, 25, 29, 255]);
    assert_eq!(radio_border_color(&unchecked_radio), [55, 65, 72, 255]);
    assert_eq!(radio_background(&checked), RADIO_CHECKED_FILL);
    assert_eq!(radio_border_color(&checked), RADIO_CHECKED_BORDER);
}

#[test]
fn radio_uses_declared_dot_size_and_color() {
    let node = TemplatePaneNodeData {
        checked: true,
        selected: true,
        value_number: 6.0,
        value_color: crate::ui::retained_host::primitives::Color::from_rgb_u8(67, 216, 226),
        ..node_with_role("Radio", "radio", "WorkbenchRadioOn")
    };

    assert_eq!(radio_dot_size(&node), 6.0);
    assert_eq!(control_accent_color(&node), [67, 216, 226, 255]);
}

#[test]
fn selection_control_honors_declared_mark_size_and_label_gap() {
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 140.0,
        height: 28.0,
    };
    let node = TemplatePaneNodeData {
        layout_icon_size: 14.0,
        layout_content_offset_x: 8.0,
        ..node_with_role("Checkbox", "checkbox", "WorkbenchCheckboxCustom")
    };
    let mark = leading_mark_rect(&node, &rect);
    let label = label_rect_after_mark(&node, &rect, &mark);

    assert_eq!(mark.x, 10.0);
    assert_eq!(mark.y, 7.0);
    assert_eq!(mark.width, 14.0);
    assert_eq!(mark.height, 14.0);
    assert_eq!(label.x, 32.0);
}

#[test]
fn selection_control_metrics_project_from_host_control_metrics() {
    let mut host = METRICS;
    host.border_width = 2.0;
    host.font_body = 11.5;
    host.gap_s = 5.0;
    host.gap_m = 10.0;
    host.gap_l = 14.0;
    host.row_height = 30.0;

    let metrics = workbench_selection_control_metrics_from_host(host);

    assert_eq!(metrics.mark_inset_x, 14.0);
    assert_eq!(metrics.mark_size, 16.0);
    assert_eq!(metrics.label_gap, 12.0);
    assert_eq!(metrics.text_inset_y, 7.0);
    assert_eq!(metrics.radio_dot_size, 7.0);
    assert_eq!(metrics.toggle_track_width, 42.0);
    assert_eq!(metrics.toggle_track_height, 20.0);
    assert_eq!(metrics.toggle_thumb_size, 11.0);
    assert_eq!(metrics.toggle_right_inset, 10.0);
    assert_eq!(metrics.toggle_thumb_inset, 4.0);
    assert_eq!(metrics.font_size, 11.5);
    assert!((metrics.line_height - 13.8).abs() < 0.001);
}
