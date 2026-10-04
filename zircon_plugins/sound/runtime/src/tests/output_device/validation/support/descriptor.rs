// 构造软件测试设备基线，允许调用方故意传入不一致元数据；不代表枚举到的主机硬件。
use super::super::super::super::*;

pub(super) fn software_test_descriptor(
    id: &str,
    display_name: &str,
    channel_layout: AudioChannelLayout,
    channel_count: u16,
    block_size_frames: usize,
) -> SoundOutputDeviceDescriptor {
    SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new(id),
        backend: "software-test".to_string(),
        display_name: display_name.to_string(),
        sample_rate_hz: 48_000,
        channel_count,
        channel_layout,
        block_size_frames,
        latency_blocks: 2,
    }
}
