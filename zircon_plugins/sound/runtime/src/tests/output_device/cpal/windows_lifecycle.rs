#[cfg(all(feature = "cpal-backend", target_os = "windows"))]
use super::super::super::*;

#[cfg(all(feature = "cpal-backend", target_os = "windows"))]
#[test]
fn cpal_backend_start_stop_is_structured_on_windows() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.cpal.windows"),
            backend: "cpal".to_string(),
            display_name: "CPAL Windows Default Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            channel_layout: SoundChannelLayout::stereo(),
            block_size_frames: 128,
            latency_blocks: 2,
        })
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
