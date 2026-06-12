use super::super::super::super::super::*;

pub(super) fn preview_output_descriptor() -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.preview"),
        backend: "software-preview".to_string(),
        display_name: "Preview Output".to_string(),
        sample_rate_hz: 24_000,
        channel_count: 1,
        channel_layout: SoundChannelLayout::mono(),
        block_size_frames: 3,
        latency_blocks: 1,
    }
}
