use super::super::super::super::super::super::*;
use super::shared::output_descriptor;

pub(super) fn cpal_disabled_descriptor() -> SoundOutputDeviceDescriptor {
    output_descriptor("sound.output.cpal.disabled", "CPAL Disabled", "cpal")
}
