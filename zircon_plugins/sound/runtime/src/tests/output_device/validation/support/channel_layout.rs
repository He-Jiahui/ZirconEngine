// 故意组合环绕布局与双声道数量，供设备描述符校验识别布局不一致。
use super::super::super::super::*;
use super::descriptor::software_test_descriptor;

pub(crate) fn invalid_channel_layout_descriptor() -> SoundOutputDeviceDescriptor {
    software_test_descriptor(
        "sound.output.bad-layout",
        "Bad Layout Output",
        AudioChannelLayout::surround_5_1(),
        2,
        128,
    )
}
