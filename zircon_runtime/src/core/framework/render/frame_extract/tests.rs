use super::*;
use crate::core::framework::render::{RenderCameraTarget, RenderLayerSet, RenderViewportRect};
use crate::core::math::{Transform, Vec4};
use crate::core::resource::TextureMarker;

#[test]
fn render_view_apply_target_size_preserves_descriptor_target_and_layers() {
    let mut view = RenderViewExtract::from_camera(ViewportCameraSnapshot::default());
    let mut descriptor =
        CameraRenderDescriptor::from_camera_payload(Some(7), ViewportCameraSnapshot::default());
    descriptor.target = RenderCameraTarget::Headless {
        size: UVec2::new(320, 180),
    };
    descriptor.viewport_rect = Some(RenderViewportRect::new(UVec2::ZERO, UVec2::new(320, 160)));
    descriptor.culling_mask = RenderLayerSet::layer(3);
    descriptor.volume_mask = RenderLayerSet::layer(4);
    view.scene_camera_entity = Some(7);
    view.cameras = vec![descriptor];

    view.apply_target_size(UVec2::new(1280, 720));

    let selected = view
        .selected_camera_descriptor()
        .expect("selected scene camera descriptor should remain present");
    assert!(matches!(
        selected.target,
        RenderCameraTarget::Headless {
            size: UVec2 { x: 320, y: 180 }
        }
    ));
    assert_eq!(selected.culling_mask.to_legacy_mask_lossy(), 1 << 3);
    assert_eq!(selected.volume_mask.to_legacy_mask_lossy(), 1 << 4);
    assert!((view.camera.aspect_ratio - 2.0).abs() < 1.0e-4);
}

#[test]
fn render_frame_extract_selected_camera_descriptor_replaces_active_selection_only() {
    let texture =
        ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label("tests/camera-loop/rt"));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(10),
        RenderSceneSnapshot {
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
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    let mut primary =
        CameraRenderDescriptor::from_camera_payload(Some(1), ViewportCameraSnapshot::default());
    primary.render_order = 0;
    let mut target =
        CameraRenderDescriptor::from_camera_payload(Some(2), ViewportCameraSnapshot::default());
    target.render_order = 10;
    target.target = RenderCameraTarget::Texture(texture);
    target.culling_mask = RenderLayerSet::layer(4);
    extract.view = extract.view.with_cameras(vec![primary, target.clone()]);

    let selected = extract.with_selected_camera_descriptor(target.clone());

    assert_eq!(selected.view.scene_camera_entity, Some(2));
    assert_eq!(selected.view.cameras.len(), 1);
    assert_eq!(selected.view.selected_camera_target(), &target.target);
    assert_eq!(
        selected
            .view
            .selected_camera_layers()
            .to_legacy_mask_lossy(),
        1 << 4
    );
}

#[test]
fn render_frame_extract_visibility_input_preserves_layers_above_legacy_mask_width() {
    let high_layer_mask = RenderLayerSet::layer(40);
    let extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(11),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: vec![RenderMeshSnapshot {
                    node_id: 42,
                    stable_instance_key: 42,
                    transform_revision: 0,
                    transform: Transform::default(),
                    model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                        "tests/visibility-input/model",
                    )),
                    mesh: None,
                    material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                        "tests/visibility-input/material",
                    )),
                    mesh_lod: None,
                    morph_weights: Vec::new(),
                    tint: Vec4::ONE,
                    mobility: Mobility::Static,
                    static_state: RenderMeshStaticState::default(),
                    render_layer_mask: high_layer_mask,
                }],
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );

    let render_layer_mask = &extract.visibility.renderables[0].render_layer_mask;
    assert!(render_layer_mask.contains(40));
    assert_eq!(render_layer_mask.to_legacy_mask_lossy(), 0);
}
