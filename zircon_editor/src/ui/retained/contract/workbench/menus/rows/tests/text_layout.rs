use super::*;

#[test]
fn shortcut_column_reserves_non_overlapping_runtime_measured_label_clip() {
    let row = FrameRect {
        x: 100.0,
        y: 40.0,
        width: 280.0,
        height: 28.0,
    };
    let popup = FrameRect {
        x: 94.0,
        y: 34.0,
        width: 292.0,
        height: 192.0,
    };

    let columns = menu_row_text_columns(&row, &popup, "Ctrl+Shift+U");
    let shortcut_x = columns.shortcut_x.expect("shortcut column");

    assert!(
        columns.label_clip.x + columns.label_clip.width
            <= shortcut_x - MENU_POPUP_LABEL_SHORTCUT_GAP
    );
    assert!(
        shortcut_x + menu_popup_text_width("Ctrl+Shift+U")
            <= row.x + row.width - MENU_POPUP_TEXT_INSET_X + f32::EPSILON
    );
}

#[test]
fn scrolled_row_label_clip_stays_inside_popup_viewport() {
    let row = FrameRect {
        x: 100.0,
        y: 12.0,
        width: 220.0,
        height: 28.0,
    };
    let popup = FrameRect {
        x: 94.0,
        y: 28.0,
        width: 232.0,
        height: 192.0,
    };

    let columns = menu_row_text_columns(&row, &popup, "");

    assert_eq!(columns.label_clip.y, popup.y);
    assert_eq!(columns.label_clip.height, 12.0);
    assert_eq!(columns.shortcut_x, None);
}
