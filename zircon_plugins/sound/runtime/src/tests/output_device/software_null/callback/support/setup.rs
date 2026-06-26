use super::super::super::super::super::*;
use super::descriptors::software_null_descriptor;

pub(crate) fn configure_started_software_null_output(sound: &DefaultSoundManager) {
    sound
        .configure_output_device(software_null_descriptor())
        .unwrap();
    sound.start_output_device().unwrap();
}
