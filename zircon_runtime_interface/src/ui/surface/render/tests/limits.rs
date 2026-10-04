use super::*;

#[test]
fn shared_slider_tick_budget_bounds_declarations_and_track_columns() {
    assert_eq!(MAX_UI_SLIDER_TICK_COUNT, 256);
    assert_eq!(bounded_ui_slider_tick_count(f32::NAN), None);
    assert_eq!(bounded_ui_slider_tick_count(-1.0), None);
    assert_eq!(bounded_ui_slider_tick_count(1.0), None);
    assert_eq!(bounded_ui_slider_tick_count(2.0), Some(2));
    assert_eq!(
        bounded_ui_slider_tick_count(f32::INFINITY),
        Some(MAX_UI_SLIDER_TICK_COUNT)
    );
    assert_eq!(
        bounded_ui_slider_tick_count(f32::MAX),
        Some(MAX_UI_SLIDER_TICK_COUNT)
    );

    assert_eq!(ui_slider_tick_count_for_track(10_000, 24.9), 24);
    assert_eq!(
        ui_slider_tick_count_for_track(10_000, 512.0),
        MAX_UI_SLIDER_TICK_COUNT
    );
    assert_eq!(ui_slider_tick_count_for_track(10_000, f32::NAN), 0);
}
