use super::{timeline_tick_label_frame, TimelineStripMetrics};
use crate::ui::retained_host::host_contract::{
    data::FrameRect, paint_text::measure_runtime_text_width,
};

fn metrics() -> TimelineStripMetrics {
    TimelineStripMetrics {
        outer_radius: 2.0,
        border_width: 1.0,
        inset: 2.0,
        ruler_height: 20.0,
        track_height: 20.0,
        footer_height: 20.0,
        font_size: 12.0,
        line_height: 16.0,
        key_radius: 3.0,
        playhead_width: 2.0,
    }
}

#[test]
fn timeline_tick_label_frame_measures_and_clamps_to_neighbor_interval() {
    let ruler = FrameRect {
        x: 0.0,
        y: 4.0,
        width: 120.0,
        height: 20.0,
    };
    let label = "WWWWWWWWWWWW";
    let frame = timeline_tick_label_frame(&ruler, 60.0, Some(0.0), Some(120.0), label, metrics());

    let available_width = 60.0;
    let expected_width = measure_runtime_text_width(label, 12.0)
        .ceil()
        .max(0.0)
        .min(available_width);
    assert_eq!(frame.width, expected_width);
    assert!(frame.x >= 30.0);
    assert!(frame.x + frame.width <= 90.0);
}

#[test]
fn collapsed_or_invalid_rulers_do_not_produce_tick_label_frames() {
    for ruler in [
        FrameRect {
            x: 0.0,
            y: 4.0,
            width: 0.0,
            height: 20.0,
        },
        FrameRect {
            x: f32::NAN,
            y: 4.0,
            width: 120.0,
            height: 20.0,
        },
    ] {
        let frame = timeline_tick_label_frame(&ruler, 60.0, None, None, "1.0", metrics());
        assert_eq!(frame.width, 0.0);
        assert_eq!(frame.height, 0.0);
    }
}
