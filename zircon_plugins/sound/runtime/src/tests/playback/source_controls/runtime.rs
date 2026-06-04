use super::super::super::*;

#[test]
fn source_runtime_controls_match_bevy_audio_sink_controls() {
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

    sound.pause_source(source).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert_eq!(sound.source_status(source).unwrap().cursor_frame, 0);
    assert!(!sound.source_status(source).unwrap().playing);

    sound.resume_source(source).unwrap();
    sound.seek_source_seconds(source, 0.2).unwrap();
    sound.set_source_gain(source, 2.0).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.6, 0.6]);
    assert_eq!(sound.source_status(source).unwrap().cursor_frame, 3);

    sound.mute_source(source).unwrap();
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    assert!(sound.source_status(source).unwrap().muted);

    sound.toggle_mute_source(source).unwrap();
    sound.set_source_speed(source, 0.5).unwrap();
    let status = sound.source_status(source).unwrap();
    assert_eq!(status.speed, 0.5);
    assert!(!status.muted);

    sound.toggle_source(source).unwrap();
    assert!(!sound.source_status(source).unwrap().playing);
    sound.toggle_source(source).unwrap();
    assert!(sound.source_status(source).unwrap().playing);

    assert!(sound.seek_source_seconds(source, -0.1).is_err());
    assert!(sound.set_source_gain(source, f32::NAN).is_err());
    assert!(sound.set_source_speed(source, 0.0).is_err());
    assert!(sound.unmute_source(SoundSourceId::new(999_999)).is_err());
}
