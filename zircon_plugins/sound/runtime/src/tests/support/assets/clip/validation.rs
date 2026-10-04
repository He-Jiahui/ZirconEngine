// 所有夹具样本必须组成完整帧，避免测试失败来自不完整资源而非被测播放契约。
pub(super) fn assert_complete_frames(channel_count: u16, samples: &[f32]) {
    assert_ne!(channel_count, 0, "test clip channel count must be non-zero");
    assert_eq!(
        samples.len() % channel_count as usize,
        0,
        "test clip samples must contain complete frames"
    );
}
