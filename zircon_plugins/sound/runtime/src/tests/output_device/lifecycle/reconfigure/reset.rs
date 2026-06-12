use super::super::super::super::*;
use super::support::reconfigure_preview_output;

#[test]
fn reconfigure_stops_output_device_and_resets_render_progress() {
    let sound = DefaultSoundManager::default();
    reconfigure_preview_output(&sound);

    let status = sound.output_device_status().unwrap();
    assert_eq!(status.state, SoundOutputDeviceState::Stopped);
    assert_eq!(status.rendered_frames, 0);
}
