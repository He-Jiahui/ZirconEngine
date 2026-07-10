use super::super::super::super::*;

#[test]
fn external_audio_source_block_rejects_non_canonical_speaker_metadata() {
    let sound = DefaultSoundManager::default();
    let handle = ExternalAudioSourceHandle::new("navigation.surface-noise");

    assert!(sound
        .submit_external_source_block(
            handle,
            SoundExternalSourceBlock {
                sample_rate_hz: 48_000,
                channel_count: 2,
                channel_layout: AudioChannelLayout {
                    name: "stereo".to_string(),
                    channel_count: 2,
                    speakers: vec![
                        AudioSpeakerChannel::FrontRight,
                        AudioSpeakerChannel::FrontLeft,
                    ],
                },
                samples: vec![0.0, 0.0],
            },
        )
        .unwrap_err()
        .to_string()
        .contains("canonical speaker metadata"));
}
