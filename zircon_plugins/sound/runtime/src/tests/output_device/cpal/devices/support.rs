use super::super::super::super::*;
use zircon_runtime::core::framework::sound::SoundOutputDeviceInfo;

pub(super) fn cpal_output_devices(sound: &DefaultSoundManager) -> Vec<SoundOutputDeviceInfo> {
    sound
        .available_output_devices()
        .unwrap()
        .into_iter()
        .filter(|device| device.descriptor.backend == "cpal")
        .collect()
}
