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
