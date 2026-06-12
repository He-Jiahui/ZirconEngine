use super::super::super::super::super::*;
use super::descriptors::preview_output_descriptor;

pub(super) fn reconfigure_preview_output(sound: &DefaultSoundManager) {
    sound.start_output_device().unwrap();
    sound
        .configure_output_device(preview_output_descriptor())
        .unwrap();
}
