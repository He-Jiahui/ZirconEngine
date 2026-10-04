use super::{fog_volume_from_extract, render_layers_for_view};
use crate::core::framework::render::{
    CameraRenderDescriptor, PostProcessVolumeExtract, RenderLayerSet, RenderViewExtract,
    ViewportCameraSnapshot, VolumeShapeExtract,
};
use crate::core::math::Vec3;

#[test]
fn render_volumetric_camera_stack_unions_overlay_culling_layers() {
    let mut base = camera_descriptor(10, 0b0001);
    base.stack = vec![20];
    let overlay = camera_descriptor(20, 0b0010);
    let unrelated = camera_descriptor(30, 0b0100);
    let view = RenderViewExtract::from_camera(base.camera.clone())
        .with_selected_camera_descriptor(base.clone())
        .with_cameras(vec![base, overlay, unrelated]);

    assert_eq!(
        render_layers_for_view(&view),
        RenderLayerSet::from_scene_schema_v1_mask(0b0011)
    );
}

#[test]
fn fog_volume_from_extract_uses_supplied_layer_mask() {
    let extract_mask = RenderLayerSet::from_scene_schema_v1_mask(0b0001);
    let fog_mask = RenderLayerSet::from_scene_schema_v1_mask(0b0010);
    let extract = PostProcessVolumeExtract::new(
        true,
        VolumeShapeExtract::sphere(Vec3::ZERO, 1.0, 0.0),
        0.0,
        1.0,
        extract_mask.clone(),
        Vec::new(),
    );

    let fog = fog_volume_from_extract(42, &extract, fog_mask.clone(), 0.5, Vec3::ONE)
        .expect("finite sphere bounds should yield a fog volume");

    assert_eq!(fog.layer_mask, fog_mask);
    assert_eq!(extract.volume_mask, extract_mask);
}

fn camera_descriptor(entity: u64, culling_mask: u32) -> CameraRenderDescriptor {
    let mut camera = CameraRenderDescriptor::from_camera_payload(
        Some(entity),
        ViewportCameraSnapshot::default(),
    );
    camera.culling_mask = RenderLayerSet::from_scene_schema_v1_mask(culling_mask);
    camera
}
