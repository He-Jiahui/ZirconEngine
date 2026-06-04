use super::super::*;

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
