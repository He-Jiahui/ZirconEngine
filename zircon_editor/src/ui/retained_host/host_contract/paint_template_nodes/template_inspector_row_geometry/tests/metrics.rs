use super::super::super::super::paint_theme::METRICS;
use super::*;

#[test]
fn inspector_row_metrics_preserve_the_slate_default_density() {
    let metrics = inspector_row_metrics_from_host(METRICS);

    assert_eq!(metrics.border_width, 1.0);
    assert_eq!(metrics.gap_s, 4.0);
    assert_eq!(metrics.row_text_y, INSPECTOR_ROW_TEXT_Y);
    assert_eq!(metrics.label_width, INSPECTOR_LABEL_WIDTH);
    assert_eq!(metrics.count_width, INSPECTOR_COUNT_WIDTH);
    assert_eq!(metrics.field_text_x, INSPECTOR_FIELD_TEXT_X);
    assert_eq!(metrics.icon_text_gap, 7.0);
    assert_eq!(metrics.field_right_pad, INSPECTOR_FIELD_RIGHT_PAD);
    assert_eq!(metrics.chevron_size, INSPECTOR_CHEVRON_SIZE);
    assert_eq!(metrics.nested_label_width, INSPECTOR_NESTED_LABEL_WIDTH);
    assert_eq!(metrics.nested_label_base_x, INSPECTOR_NESTED_LABEL_BASE_X);
    assert_eq!(
        metrics.nested_label_offset_x,
        INSPECTOR_NESTED_LABEL_OFFSET_X
    );
    assert_eq!(
        metrics.nested_select_offset_x,
        INSPECTOR_NESTED_SELECT_OFFSET_X
    );
    assert_eq!(metrics.field_inset_y, INSPECTOR_FIELD_INSET_Y);
    assert_eq!(metrics.icon_size, INSPECTOR_ICON_SIZE);
    assert_eq!(metrics.chevron_right_pad, INSPECTOR_CHEVRON_RIGHT_PAD);
    assert_eq!(metrics.check_size, INSPECTOR_CHECK_SIZE);
    assert_eq!(
        metrics.shadow_check_default_content_offset_x,
        INSPECTOR_SHADOW_CHECK_DEFAULT_CONTENT_OFFSET_X
    );
}
