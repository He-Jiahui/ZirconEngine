use super::super::super::super::super::super::*;

pub(super) fn output_descriptor(
    id: &str,
    display_name: &str,
    backend: &str,
) -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new(id),
        backend: backend.to_string(),
        display_name: display_name.to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        channel_layout: SoundChannelLayout::stereo(),
        block_size_frames: 128,
        latency_blocks: 2,
    }
}
