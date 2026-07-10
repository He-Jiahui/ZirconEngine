use super::super::super::super::*;
use super::descriptor::software_test_descriptor;

pub(crate) fn invalid_block_size_descriptor() -> SoundOutputDeviceDescriptor {
    software_test_descriptor(
        "sound.output.bad",
        "Bad Output",
        AudioChannelLayout::stereo(),
        2,
        0,
    )
}
