use super::{prompt_button_label_x, prompt_button_label_y};
use crate::ui::retained_host::host_contract::{data::FrameRect, paint_theme::METRICS};

#[test]
fn prompt_button_label_offsets_follow_host_metrics_inside_narrow_buttons() {
    let button = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 32.0,
        height: 32.0,
    };

    assert_eq!(prompt_button_label_x(&button, METRICS), 16.0);
    assert_eq!(prompt_button_label_y(&button, METRICS), 16.0);

    let narrow = FrameRect {
        width: 12.0,
        height: 10.0,
        ..button
    };
    assert!(prompt_button_label_x(&narrow, METRICS) <= narrow.x + narrow.width);
    assert!(prompt_button_label_y(&narrow, METRICS) >= narrow.y);
}
