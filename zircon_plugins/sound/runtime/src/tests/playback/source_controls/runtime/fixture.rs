use super::super::super::super::*;

pub(super) struct RuntimeSourceControlFixture {
    pub(super) sound: DefaultSoundManager,
    pub(super) source: SoundSourceId,
}

impl RuntimeSourceControlFixture {
    pub(super) fn new() -> Self {
        let sound = DefaultSoundManager::default();
        sound
            .configure_output_device(SoundOutputDeviceDescriptor {
                id: SoundOutputDeviceId::new("sound.output.source_controls"),
                backend: "software-test".to_string(),
                display_name: "Source Controls Test Output".to_string(),
                sample_rate_hz: 10,
                channel_count: 2,
                channel_layout: SoundChannelLayout::stereo(),
                block_size_frames: 1,
                latency_blocks: 1,
            })
            .unwrap();
        let clip = sound.insert_clip_for_test(test_clip_with_rate(
            "res://sound/source-controls.wav",
            10,
            &[0.1, 0.2, 0.3, 0.4],
        ));
        let source = sound
            .create_source(SoundSourceDescriptor::clip(clip))
            .unwrap();

        Self { sound, source }
    }
}
