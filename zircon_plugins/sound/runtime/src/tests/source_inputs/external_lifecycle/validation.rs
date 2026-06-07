use super::super::super::*;

#[test]
fn external_audio_source_lifecycle_rejects_invalid_handles_and_blocks() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("navigation.surface-noise");
    let empty_handle = ExternalAudioSourceHandle::new(" ");

    assert!(sound
        .submit_external_source_block(
            empty_handle.clone(),
            SoundExternalSourceBlock::new(48_000, SoundChannelLayout::mono(), vec![0.0]),
        )
        .unwrap_err()
        .to_string()
        .contains("external source handle"));
    assert!(sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 0,
                channel_layout: SoundChannelLayout::mono(),
                samples: vec![0.0],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("channel count"));
    assert!(sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 2,
                channel_layout: SoundChannelLayout::mono(),
                samples: vec![0.0, 0.0],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("channel layout"));
    assert!(sound
        .submit_external_source_block(
            handle.clone(),
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 2,
                channel_layout: SoundChannelLayout {
                    name: "stereo".to_string(),
                    channel_count: 2,
                    speakers: vec![
                        SoundSpeakerChannel::FrontRight,
                        SoundSpeakerChannel::FrontLeft,
                    ],
                },
                samples: vec![0.0, 0.0],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("canonical speaker metadata"));
    assert!(sound
        .submit_external_source_block(
            handle,
            SoundExternalSourceBlock::new(48_000, SoundChannelLayout::mono(), vec![f32::NAN]),
        )
        .unwrap_err()
        .to_string()
        .contains("finite"));
    assert!(sound
        .create_source(SoundSourceDescriptor {
            input: SoundSourceInput::External(empty_handle),
            ..SoundSourceDescriptor::clip(SoundClipId::new(999))
        })
        .unwrap_err()
        .to_string()
        .contains("external source handle"));
}
