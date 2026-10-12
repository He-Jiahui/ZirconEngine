use super::*;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_popup_rows::metrics::workbench_popup_row_metrics;

#[test]
fn shortcut_and_adornment_columns_never_overlap_the_label_or_each_other() {
    let metrics = workbench_popup_row_metrics();
    let row = FrameRect {
        x: 20.0,
        y: 16.0,
        width: 240.0,
        height: 28.0,
    };

    let columns = popup_row_text_columns(&row, &metrics, "Ctrl+Shift+S", true);
    let shortcut = columns.shortcut.expect("shortcut column");
    let adornment_left = row.x + row.width - metrics.adornment_reserved_width;

    assert!(
        columns.label.x + columns.label.width + MENU_POPUP_LABEL_SHORTCUT_GAP
            <= shortcut.x + f32::EPSILON
    );
    assert!(shortcut.x + shortcut.width <= adornment_left + f32::EPSILON);
}

#[test]
fn measured_shortcuts_align_to_the_same_trailing_content_edge() {
    let metrics = workbench_popup_row_metrics();
    let row = FrameRect {
        x: 20.0,
        y: 16.0,
        width: 240.0,
        height: 28.0,
    };

    let short = popup_row_text_columns(&row, &metrics, "F5", false)
        .shortcut
        .expect("short shortcut");
    let long = popup_row_text_columns(&row, &metrics, "Ctrl+Shift+F5", false)
        .shortcut
        .expect("long shortcut");

    assert!(long.x < short.x);
    assert!((long.x + long.width - (short.x + short.width)).abs() <= f32::EPSILON);
    assert!(long.x + long.width <= row.x + row.width - metrics.text_right + f32::EPSILON);
}
