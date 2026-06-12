use super::super::super::super::*;

pub(super) fn software_null_picker(
    sound: &DefaultSoundManager,
) -> zircon_runtime::core::framework::sound::SoundOutputDeviceInfo {
    sound
        .available_output_devices()
        .unwrap()
        .into_iter()
        .find(|device| device.descriptor.backend == "software-null")
        .expect("software-null output device should be listed")
}
