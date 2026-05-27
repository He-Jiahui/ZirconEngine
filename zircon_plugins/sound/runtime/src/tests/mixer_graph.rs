use super::*;

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
fn removing_track_reroutes_active_playbacks_before_finished_events() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/remove-track.wav", &[0.5]));
    let music = SoundTrackId::new(2);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(music, "Music"))
        .unwrap();
    let playback = sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: music,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();

    assert_eq!(sound.playback_status(playback).unwrap().output_track, music);
    sound.remove_track(music).unwrap();
    assert_eq!(
        sound.playback_status(playback).unwrap().output_track,
        SoundTrackId::master()
    );
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.5, 0.5]);
    assert_eq!(sound.render_mix(1).unwrap().samples, vec![0.0, 0.0]);
    let finished = sound.drain_finished_playbacks().unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!(finished[0].playback, playback);
    assert_eq!(finished[0].output_track, SoundTrackId::master());
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
