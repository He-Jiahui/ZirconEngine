use super::super::super::super::*;
use super::support::{configure_started_test_output, play_output_test_clip};

#[test]
fn configured_output_device_renders_block_samples() {
    let sound = DefaultSoundManager::default();
    configure_started_test_output(&sound);
    play_output_test_clip(&sound);

    let block = sound.render_output_device_block().unwrap();
    assert_eq!(block.channel_layout, AudioChannelLayout::stereo());
    assert_samples_near(&block.samples, &[0.25, 0.25, 0.5, 0.5]);
}
