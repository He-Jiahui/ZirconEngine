use zircon_runtime::asset::{AssetUri, SoundAsset};
use zircon_runtime::core::framework::sound::{
    SoundAttenuationMode, SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationTarget,
    SoundChorusEffect, SoundClipId, SoundCompressorEffect, SoundConvolutionReverbEffect,
    SoundDelayEffect, SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundFilterEffect,
    SoundFilterMode, SoundFlangerEffect, SoundGainEffect, SoundImpulseResponseId,
    SoundLimiterEffect, SoundListenerDescriptor, SoundListenerId, SoundManager,
    SoundPanStereoEffect, SoundParameterId, SoundPhaserEffect, SoundPlaybackSettings,
    SoundReverbEffect, SoundSidechainInput, SoundSourceDescriptor, SoundSourceInput,
    SoundSourceSend, SoundSpatialSourceSettings, SoundTrackDescriptor, SoundTrackId,
    SoundTrackSend, SoundVolumeDescriptor, SoundVolumeId, SoundVolumeShape, SoundWaveShaperEffect,
    AUDIO_LISTENER_COMPONENT_TYPE, AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};
use zircon_runtime::plugin::RuntimePluginRegistrationReport;

use super::{
    runtime_plugin, DefaultSoundManager, SOUND_DYNAMIC_EVENT_NAMESPACE, SOUND_MODULE_NAME,
};

#[test]
fn sound_plugin_registration_contributes_runtime_module_components_options_and_events() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == SOUND_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ]
    );
    for component in [
        AUDIO_SOURCE_COMPONENT_TYPE,
        AUDIO_LISTENER_COMPONENT_TYPE,
        AUDIO_VOLUME_COMPONENT_TYPE,
    ] {
        assert!(report
            .extensions
            .components()
            .iter()
            .any(|descriptor| descriptor.type_id == component));
        assert!(report
            .package_manifest
            .components
            .iter()
            .any(|descriptor| descriptor.type_id == component));
    }
    assert!(report
        .extensions
        .plugin_options()
        .iter()
        .any(|option| option.key == "sound.ray_tracing_quality"));
    assert!(report
        .package_manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.id == "timeline_sequence" && !dependency.required));
    assert!(report
        .extensions
        .plugin_event_catalogs()
        .iter()
        .any(|catalog| {
            catalog.namespace == SOUND_DYNAMIC_EVENT_NAMESPACE && catalog.events.is_empty()
        }));
}

#[test]
fn default_sound_manager_renders_silence_without_active_playback() {
    let sound = DefaultSoundManager::default();
    let mix = sound.render_mix(3).unwrap();

    assert_eq!(mix.sample_rate_hz, 48_000);
    assert_eq!(mix.channel_count, 2);
    assert_eq!(mix.samples, vec![0.0; 6]);
}

#[test]
fn mixer_graph_routes_custom_track_through_effect_chain_to_master() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/tone.wav", &[1.0, 1.0]));
    let music = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(music, "Music"))
        .unwrap();
    sound
        .add_or_update_effect(
            music,
            SoundEffectDescriptor::new(
                SoundEffectId::new(1),
                "Music Gain",
                SoundEffectKind::Gain(SoundGainEffect { gain: 0.5 }),
            ),
        )
        .unwrap();
    sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: music,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();

    let mix = sound.render_mix(2).unwrap();

    assert_eq!(mix.samples, vec![0.5, 0.5, 0.5, 0.5]);
    assert!(sound
        .mixer_snapshot()
        .unwrap()
        .meters
        .iter()
        .any(|meter| meter.track == music && meter.peak_left == 0.5));
}

#[test]
fn mixer_graph_rejects_parent_cycles_and_missing_tracks() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/cycle.wav", &[1.0]));
    let a = SoundTrackId::new(2);
    let b = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(a, "A"))
        .unwrap();
    let mut b_track = SoundTrackDescriptor::child(b, "B");
    b_track.parent = Some(a);
    sound.add_or_update_track(b_track).unwrap();
    let mut a_cycle = SoundTrackDescriptor::child(a, "A");
    a_cycle.parent = Some(b);

    let error = sound.add_or_update_track(a_cycle).unwrap_err();
    assert!(error.to_string().contains("cycle"));

    let missing = sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: SoundTrackId::new(99),
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap_err();
    assert!(missing.to_string().contains("unknown track"));
}

#[test]
fn track_send_crud_routes_audio_and_reports_missing_targets() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/send.wav", &[0.5]));
    let music = SoundTrackId::new(2);
    let aux = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(music, "Music"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(aux, "Aux"))
        .unwrap();
    sound
        .add_or_update_track_send(
            music,
            SoundTrackSend {
                target: aux,
                gain: 0.25,
                pre_effects: false,
            },
        )
        .unwrap();
    sound
        .add_or_update_track_send(
            music,
            SoundTrackSend {
                target: aux,
                gain: 0.5,
                pre_effects: false,
            },
        )
        .unwrap();

    let snapshot = sound.mixer_snapshot().unwrap();
    let music_track = snapshot
        .graph
        .tracks
        .iter()
        .find(|track| track.id == music)
        .unwrap();
    assert_eq!(music_track.sends.len(), 1);
    assert_sample_near(music_track.sends[0].gain, 0.5);

    sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: music,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    let mix = sound.render_mix(1).unwrap();
    assert_eq!(mix.samples, vec![0.75, 0.75]);

    sound.remove_track_send(music, aux).unwrap();
    assert!(sound
        .remove_track_send(music, aux)
        .unwrap_err()
        .to_string()
        .contains("unknown send"));
    assert!(sound
        .add_or_update_track_send(
            music,
            SoundTrackSend {
                target: SoundTrackId::new(99),
                gain: 1.0,
                pre_effects: false,
            },
        )
        .unwrap_err()
        .to_string()
        .contains("unknown track"));
}

#[test]
fn mixer_graph_rejects_track_send_cycles() {
    let sound = DefaultSoundManager::default();
    let a = SoundTrackId::new(2);
    let b = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(a, "A"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(b, "B"))
        .unwrap();
    sound
        .add_or_update_track_send(
            a,
            SoundTrackSend {
                target: b,
                gain: 1.0,
                pre_effects: false,
            },
        )
        .unwrap();

    let error = sound
        .add_or_update_track_send(
            b,
            SoundTrackSend {
                target: a,
                gain: 1.0,
                pre_effects: false,
            },
        )
        .unwrap_err();

    assert!(error.to_string().contains("cycle"));
}

#[test]
fn track_solo_mutes_non_solo_direct_inputs_but_keeps_route_to_master() {
    let sound = DefaultSoundManager::default();
    let solo_clip = sound.insert_clip_for_test(test_clip("res://sound/solo.wav", &[0.5]));
    let muted_clip = sound.insert_clip_for_test(test_clip("res://sound/non-solo.wav", &[0.5]));
    let master_clip = sound.insert_clip_for_test(test_clip("res://sound/master.wav", &[0.25]));
    let solo = SoundTrackId::new(2);
    let muted = SoundTrackId::new(3);
    let mut solo_track = SoundTrackDescriptor::child(solo, "Solo");
    solo_track.controls.solo = true;
    sound.add_or_update_track(solo_track).unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(muted, "Muted"))
        .unwrap();
    sound
        .play_clip(
            solo_clip,
            SoundPlaybackSettings {
                output_track: solo,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    sound
        .play_clip(
            muted_clip,
            SoundPlaybackSettings {
                output_track: muted,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    sound
        .play_clip(master_clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_eq!(mix.samples, vec![0.5, 0.5]);
}

#[test]
fn sidechain_compressor_ducks_target_track_from_another_track() {
    let sound = DefaultSoundManager::default();
    let target_clip = sound.insert_clip_for_test(test_clip("res://sound/pad.wav", &[0.5, 0.5]));
    let key_clip = sound.insert_clip_for_test(test_clip("res://sound/kick.wav", &[0.5, 0.5]));
    let target = SoundTrackId::new(2);
    let key = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(target, "Pad"))
        .unwrap();
    sound
        .add_or_update_track(SoundTrackDescriptor::child(key, "Kick Sidechain"))
        .unwrap();
    sound
        .add_or_update_effect(
            target,
            SoundEffectDescriptor::new(
                SoundEffectId::new(2),
                "Sidechain Compressor",
                SoundEffectKind::Compressor(SoundCompressorEffect {
                    threshold_db: -18.0,
                    ratio: 8.0,
                    attack_ms: 1.0,
                    release_ms: 50.0,
                    makeup_gain_db: 0.0,
                    sidechain: Some(
                        zircon_runtime::core::framework::sound::SoundSidechainInput {
                            track: key,
                            pre_effects: true,
                        },
                    ),
                }),
            ),
        )
        .unwrap();
    sound
        .play_clip(
            target_clip,
            SoundPlaybackSettings {
                output_track: target,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    sound
        .play_clip(
            key_clip,
            SoundPlaybackSettings {
                output_track: key,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert!(mix.samples[0] > 0.5);
    assert!(mix.samples[0] < 1.0);
}

#[test]
fn sidechain_compressor_respects_pre_and_post_effect_taps() {
    let pre_effect_mix = render_sidechain_tap_mix(true);
    let post_effect_mix = render_sidechain_tap_mix(false);

    assert!(pre_effect_mix[0] < 0.5);
    assert_sample_near(post_effect_mix[0], 0.5);
}

#[test]
fn static_convolution_impulse_response_processes_master_track() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/impulse.wav", &[1.0, 0.0]));
    let impulse_response = SoundImpulseResponseId::new(1);
    sound
        .set_impulse_response(impulse_response, vec![0.5, 0.25])
        .unwrap();
    sound
        .add_or_update_effect(
            SoundTrackId::master(),
            SoundEffectDescriptor::new(
                SoundEffectId::new(3),
                "Static IR",
                SoundEffectKind::ConvolutionReverb(SoundConvolutionReverbEffect {
                    impulse_response,
                    fallback_to_algorithmic: true,
                    latency_frames: 1,
                }),
            ),
        )
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();

    let mix = sound.render_mix(2).unwrap();

    assert_eq!(mix.samples, vec![0.5, 0.5, 0.25, 0.25]);
}

#[test]
fn dsp_bypass_wet_dry_delay_pan_phase_and_limiter_are_deterministic() {
    let mut wet_gain = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }));
    wet_gain.wet = 0.25;
    assert_samples_near(&render_master_effect(wet_gain, &[1.0], 1), &[0.75, 0.75]);

    let mut bypass = test_effect(SoundEffectKind::Gain(SoundGainEffect { gain: 0.0 }));
    bypass.bypass = true;
    assert_samples_near(&render_master_effect(bypass, &[1.0], 1), &[1.0, 1.0]);

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Delay(SoundDelayEffect {
                delay_frames: 1,
                feedback: 0.0,
            })),
            &[0.5, 0.0],
            2,
        ),
        &[0.0, 0.0, 0.5, 0.5],
    );

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::PanStereo(SoundPanStereoEffect {
                pan: 1.0,
                width: 1.0,
                left_gain: 1.0,
                right_gain: 1.0,
                invert_left_phase: true,
                invert_right_phase: false,
            })),
            &[0.5],
            1,
        ),
        &[-0.0, 0.5],
    );

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Limiter(SoundLimiterEffect {
                ceiling: 0.25,
            })),
            &[0.75],
            1,
        ),
        &[0.25, 0.25],
    );
}

#[test]
fn dsp_filter_reverb_waveshaper_and_modulation_effects_are_deterministic() {
    let low_pass = render_master_effect(
        test_effect(SoundEffectKind::Filter(SoundFilterEffect {
            mode: SoundFilterMode::LowPass,
            cutoff_hz: 1_000.0,
            resonance: 0.0,
            gain_db: 0.0,
        })),
        &[1.0, 0.0],
        2,
    );
    assert!(low_pass[0] > 0.0 && low_pass[0] < 0.2);
    assert!(low_pass[2] > 0.0 && low_pass[2] < low_pass[0]);

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Reverb(SoundReverbEffect {
                room_size: 0.5,
                damping: 0.5,
                pre_delay_frames: 1,
                tail_frames: 2,
            })),
            &[0.5, 0.0],
            2,
        ),
        &[0.5, 0.5, 0.375, 0.375],
    );

    let shaped = render_master_effect(
        test_effect(SoundEffectKind::WaveShaper(SoundWaveShaperEffect {
            drive: 2.0,
        })),
        &[0.5],
        1,
    );
    assert!(shaped[0] > 0.5 && shaped[0] <= 1.0);

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Flanger(SoundFlangerEffect {
                delay_frames: 0,
                depth_frames: 0,
                rate_hz: 0.0,
                feedback: 0.0,
            })),
            &[0.25],
            1,
        ),
        &[0.375, 0.375],
    );

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Phaser(SoundPhaserEffect {
                rate_hz: 0.0,
                depth: 1.0,
                feedback: 0.0,
                phase_offset: 0.25,
            })),
            &[0.5],
            1,
        ),
        &[0.0, 0.0],
    );

    assert_samples_near(
        &render_master_effect(
            test_effect(SoundEffectKind::Chorus(SoundChorusEffect {
                voices: 1,
                delay_frames: 0,
                depth_frames: 0,
                rate_hz: 0.0,
            })),
            &[0.25],
            1,
        ),
        &[0.375, 0.375],
    );
}

#[test]
fn synth_parameter_source_and_timeline_binding_are_visible_in_snapshot() {
    let sound = DefaultSoundManager::default();
    let parameter = SoundParameterId::new("synth.cutoff");
    assert!(sound
        .parameter_value(&parameter)
        .unwrap_err()
        .to_string()
        .contains("unknown sound parameter"));
    sound.set_parameter(parameter.clone(), 0.25).unwrap();
    assert_sample_near(sound.parameter_value(&parameter).unwrap(), 0.25);
    let source = SoundSourceDescriptor {
        input: SoundSourceInput::SynthParameter {
            parameter: parameter.clone(),
            default_value: 0.0,
        },
        ..SoundSourceDescriptor::clip(SoundClipId::new(999))
    };
    let source_id = sound.create_source(source).unwrap();
    sound
        .bind_automation(SoundAutomationBinding {
            id: SoundAutomationBindingId::new(1),
            timeline_track_path: "Root/Synth:sound.synth.cutoff".to_string(),
            target: SoundAutomationTarget::SynthParameter(parameter),
            parameter: SoundParameterId::new("value"),
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();
    let snapshot = sound.mixer_snapshot().unwrap();

    assert_eq!(mix.samples, vec![0.25, 0.25]);
    assert!(snapshot
        .graph
        .sources
        .iter()
        .any(|source| source.id == Some(source_id)));
    assert_eq!(snapshot.graph.automation_bindings.len(), 1);
    assert!(snapshot.graph.dynamic_events.events.is_empty());
}

#[test]
fn spatial_source_uses_active_listener_for_attenuation_pan_and_occlusion() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/spatial.wav", &[1.0]));
    sound.update_listener(test_listener()).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [3.0, 0.0, 0.0];
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        occlusion_enabled: true,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_sample_near(mix.samples[0], 0.0);
    assert_sample_near(mix.samples[1], 0.35);
}

#[test]
fn audio_volume_priority_and_crossfade_apply_to_source_output() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/volume.wav", &[1.0]));
    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [2.0, 0.0, 0.0];
    sound.create_source(source).unwrap();
    sound
        .update_volume(SoundVolumeDescriptor {
            id: SoundVolumeId::new(1),
            shape: SoundVolumeShape::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 5.0,
            },
            priority: 0,
            interior_gain: 0.1,
            exterior_gain: 1.0,
            low_pass_cutoff_hz: None,
            reverb_send: 0.0,
            convolution_send: None,
            crossfade_distance: 0.0,
        })
        .unwrap();
    sound
        .update_volume(SoundVolumeDescriptor {
            id: SoundVolumeId::new(2),
            shape: SoundVolumeShape::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
            },
            priority: 10,
            interior_gain: 0.25,
            exterior_gain: 1.0,
            low_pass_cutoff_hz: None,
            reverb_send: 0.0,
            convolution_send: None,
            crossfade_distance: 3.0,
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_sample_near(mix.samples[0], 0.5);
    assert_sample_near(mix.samples[1], 0.5);
}

#[test]
fn source_sends_can_tap_pre_spatial_signal() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/pre-spatial.wav", &[0.5]));
    let room = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(room, "Room"))
        .unwrap();
    sound.update_listener(test_listener()).unwrap();

    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [3.0, 0.0, 0.0];
    source.sends.push(SoundSourceSend {
        target: room,
        gain: 1.0,
        pre_spatial: true,
    });
    source.spatial = SoundSpatialSourceSettings {
        spatial_blend: 1.0,
        min_distance: 1.0,
        max_distance: 5.0,
        attenuation: SoundAttenuationMode::Linear,
        ..SoundSpatialSourceSettings::default()
    };
    sound.create_source(source).unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_sample_near(mix.samples[0], 0.5);
    assert_sample_near(mix.samples[1], 0.75);
}

fn test_clip(uri: &str, mono_samples: &[f32]) -> SoundAsset {
    SoundAsset {
        uri: AssetUri::parse(uri).unwrap(),
        sample_rate_hz: 48_000,
        channel_count: 1,
        samples: mono_samples.to_vec(),
    }
}

fn test_listener() -> SoundListenerDescriptor {
    SoundListenerDescriptor {
        id: SoundListenerId::new(1),
        active: true,
        position: [0.0, 0.0, 0.0],
        forward: [0.0, 0.0, 1.0],
        up: [0.0, 1.0, 0.0],
        left_ear_offset: [-0.08, 0.0, 0.0],
        right_ear_offset: [0.08, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        hrtf_profile: None,
        doppler_tracking: true,
        mixer_target: SoundTrackId::master(),
    }
}

fn test_effect(kind: SoundEffectKind) -> SoundEffectDescriptor {
    SoundEffectDescriptor::new(SoundEffectId::new(99), "Test Effect", kind)
}

fn render_master_effect(
    effect: SoundEffectDescriptor,
    mono_samples: &[f32],
    frames: usize,
) -> Vec<f32> {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/effect.wav", mono_samples));
    sound
        .add_or_update_effect(SoundTrackId::master(), effect)
        .unwrap();
    sound
        .play_clip(clip, SoundPlaybackSettings::default())
        .unwrap();
    sound.render_mix(frames).unwrap().samples
}

fn render_sidechain_tap_mix(pre_effects: bool) -> Vec<f32> {
    let sound = DefaultSoundManager::default();
    let target_clip =
        sound.insert_clip_for_test(test_clip("res://sound/sidechain-target.wav", &[0.5]));
    let key_clip = sound.insert_clip_for_test(test_clip("res://sound/sidechain-key.wav", &[0.5]));
    let target = SoundTrackId::new(2);
    let key = SoundTrackId::new(3);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(target, "Target"))
        .unwrap();
    let mut key_track = SoundTrackDescriptor::child(key, "Muted Key");
    key_track.controls.mute = true;
    sound.add_or_update_track(key_track).unwrap();
    sound
        .add_or_update_effect(
            target,
            SoundEffectDescriptor::new(
                SoundEffectId::new(77),
                "Sidechain Compressor",
                SoundEffectKind::Compressor(SoundCompressorEffect {
                    threshold_db: -18.0,
                    ratio: 8.0,
                    attack_ms: 1.0,
                    release_ms: 50.0,
                    makeup_gain_db: 0.0,
                    sidechain: Some(SoundSidechainInput {
                        track: key,
                        pre_effects,
                    }),
                }),
            ),
        )
        .unwrap();
    sound
        .play_clip(
            target_clip,
            SoundPlaybackSettings {
                output_track: target,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    sound
        .play_clip(
            key_clip,
            SoundPlaybackSettings {
                output_track: key,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    sound.render_mix(1).unwrap().samples
}

fn assert_sample_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 0.0001,
        "expected {expected}, got {actual}"
    );
}

fn assert_samples_near(actual: &[f32], expected: &[f32]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected.iter()) {
        assert_sample_near(*actual, *expected);
    }
}
