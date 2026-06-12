use super::super::super::super::*;

pub(super) fn software_test_descriptor(
    id: &str,
    display_name: &str,
    channel_layout: SoundChannelLayout,
    channel_count: u16,
    block_size_frames: u32,
) -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new(id),
        backend: "software-test".to_string(),
        display_name: display_name.to_string(),
        sample_rate_hz: 48_000,
        channel_count,
        channel_layout,
        block_size_frames,
        latency_blocks: 2,
    }
}
