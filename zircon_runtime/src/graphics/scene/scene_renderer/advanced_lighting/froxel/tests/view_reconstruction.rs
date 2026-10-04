use super::*;

#[test]
fn render_froxel_view_reconstruction_uses_unjittered_camera_projection() {
    let camera = ViewportCameraSnapshot::default();
    let viewport = UVec2::new(1600, 900);

    let view = FroxelViewReconstruction::from_camera(&camera, viewport);
    let expected = ViewProjectionMatrixPair::from_camera(&camera, viewport)
        .clip_from_world_unjittered
        .inverse();

    assert_eq!(view.world_from_clip, expected);
    assert!(!view.orthographic);
}
