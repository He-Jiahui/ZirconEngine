use super::super::super::super::super::super::*;
use super::shared::output_descriptor;

pub(super) fn software_null_recovery_descriptor() -> SoundOutputDeviceDescriptor {
    output_descriptor(
        "sound.output.cpal.recovery",
        "Software Null Recovery",
        "software-null",
    )
}
