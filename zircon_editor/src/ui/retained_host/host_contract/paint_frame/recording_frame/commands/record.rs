use super::super::super::super::data::FrameRect;
use super::super::super::recording::HostRecordedPaintKind;
use super::super::super::HostRgbaFrame;

pub(super) fn record_command(
    frame_target: &mut HostRgbaFrame,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    kind: HostRecordedPaintKind,
) {
    if let Some(recording) = frame_target.recording.as_mut() {
        recording.record_command(frame, clip_frame, kind);
    }
}
