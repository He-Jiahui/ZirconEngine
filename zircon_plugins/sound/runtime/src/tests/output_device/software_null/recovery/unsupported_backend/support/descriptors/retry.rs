use super::super::super::super::super::super::super::*;

pub(crate) fn software_null_retry_descriptor() -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.null.retry"),
        backend: "software-null".to_string(),
        display_name: "Software Null Retry Output".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        channel_layout: AudioChannelLayout::stereo(),
        block_size_frames: 128,
        latency_blocks: 2,
    }
}
