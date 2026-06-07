use super::super::*;

#[test]
fn sound_config_normalizes_zero_channel_count_and_layout_mismatch() {
    let config = SoundConfig::from_plugin_options(SoundPluginOptions {
        channel_count: 0,
        channel_layout: SoundChannelLayout::surround_5_1(),
        ..SoundPluginOptions::default()
    });

    assert_eq!(config.channel_count, 1);
    assert_eq!(config.channel_layout, SoundChannelLayout::mono());

    let sound = DefaultSoundManager::with_config(None, config);
    assert_eq!(
        sound
            .output_device_status()
            .unwrap()
            .descriptor
            .channel_layout,
        SoundChannelLayout::mono()
    );
    assert_eq!(
        sound.render_mix(1).unwrap().channel_layout,
        SoundChannelLayout::mono()
    );

    let config = SoundConfig::from_plugin_options(SoundPluginOptions {
        channel_count: 2,
        channel_layout: SoundChannelLayout {
            name: "stereo".to_string(),
            channel_count: 2,
            speakers: vec![
                SoundSpeakerChannel::FrontRight,
                SoundSpeakerChannel::FrontLeft,
            ],
        },
        ..SoundPluginOptions::default()
    });

    assert_eq!(config.channel_count, 2);
    assert_eq!(config.channel_layout, SoundChannelLayout::stereo());
}
