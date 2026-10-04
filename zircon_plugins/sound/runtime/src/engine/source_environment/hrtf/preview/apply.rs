//! 几何预览只处理当前块内的双耳延迟，块外样本按零补齐；需要连续尾音的路径应使用有历史状态的已加载 HRTF。
use super::profile::HrtfPreviewProfile;

use crate::engine::hrtf::clear_non_binaural_output_channels;

pub(super) fn apply_hrtf_preview_profile(
    buffer: &mut [f32],
    channels: usize,
    profile: HrtfPreviewProfile,
) {
    let dry = buffer.to_vec();
    let frames = buffer.len() / channels;
    for frame in 0..frames {
        let left = frame
            .checked_sub(profile.left_delay_frames)
            .and_then(|source_frame| dry.get(source_frame * channels))
            .copied()
            .unwrap_or_default();
        let right = frame
            .checked_sub(profile.right_delay_frames)
            .and_then(|source_frame| dry.get(source_frame * channels + 1))
            .copied()
            .unwrap_or_default();
        buffer[frame * channels] = left * profile.left_gain;
        buffer[frame * channels + 1] = right * profile.right_gain;
    }
    clear_non_binaural_output_channels(buffer, channels);
}
