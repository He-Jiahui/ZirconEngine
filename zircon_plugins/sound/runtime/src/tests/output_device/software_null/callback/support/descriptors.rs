use super::super::super::super::super::*;

pub(super) fn software_null_descriptor() -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.null"),
        backend: "software-null".to_string(),
        display_name: "Software Null Output".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        channel_layout: SoundChannelLayout::stereo(),
        block_size_frames: 2,
        latency_blocks: 2,
    }
}
