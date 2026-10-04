use super::*;

#[test]
fn rgba_channel_rows_share_one_bounded_popup_without_overlap() {
    let control = FrameRect {
        x: 180.0,
        y: 140.0,
        width: 180.0,
        height: 28.0,
    };
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 400.0,
        height: 300.0,
    };
    let popup = settings_color_popup_frame_within(&control, &bounds).unwrap();
    assert!(popup.x >= bounds.x);
    assert!(popup.y >= bounds.y);
    assert!(popup.x + popup.width <= bounds.x + bounds.width);
    assert!(popup.y + popup.height <= bounds.y + bounds.height);

    for channel in 0..SETTINGS_COLOR_CHANNEL_COUNT {
        let frames = settings_color_channel_frames_within(&control, channel, &bounds).unwrap();
        assert!(frames.label.x + frames.label.width <= frames.decrement.x);
        assert!(frames.decrement.x + frames.decrement.width <= frames.value.x);
        assert!(frames.value.x + frames.value.width <= frames.increment.x);
        assert!(frames.increment.x + frames.increment.width <= popup.x + popup.width);
    }
}

#[test]
fn narrow_channel_rows_never_overlap() {
    let control = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 28.0,
    };
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 200.0,
    };
    let frames = settings_color_channel_frames_within(&control, 0, &bounds).unwrap();

    assert!(frames.label.x + frames.label.width <= frames.decrement.x);
    assert!(frames.decrement.x + frames.decrement.width <= frames.value.x);
    assert!(frames.value.x + frames.value.width <= frames.increment.x);
    assert!(frames.increment.x + frames.increment.width <= 1.0);
}
