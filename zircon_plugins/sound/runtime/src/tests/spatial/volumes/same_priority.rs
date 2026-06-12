use super::super::super::*;

#[test]
fn equal_priority_audio_volumes_choose_strongest_crossfade_influence() {
    let sound = DefaultSoundManager::default();
    let clip = sound.insert_clip_for_test(test_clip("res://sound/volume-weight.wav", &[1.0]));
    let mut source = SoundSourceDescriptor::clip(clip);
    source.position = [0.0, 0.0, 0.0];
    sound.create_source(source).unwrap();
    sound
        .update_volume(SoundVolumeDescriptor {
            id: SoundVolumeId::new(1),
            shape: SoundVolumeShape::Sphere {
                center: [1.0, 0.0, 0.0],
                radius: 0.0,
            },
            priority: 5,
            interior_gain: 0.0,
            exterior_gain: 1.0,
            low_pass_cutoff_hz: None,
            reverb_send: 0.0,
            convolution_send: None,
            crossfade_distance: 2.0,
        })
        .unwrap();
    sound
        .update_volume(SoundVolumeDescriptor {
            id: SoundVolumeId::new(2),
            shape: SoundVolumeShape::Sphere {
                center: [0.0, 0.0, 0.0],
                radius: 1.0,
            },
            priority: 5,
            interior_gain: 0.2,
            exterior_gain: 1.0,
            low_pass_cutoff_hz: None,
            reverb_send: 0.0,
            convolution_send: None,
            crossfade_distance: 0.0,
        })
        .unwrap();

    let mix = sound.render_mix(1).unwrap();

    assert_sample_near(mix.samples[0], 0.2);
    assert_sample_near(mix.samples[1], 0.2);
}
