use super::super::super::super::*;
use super::descriptor::software_test_descriptor;

pub(crate) fn invalid_channel_layout_descriptor() -> SoundOutputDeviceDescriptor {
    software_test_descriptor(
        "sound.output.bad-layout",
        "Bad Layout Output",
        SoundChannelLayout::surround_5_1(),
        2,
        128,
    )
}
