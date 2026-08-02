use crate::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};

use super::*;

#[test]
fn default_sound_plugin_options_match_runtime_contract() {
    let options = SoundPluginOptions::default();

    assert!(options.enabled);
    assert_eq!(options.backend, "kira-cpal");
    assert_eq!(options.sample_rate_hz, 48_000);
    assert_eq!(options.channel_count, 2);
    assert_eq!(options.channel_layout, AudioChannelLayout::stereo());
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
    assert_eq!(graph.channel_layout, AudioChannelLayout::stereo());
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
    let mono = AudioChannelLayout::mono();
    let stereo = AudioChannelLayout::stereo();
    let quad = AudioChannelLayout::quad();
    let surround_5_0 = AudioChannelLayout::surround_5_0();
    let surround = AudioChannelLayout::surround_5_1();
    let surround_side = AudioChannelLayout::surround_5_1_side();
    let surround_7_0 = AudioChannelLayout::surround_7_0();

    assert_eq!(mono.channel_count, 1);
    assert_eq!(stereo.channel_count, 2);
    assert_eq!(quad.channel_count, 4);
    assert_eq!(surround_5_0.channel_count, 5);
    assert_eq!(surround.channel_count, 6);
    assert_eq!(surround_side.channel_count, 6);
    assert_eq!(surround_7_0.channel_count, 7);
    assert_eq!(
        quad.speakers,
        vec![
            AudioSpeakerChannel::FrontLeft,
            AudioSpeakerChannel::FrontRight,
            AudioSpeakerChannel::BackLeft,
            AudioSpeakerChannel::BackRight,
        ]
    );
    assert_eq!(
        surround_5_0.speakers,
        vec![
            AudioSpeakerChannel::FrontLeft,
            AudioSpeakerChannel::FrontRight,
            AudioSpeakerChannel::FrontCenter,
            AudioSpeakerChannel::BackLeft,
            AudioSpeakerChannel::BackRight,
        ]
    );
    assert_eq!(
        surround.speakers,
        vec![
            AudioSpeakerChannel::FrontLeft,
            AudioSpeakerChannel::FrontRight,
            AudioSpeakerChannel::FrontCenter,
            AudioSpeakerChannel::LowFrequency,
            AudioSpeakerChannel::BackLeft,
            AudioSpeakerChannel::BackRight,
        ]
    );
    assert_eq!(
        surround_side.speakers,
        vec![
            AudioSpeakerChannel::FrontLeft,
            AudioSpeakerChannel::FrontRight,
            AudioSpeakerChannel::FrontCenter,
            AudioSpeakerChannel::LowFrequency,
            AudioSpeakerChannel::SideLeft,
            AudioSpeakerChannel::SideRight,
        ]
    );
    assert_eq!(
        surround_7_0.speakers,
        vec![
            AudioSpeakerChannel::FrontLeft,
            AudioSpeakerChannel::FrontRight,
            AudioSpeakerChannel::FrontCenter,
            AudioSpeakerChannel::BackLeft,
            AudioSpeakerChannel::BackRight,
            AudioSpeakerChannel::SideLeft,
            AudioSpeakerChannel::SideRight,
        ]
    );
    assert_eq!(
        AudioChannelLayout::for_channel_count(4),
        AudioChannelLayout::quad()
    );
    assert_eq!(
        AudioChannelLayout::for_channel_count(5),
        AudioChannelLayout::surround_5_0()
    );
    assert_eq!(
        AudioChannelLayout::for_channel_count(7),
        AudioChannelLayout::surround_7_0()
    );
    assert_eq!(
        AudioChannelLayout::for_channel_count(8),
        AudioChannelLayout::surround_7_1()
    );
    assert_eq!(
        AudioChannelLayout::named_layout_names(),
        &[
            "mono",
            "stereo",
            "quad",
            "surround_5_0",
            "surround_5_1",
            "surround_5_1_side",
            "surround_7_0",
            "surround_7_1",
        ]
    );
    for layout_name in AudioChannelLayout::named_layout_names() {
        let layout = AudioChannelLayout::from_name(layout_name).expect("known layout name");
        assert_eq!(layout.name, *layout_name);
    }
    assert_eq!(AudioChannelLayout::from_name("discrete_12"), None);
    assert_eq!(AudioChannelLayout::from_name("surround_6_1"), None);
    assert!(AudioChannelLayout::discrete(12).matches_channel_count(12));
    assert!(!AudioChannelLayout::discrete(12).matches_channel_count(2));
    assert!(!AudioChannelLayout::surround_5_1().matches_channel_count(2));
    assert_eq!(
        serde_json::to_value(AudioSpeakerChannel::LowFrequency).unwrap(),
        "low_frequency"
    );
}

#[test]
fn sound_channel_layout_contract_rejects_ambiguous_speaker_metadata() {
    let mut reordered_stereo = AudioChannelLayout::stereo();
    reordered_stereo.speakers.reverse();

    assert!(!reordered_stereo.is_canonical_named_layout());
    assert!(!reordered_stereo.is_valid_contract_layout());

    let duplicate_custom = AudioChannelLayout {
        name: "custom_duplicate_front".to_string(),
        channel_count: 2,
        speakers: vec![
            AudioSpeakerChannel::FrontLeft,
            AudioSpeakerChannel::FrontLeft,
        ],
    };
    assert!(duplicate_custom.has_matching_speaker_count());
    assert!(!duplicate_custom.has_unique_speakers());
    assert!(!duplicate_custom.is_valid_contract_layout());

    let custom_named_speakers = AudioChannelLayout {
        name: "custom_front_lfe".to_string(),
        channel_count: 2,
        speakers: vec![
            AudioSpeakerChannel::FrontLeft,
            AudioSpeakerChannel::LowFrequency,
        ],
    };
    assert!(custom_named_speakers.is_valid_contract_layout());
    assert!(AudioChannelLayout::discrete(12).is_valid_contract_layout());

    let invalid_discrete = AudioChannelLayout {
        name: "speakerless_custom".to_string(),
        channel_count: 12,
        speakers: Vec::new(),
    };
    assert!(!invalid_discrete.is_valid_contract_layout());

    let discrete_with_named_speakers = AudioChannelLayout {
        name: "discrete_2".to_string(),
        channel_count: 2,
        speakers: vec![
            AudioSpeakerChannel::FrontLeft,
            AudioSpeakerChannel::FrontRight,
        ],
    };
    assert!(!discrete_with_named_speakers.is_valid_contract_layout());

    for reserved_name in [
        "discrete_",
        "discrete_two",
        "discrete_02",
        "discrete_65536",
        "discrete_-1",
    ] {
        let malformed_discrete = AudioChannelLayout {
            name: reserved_name.to_string(),
            channel_count: 2,
            speakers: vec![
                AudioSpeakerChannel::FrontLeft,
                AudioSpeakerChannel::FrontRight,
            ],
        };
        assert!(
            !malformed_discrete.is_valid_contract_layout(),
            "reserved discrete name must not degrade to a custom layout: {reserved_name}"
        );
    }
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
    assert_eq!(block.channel_layout, AudioChannelLayout::surround_5_1());
    assert_eq!(block.samples.len(), 24);
}

#[test]
fn external_source_block_carries_explicit_channel_layout_contract() {
    let block = SoundExternalSourceBlock {
        sample_rate_hz: 48_000,
        channel_count: 6,
        channel_layout: AudioChannelLayout::surround_5_1_side(),
        samples: vec![0.0; 12],
    };

    assert_eq!(
        block.channel_layout,
        AudioChannelLayout::surround_5_1_side()
    );
    assert!(block
        .channel_layout
        .matches_channel_count(block.channel_count));
    assert_eq!(
        serde_json::from_value::<SoundExternalSourceBlock>(serde_json::to_value(&block).unwrap())
            .unwrap(),
        block
    );
}

#[test]
fn external_source_block_constructor_derives_count_from_declared_layout() {
    let block = SoundExternalSourceBlock::new(
        48_000,
        AudioChannelLayout::surround_5_1_side(),
        vec![0.0; 12],
    );

    assert_eq!(block.channel_count, 6);
    assert_eq!(
        block.channel_layout,
        AudioChannelLayout::surround_5_1_side()
    );
    assert!(block
        .channel_layout
        .matches_channel_count(block.channel_count));
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
