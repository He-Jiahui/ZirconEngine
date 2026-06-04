use zircon_runtime::core::framework::sound::{
    SoundMixRenderManager, SoundMixerGraphManager, SoundPlaybackManager, SoundPlaybackSettings,
    SoundSourceDescriptor, SoundSourceManager, SoundSourceSend, SoundTrackDescriptor, SoundTrackId,
};

use super::super::{assert_samples_near, test_clip, DefaultSoundManager};

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
