use crate::service_types::{
    absolute_position_from_kira_slice, kira_slice_position_for_absolute_frame,
};

#[test]
fn sliced_playback_and_source_seek_convert_absolute_clip_frame_to_kira_relative_time() {
    assert_eq!(
        kira_slice_position_for_absolute_frame(72_000, 48_000, 48_000.0),
        0.5
    );
}

#[test]
fn sliced_playback_and_source_status_convert_kira_relative_time_to_absolute_clip_position() {
    let (frame, seconds) = absolute_position_from_kira_slice(0.5, 48_000, Some(96_000), 48_000);

    assert_eq!(frame, 72_000);
    assert_eq!(seconds, 1.5);
}
