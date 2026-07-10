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
mod tests {
    use super::*;
    use zircon_runtime::core::framework::render::{
        RenderWorldSnapshotHandle, TemporalJitterSample, ViewProjectionMatrixPair,
    };
    use zircon_runtime::core::math::Vec2;
    use zircon_runtime::scene::World;

    #[test]
    fn scene_hzb_camera_packet_contains_inverse_view_projection_and_viewport() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            World::new().to_render_snapshot(),
        );
        extract.view.camera.temporal_jitter = TemporalJitterSample {
            offset_pixels: Vec2::new(0.5, -0.25),
            sequence_index: 3,
        };
        let packet = scene_hzb_camera_packet(&extract, UVec2::new(192, 128));

        assert_eq!(packet[0], SCENE_HZB_CAMERA_PACKET_MAGIC);
        assert!(packet[1..17].iter().any(|word| *word != 0));
        let expected =
            ViewProjectionMatrixPair::from_camera(&extract.view.camera, UVec2::new(192, 128))
                .clip_from_world_jittered
                .inverse()
                .to_cols_array()
                .map(f32::to_bits);
        assert_eq!(packet[1..17], expected);
        let unjittered =
            ViewProjectionMatrixPair::from_camera(&extract.view.camera, UVec2::new(192, 128))
                .clip_from_world_unjittered
                .inverse()
                .to_cols_array()
                .map(f32::to_bits);
        assert_ne!(packet[1..17], unjittered);
        assert_eq!(packet[20..22], [192, 128]);
    }
}
