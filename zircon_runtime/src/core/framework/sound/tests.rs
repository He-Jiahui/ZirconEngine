use super::*;

#[test]
fn default_sound_plugin_options_match_runtime_contract() {
    let options = SoundPluginOptions::default();

    assert!(options.enabled);
    assert_eq!(options.backend, "software-mixer");
    assert_eq!(options.sample_rate_hz, 48_000);
    assert_eq!(options.channel_count, 2);
    assert_eq!(options.channel_layout, SoundChannelLayout::stereo());
    assert_eq!(options.global_volume_gain, 1.0);
    assert_eq!(options.block_size_frames, 256);
    assert_eq!(options.max_voices, 128);
    assert_eq!(options.max_tracks, 64);
    assert_eq!(options.default_spatial_scale, 1.0);
    assert_eq!(options.hrtf_profile, "default");
    assert_eq!(options.default_mixer_preset, "sound://mixer/default");
    assert!(options.convolution_enabled);
    assert!(options.timeline_integration);
    assert!(options.dynamic_events_enabled);
    assert_eq!(
        options.ray_tracing_quality,
        SoundRayTracingQuality::Disabled
    );
}

#[test]
fn default_stereo_mixer_graph_keeps_master_track_and_event_namespace() {
    let graph = SoundMixerGraph::default_stereo(48_000);
    let master = graph.master_track().expect("default master track");

    assert_eq!(graph.sample_rate_hz, 48_000);
    assert_eq!(graph.channel_count, 2);
    assert_eq!(graph.channel_layout, SoundChannelLayout::stereo());
    assert_eq!(graph.tracks.len(), 1);
    assert_eq!(master.id, SoundTrackId::master());
    assert_eq!(master.display_name, "Master");
    assert_eq!(master.controls, SoundTrackControls::default());
    assert_eq!(graph.dynamic_events.namespace, "sound.dynamic_events");
    assert_eq!(graph.dynamic_events.version, 1);
    assert!(graph.dynamic_events.events.is_empty());

    let json = serde_json::to_value(&graph).unwrap();
    assert_eq!(
        serde_json::from_value::<SoundMixerGraph>(json).unwrap(),
        graph
    );
}

#[test]
fn sound_channel_layouts_name_speaker_order_for_multichannel_formats() {
    let mono = SoundChannelLayout::mono();
    let stereo = SoundChannelLayout::stereo();
    let surround = SoundChannelLayout::surround_5_1();

    assert_eq!(mono.channel_count, 1);
    assert_eq!(stereo.channel_count, 2);
    assert_eq!(surround.channel_count, 6);
    assert_eq!(
        surround.speakers,
        vec![
            SoundSpeakerChannel::FrontLeft,
            SoundSpeakerChannel::FrontRight,
            SoundSpeakerChannel::FrontCenter,
            SoundSpeakerChannel::LowFrequency,
            SoundSpeakerChannel::BackLeft,
            SoundSpeakerChannel::BackRight,
        ]
    );
    assert_eq!(
        SoundChannelLayout::for_channel_count(8),
        SoundChannelLayout::surround_7_1()
    );
    assert!(SoundChannelLayout::discrete(12).matches_channel_count(12));
    assert!(!SoundChannelLayout::discrete(12).matches_channel_count(2));
    assert!(!SoundChannelLayout::surround_5_1().matches_channel_count(2));
    assert_eq!(
        serde_json::to_value(SoundSpeakerChannel::LowFrequency).unwrap(),
        "low_frequency"
    );
}

#[test]
fn clip_source_defaults_to_master_track_and_neutral_spatial_contract() {
    let clip = SoundClipId::new(7);
    let source = SoundSourceDescriptor::clip(clip);
    let block = SoundMixBlock::silent(48_000, 6, 4);

    assert_eq!(source.input, SoundSourceInput::Clip(clip));
    assert_eq!(source.output_track, SoundTrackId::master());
    assert_eq!(source.gain, 1.0);
    assert_eq!(source.speed, 1.0);
    assert!(source.playing);
    assert!(!source.looped);
    assert!(!source.muted);
    assert_eq!(
        source.completion_action,
        SoundPlaybackCompletionAction::None
    );
    assert_eq!(source.spatial.spatial_blend, 0.0);
    assert_eq!(source.spatial.min_distance, 1.0);
    assert_eq!(source.spatial.max_distance, 50.0);
    assert_eq!(
        source.spatial.attenuation,
        SoundAttenuationMode::InverseDistance
    );
    assert_eq!(source.spatial.convolution_send, None);
    assert_eq!(block.channel_layout, SoundChannelLayout::surround_5_1());
    assert_eq!(block.samples.len(), 24);
}

#[test]
fn sound_scene_component_type_ids_are_plugin_prefixed() {
    for type_id in [
        AUDIO_SOURCE_COMPONENT_TYPE,
        AUDIO_LISTENER_COMPONENT_TYPE,
        AUDIO_VOLUME_COMPONENT_TYPE,
    ] {
        assert!(type_id.starts_with("sound.Component."));
    }
}
