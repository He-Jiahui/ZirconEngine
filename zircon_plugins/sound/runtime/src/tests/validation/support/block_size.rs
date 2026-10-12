// 只把块大小设为无效值，其余设备字段沿用有效基线，供公共入口定位对应拒绝原因。
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
