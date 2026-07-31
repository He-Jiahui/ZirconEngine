pub const MAX_UI_SLIDER_TICK_COUNT: usize = 256;

pub fn bounded_ui_slider_tick_count(declared: f32) -> Option<usize> {
    let rounded = declared.round();
    if rounded.is_nan() || rounded < 2.0 {
        return None;
    }
    if rounded.is_infinite() || rounded >= MAX_UI_SLIDER_TICK_COUNT as f32 {
        return Some(MAX_UI_SLIDER_TICK_COUNT);
    }
    Some(rounded as usize)
}

pub fn ui_slider_tick_count_for_track(declared: usize, track_width: f32) -> usize {
    let track_columns = if track_width.is_nan() || track_width <= 0.0 {
        0
    } else if track_width.is_infinite() {
        MAX_UI_SLIDER_TICK_COUNT
    } else {
        track_width.floor().min(MAX_UI_SLIDER_TICK_COUNT as f32) as usize
    };
    declared.min(MAX_UI_SLIDER_TICK_COUNT).min(track_columns)
}

#[cfg(test)]
mod tests {
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
}
