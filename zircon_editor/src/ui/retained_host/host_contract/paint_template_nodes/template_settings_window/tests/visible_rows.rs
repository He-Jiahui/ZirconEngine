use super::*;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;

#[test]
fn visible_setting_rows_are_clip_bounded_with_one_overscan_row() {
    let metrics = current_host_metrics();
    let panel = FrameRect {
        x: 10.0,
        y: 20.0,
        width: 860.0,
        height: 560.0,
    };
    let layout = SettingsWindowLayout::new(&panel, metrics, 0.0, 0, 0.0, 64);
    let clip = FrameRect {
        x: layout.setting_list.x,
        y: layout.setting_list.y + layout.setting_row_height * 10.25,
        width: layout.setting_list.width,
        height: layout.setting_row_height * 2.5,
    };

    assert_eq!(
        settings_window_visible_rows(&layout.setting_list, &clip, 64, &layout),
        9..14
    );
}
