use super::super::super::super::*;
use super::support::cpal_windows_default_output;

#[test]
fn cpal_backend_start_stop_is_structured_on_windows() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(cpal_windows_default_output())
        .unwrap();

    match sound.start_output_device() {
        Ok(()) => {
            assert_eq!(
                sound.output_device_status().unwrap().state,
                SoundOutputDeviceState::Started
            );
            sound.stop_output_device().unwrap();
            assert_eq!(
                sound.output_device_status().unwrap().state,
                SoundOutputDeviceState::Stopped
            );
        }
        Err(error) => {
            assert!(error.to_string().contains("cpal") || error.to_string().contains("device"));
            assert_eq!(
                sound.output_device_status().unwrap().state,
                SoundOutputDeviceState::Stopped
            );
        }
    }
}
