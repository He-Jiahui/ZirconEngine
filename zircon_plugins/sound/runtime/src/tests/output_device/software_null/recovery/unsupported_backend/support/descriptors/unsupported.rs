use super::super::super::super::super::super::super::*;

pub(crate) fn unsupported_native_descriptor() -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.unsupported"),
        backend: "native-missing".to_string(),
        display_name: "Unsupported Native".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        channel_layout: AudioChannelLayout::stereo(),
        block_size_frames: 128,
        latency_blocks: 2,
    }
}
