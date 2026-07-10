use super::super::super::super::super::*;

pub(super) fn test_output_descriptor() -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.test"),
        backend: "software-test".to_string(),
        display_name: "Software Test Output".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        channel_layout: AudioChannelLayout::stereo(),
        block_size_frames: 2,
        latency_blocks: 2,
    }
}
