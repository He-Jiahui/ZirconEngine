use super::super::super::*;

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
