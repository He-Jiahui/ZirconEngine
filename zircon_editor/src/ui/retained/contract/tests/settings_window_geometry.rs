use super::*;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;

fn ultra_preferences_frame() -> FrameRect {
    FrameRect {
        x: 12.0,
        y: 12.0,
        width: 396.0,
        height: 336.0,
    }
}

#[test]
fn setting_scroll_projects_paint_and_hit_rows_from_one_offset_authority() {
    let frame = ultra_preferences_frame();
    let metrics = current_host_metrics();
    let initial = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, 0.0, 12);
    let scrolled =
        SettingsWindowLayout::new(&frame, metrics, 0.0, 0, initial.setting_row_height, 12);

    assert_eq!(
        scrolled.setting_row_index_at(scrolled.setting_list.y, 12),
        Some(1)
    );
    assert_eq!(scrolled.setting_row(1).y, scrolled.setting_list.y);
}

#[test]
fn setting_scroll_is_clamped_to_the_current_content_extent() {
    let frame = ultra_preferences_frame();
    let metrics = current_host_metrics();
    let overflow = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, f32::MAX, 12);
    let no_overflow = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, f32::MAX, 1);

    assert_eq!(
        overflow.setting_scroll_offset(),
        overflow.max_setting_scroll_offset()
    );
    assert!(overflow.setting_scroll_offset() > 0.0);
    assert_eq!(no_overflow.setting_scroll_offset(), 0.0);
    assert_eq!(no_overflow.setting_scroll_offset_for_delta(100.0), 0.0);
}

#[test]
fn overflowing_settings_reserve_a_tokenized_scrollbar_and_move_the_thumb() {
    let frame = ultra_preferences_frame();
    let metrics = current_host_metrics();
    let top = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, 0.0, 12);
    let bottom = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, f32::MAX, 12);
    let track = top
        .setting_scrollbar_track
        .as_ref()
        .expect("overflowing settings need a visible scroll track");
    let top_thumb = top.setting_scrollbar_thumb.as_ref().unwrap();
    let bottom_thumb = bottom.setting_scrollbar_thumb.as_ref().unwrap();

    assert_eq!(track.width, metrics.scrollbar_thickness);
    assert!(top.setting_list.x + top.setting_list.width + metrics.gap_s <= track.x);
    assert_eq!(top_thumb.y, track.y);
    assert!(bottom_thumb.y > top_thumb.y);
    assert!(bottom_thumb.y + bottom_thumb.height <= track.y + track.height);

    let no_overflow = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, 0.0, 1);
    assert!(no_overflow.setting_scrollbar_track.is_none());
    assert!(no_overflow.setting_scrollbar_thumb.is_none());
}

#[test]
fn ultra_preferences_preserve_readable_label_and_value_columns() {
    let frame = ultra_preferences_frame();
    let metrics = current_host_metrics();
    let layout = SettingsWindowLayout::new(&frame, metrics, 0.0, 0, 0.0, 12);
    let value = layout.setting_value_control(0, true);

    assert!(layout.sidebar.width <= 120.0);
    assert!(layout.setting_text_width(0) >= 96.0);
    assert!(value.width >= metrics.control_default_height * 2.0);
    assert!(value.x >= layout.setting_list.x + layout.setting_text_width(0));
    assert!(value.x + value.width <= layout.setting_reset_control(0).x - metrics.gap_m);
}

#[test]
fn category_scroll_projects_rows_and_thumb_from_the_same_offset() {
    let frame = ultra_preferences_frame();
    let metrics = current_host_metrics();
    let initial = SettingsWindowLayout::new(&frame, metrics, 0.0, 20, 0.0, 0);
    let scrolled =
        SettingsWindowLayout::new(&frame, metrics, initial.category_row_height, 20, 0.0, 0);

    assert_eq!(
        scrolled.category_row_index_at(scrolled.category_list.y, 20),
        Some(1)
    );
    assert_eq!(scrolled.category_row(1).y, scrolled.category_list.y);
    assert!(scrolled.category_scrollbar_track.is_some());
    assert!(
        scrolled.category_scrollbar_thumb.as_ref().unwrap().y
            > initial.category_scrollbar_thumb.as_ref().unwrap().y
    );
}
