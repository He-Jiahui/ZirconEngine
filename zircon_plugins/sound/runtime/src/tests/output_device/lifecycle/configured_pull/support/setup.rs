use super::super::super::super::super::*;
use super::descriptors::test_output_descriptor;

pub(super) fn configure_started_test_output(
    sound: &DefaultSoundManager,
) -> SoundOutputDeviceDescriptor {
    let descriptor = test_output_descriptor();
    sound.configure_output_device(descriptor.clone()).unwrap();
    sound.start_output_device().unwrap();
    descriptor
}
