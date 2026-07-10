use super::super::*;

#[test]
fn source_start_and_duration_limit_clip_playback_range() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.source_range"),
            backend: "software-test".to_string(),
            display_name: "Source Range Test Output".to_string(),
            sample_rate_hz: 10,
            channel_count: 2,
            channel_layout: AudioChannelLayout::stereo(),
            block_size_frames: 3,
            latency_blocks: 1,
        })
        .unwrap();
    let clip = sound.insert_clip_for_test(test_clip_with_rate(
        "res://sound/source-range.wav",
        10,
        &[0.1, 0.2, 0.3, 0.4],
    ));

    let mut source = SoundSourceDescriptor::clip(clip);
    source.looped = true;
    source.start_seconds = Some(0.1);
    source.duration_seconds = Some(0.2);
    let source_id = sound.create_source(source).unwrap();

    let status = sound.source_status(source_id).unwrap();
    assert_eq!(status.range_start_frame, 1);
    assert_eq!(status.range_end_frame, Some(3));
    assert_eq!(status.cursor_frame, 0);
    assert!(status.looped);

    assert_eq!(
        sound.render_mix(3).unwrap().samples,
        vec![0.2, 0.2, 0.3, 0.3, 0.2, 0.2]
    );
    assert_eq!(sound.source_status(source_id).unwrap().cursor_frame, 2);

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.start_seconds = Some(-0.1);
    assert!(sound.create_source(invalid).is_err());

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.duration_seconds = Some(0.0);
    assert!(sound.create_source(invalid).is_err());

    let mut invalid = SoundSourceDescriptor::clip(clip);
    invalid.duration_seconds = Some(f32::NAN);
    assert!(sound.create_source(invalid).is_err());
}
