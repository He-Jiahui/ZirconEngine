use super::*;
use crate::core::framework::render::{
    EnvironmentExtract, PreviewEnvironmentExtract, RenderLayerSet, RenderSceneGeometryExtract,
    ViewProjectionMatrixPair, ViewportCameraSnapshot,
};
use crate::core::math::Vec3;

#[test]
fn six_face_selection_reuses_one_moved_scene_extract() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [4.0, 5.0, 6.0], 7)
        .unwrap()
        .with_face_size(256)
        .unwrap();
    let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);
    let extract_identity = batch.extract_identity();

    for face in CubemapFace::ALL {
        let view = batch.select_face(face);

        assert_eq!(view.face(), face);
        assert!(view.reverse_raster_winding());
        assert_eq!(view.frame().viewport_size, UVec2::splat(256));
        assert_eq!(
            view.frame().render_region().physical_size(),
            UVec2::splat(256)
        );
        assert_eq!(view.frame().overlays(), &RenderOverlayExtract::default());
        assert_eq!(batch.extract_identity(), extract_identity);
    }
    assert_eq!(batch.selected_face(), Some(CubemapFace::NegativeZ));
}

#[test]
fn capture_uses_authored_lighting_independent_of_viewport_preview_mode() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1).unwrap();
    let batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

    assert!(batch.frame().preview().lighting_enabled);
}

#[test]
fn capture_mask_is_installed_before_draw_build_and_kept_for_every_face() {
    let capture_layers = RenderLayerSet::from_layers([3, 37]);
    let request = RenderEnvironmentCaptureRequest::new("probe", [0.0; 3], 1)
        .unwrap()
        .with_capture_layer_mask(capture_layers.clone());
    let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

    let initial_descriptor = batch
        .frame()
        .extract
        .view
        .selected_camera_descriptor()
        .unwrap();
    assert_eq!(initial_descriptor.culling_mask, capture_layers);
    assert_eq!(initial_descriptor.volume_mask, RenderLayerSet::default());

    for face in CubemapFace::ALL {
        let view = batch.select_face(face);
        let descriptor = view
            .frame()
            .extract
            .view
            .selected_camera_descriptor()
            .unwrap();
        assert_eq!(descriptor.culling_mask, capture_layers);
        assert_eq!(descriptor.volume_mask, RenderLayerSet::default());
    }
}

#[test]
fn selected_face_projects_its_canonical_center_without_temporal_state() {
    let request = RenderEnvironmentCaptureRequest::new("probe", [4.0, 5.0, 6.0], 7)
        .unwrap()
        .with_clip_planes(0.25, 500.0)
        .unwrap();
    let origin = Vec3::from_array(request.position());
    let mut batch = EnvironmentCaptureSceneBatch::new(test_scene(), request);

    for face in CubemapFace::ALL {
        let axes = face.projection_axes();
        let view = batch.select_face(face);
        let pair = ViewProjectionMatrixPair::from_camera(
            &view.frame().effective_camera(),
            view.frame().viewport_size,
        );
        let clip = pair
            .clip_from_world_unjittered
            .project_point3(origin + Vec3::from_array(axes.forward));

        assert!(clip.x.abs() <= 0.00001);
        assert!(clip.y.abs() <= 0.00001);
        assert!(view.frame().previous_motion_vector_camera().is_none());
        assert!(view.frame().ui.is_none());
    }
}

fn test_scene() -> SceneViewportRenderPacket {
    let environment = EnvironmentExtract::default();
    SceneViewportRenderPacket {
        scene: RenderSceneGeometryExtract {
            camera: ViewportCameraSnapshot::default(),
            meshes: Vec::new(),
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract::default(),
        preview: PreviewEnvironmentExtract::from_environment(
            &environment,
            false,
            crate::core::math::Vec4::ZERO,
        ),
        environment,
        virtual_geometry_debug: None,
    }
}
