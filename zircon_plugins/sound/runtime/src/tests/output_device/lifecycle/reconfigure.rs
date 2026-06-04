use super::super::super::*;

#[test]
fn output_device_updates_runtime_format_and_stops_on_reconfigure() {
    let sound = DefaultSoundManager::default();
    sound.start_output_device().unwrap();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.preview"),
            backend: "software-preview".to_string(),
            display_name: "Preview Output".to_string(),
            sample_rate_hz: 24_000,
            channel_count: 1,
            channel_layout: SoundChannelLayout::mono(),
            block_size_frames: 3,
            latency_blocks: 1,
        })
        .unwrap();

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.state, SoundOutputDeviceState::Stopped);
    assert_eq!(status.rendered_frames, 0);
    assert_eq!(sound.backend_status().sample_rate_hz, 24_000);
    assert_eq!(sound.backend_status().channel_count, 1);
    assert_eq!(
        sound.backend_status().channel_layout,
        SoundChannelLayout::mono()
    );
    let snapshot = sound.mixer_snapshot().unwrap();
    assert_eq!(snapshot.graph.sample_rate_hz, 24_000);
    assert_eq!(snapshot.graph.channel_count, 1);
    assert_eq!(snapshot.graph.channel_layout, SoundChannelLayout::mono());
}
