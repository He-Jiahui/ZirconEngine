use zircon_runtime::core::framework::render::{RenderFrameExtract, ViewProjectionMatrixPair};
use zircon_runtime::core::math::UVec2;

pub(super) const SCENE_HZB_CAMERA_PACKET_MAGIC: u32 = 0x4847_4943;
pub(super) const SCENE_HZB_CAMERA_WORD_OFFSET: u64 = 272;
pub(super) const SCENE_HZB_CAMERA_WORD_COUNT: usize = 22;

pub(super) fn scene_hzb_camera_packet(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
) -> [u32; SCENE_HZB_CAMERA_WORD_COUNT] {
    let camera = &extract.view.camera;
    // 深度与 HZB 使用带时域抖动的投影；世界坐标重建必须使用同一投影的逆矩阵。
    let inverse_view_projection = ViewProjectionMatrixPair::from_camera(camera, viewport_size)
        .clip_from_world_jittered
        .inverse();

    let mut words = [0_u32; SCENE_HZB_CAMERA_WORD_COUNT];
    words[0] = SCENE_HZB_CAMERA_PACKET_MAGIC;
    for (index, value) in inverse_view_projection
        .to_cols_array()
        .into_iter()
        .enumerate()
    {
        words[index + 1] = value.to_bits();
    }
    let camera_position = camera.transform.translation.to_array();
    for (index, value) in camera_position.into_iter().enumerate() {
        words[index + 17] = value.to_bits();
    }
    words[20] = viewport_size.x.max(1);
    words[21] = viewport_size.y.max(1);
    words
}

#[cfg(test)]
#[path = "tests/scene_hzb_camera_packet.rs"]
mod tests;
