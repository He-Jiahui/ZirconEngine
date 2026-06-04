use super::super::super::*;
use super::support::cinematic_config;

#[test]
fn sound_manager_projects_preserved_config_options() {
    let sound = DefaultSoundManager::with_config(None, cinematic_config());

    assert_eq!(sound.global_volume_gain().unwrap(), 0.25);
    assert_eq!(sound.default_spatial_scale().unwrap(), 2.5);
    let mix = sound.render_mix(1).unwrap();
    assert_eq!(mix.sample_rate_hz, 44_100);
    assert_eq!(mix.channel_count, 6);
    assert_eq!(mix.channel_layout, SoundChannelLayout::surround_5_1());
    assert_eq!(mix.samples, vec![0.0; 6]);
}
