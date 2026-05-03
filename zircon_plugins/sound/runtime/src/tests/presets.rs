use super::*;

#[test]
fn built_in_mixer_presets_are_discoverable_and_apply() {
    let sound = DefaultSoundManager::default();

    let presets = sound.available_mixer_presets().unwrap();
    assert!(presets
        .iter()
        .any(|preset| preset.locator == "sound://mixer/default"));
    assert!(presets
        .iter()
        .any(|preset| preset.locator == "sound://mixer/music_sfx"));
    assert!(presets
        .iter()
        .any(|preset| preset.locator == "sound://mixer/spatial_room"));

    sound
        .apply_mixer_preset("sound://mixer/spatial_room")
        .unwrap();
    let snapshot = sound.mixer_snapshot().unwrap();

    assert!(snapshot
        .graph
        .tracks
        .iter()
        .any(|track| track.id == SoundTrackId::master() && track.display_name == "Master"));
    assert!(snapshot
        .graph
        .tracks
        .iter()
        .any(|track| track.id == SoundTrackId::new(5) && track.display_name == "Room Reverb"));
    let sfx = snapshot
        .graph
        .tracks
        .iter()
        .find(|track| track.id == SoundTrackId::new(3))
        .unwrap();
    assert!(sfx
        .sends
        .iter()
        .any(|send| send.target == SoundTrackId::new(5) && send.gain > 0.0));
}

#[test]
fn applying_mixer_preset_reroutes_sources_and_playbacks_from_removed_tracks() {
    let sound = DefaultSoundManager::default();
    let custom_track = SoundTrackId::new(99);
    sound
        .add_or_update_track(SoundTrackDescriptor::child(custom_track, "Temporary Bus"))
        .unwrap();

    let clip = sound.insert_clip_for_test(test_clip("res://sound/preset-reroute.wav", &[0.5]));
    sound
        .play_clip(
            clip,
            SoundPlaybackSettings {
                output_track: custom_track,
                ..SoundPlaybackSettings::default()
            },
        )
        .unwrap();
    let mut source = SoundSourceDescriptor::clip(clip);
    source.output_track = custom_track;
    source.sends.push(SoundSourceSend {
        target: custom_track,
        gain: 1.0,
        pre_spatial: false,
    });
    let source_id = sound.create_source(source).unwrap();

    sound.apply_mixer_preset("sound://mixer/default").unwrap();
    let snapshot = sound.mixer_snapshot().unwrap();
    let source = snapshot
        .graph
        .sources
        .iter()
        .find(|source| source.id == Some(source_id))
        .unwrap();

    assert_eq!(source.output_track, SoundTrackId::master());
    assert!(source.sends.is_empty());
    assert_samples_near(&sound.render_mix(1).unwrap().samples, &[1.0, 1.0]);
}

#[test]
fn applying_unknown_mixer_preset_returns_typed_locator_error() {
    let sound = DefaultSoundManager::default();

    assert!(matches!(
        sound
            .apply_mixer_preset("sound://mixer/missing")
            .unwrap_err(),
        SoundError::InvalidLocator { .. }
    ));
}
