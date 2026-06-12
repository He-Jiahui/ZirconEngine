use super::super::super::super::super::*;

pub(super) fn cpal_windows_default_output() -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.cpal.windows"),
        backend: "cpal".to_string(),
        display_name: "CPAL Windows Default Output".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        channel_layout: SoundChannelLayout::stereo(),
        block_size_frames: 128,
        latency_blocks: 2,
    }
}
