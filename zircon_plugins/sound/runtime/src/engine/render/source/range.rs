use zircon_runtime::core::framework::sound::SoundSourceDescriptor;

pub(super) fn source_clip_range(
    descriptor: &SoundSourceDescriptor,
    sample_rate_hz: u32,
    frame_count: usize,
) -> (usize, Option<usize>) {
    let sample_rate = sample_rate_hz.max(1) as f32;
    let start_frame = descriptor
        .start_seconds
        .map(|seconds| (seconds * sample_rate).round().max(0.0) as usize)
        .unwrap_or_default()
        .min(frame_count);
    let end_frame = descriptor.duration_seconds.map(|seconds| {
        let duration_frames = (seconds * sample_rate).round().max(0.0) as usize;
        start_frame.saturating_add(duration_frames).min(frame_count)
    });
    (start_frame, end_frame)
}
