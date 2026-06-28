use super::*;
use crate::core::framework::render::{
    resolve_camera_sequence, resolve_camera_sequence_borrowed, CameraRenderType,
    FallbackSkyboxKind, PreviewEnvironmentExtract, RenderCameraTarget, RenderLayerSet,
    RenderOverlayExtract, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
    RenderPreparedRuntimeSidebands, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderViewportRect, RenderVirtualGeometryExtract, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::ViewportRenderFrame;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

mod frame;

fn camera_loop_extracts(
    extract: &RenderFrameExtract,
) -> Result<Vec<RenderFrameExtract>, RenderFrameworkError> {
    let sequence = resolve_camera_sequence_borrowed(&extract.view.cameras);
    if sequence.sequence.is_empty() {
        return Err(RenderFrameworkError::UnsupportedCapability {
            capability: "active camera sequence".to_string(),
        });
    }

    Ok(camera_sequence_descriptors(sequence)
        .into_iter()
        .map(|camera| extract.clone().with_selected_camera_descriptor(camera))
        .collect())
}

fn camera_sequence_descriptors(
    sequence: crate::core::framework::render::CameraSequenceReport,
) -> Vec<CameraRenderDescriptor> {
    camera_sequence_submission_descriptors(sequence.sequence)
        .into_iter()
        .map(|submission| submission.camera)
        .collect()
}

#[test]
fn camera_loop_flattens_base_then_overlays_for_submit_order() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/base-target",
    ));
    let base = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(texture),
    )
    .with_stack([3, 2]);
    let overlay_late = descriptor(20, 2, CameraRenderType::Overlay, base.target.clone());
    let overlay_first = descriptor(10, 3, CameraRenderType::Overlay, base.target.clone());
    let other_base = descriptor(
        -10,
        4,
        CameraRenderType::Base,
        RenderCameraTarget::Headless {
            size: UVec2::new(32, 32),
        },
    );

    let flattened = camera_sequence_descriptors(resolve_camera_sequence([
        overlay_late,
        base,
        overlay_first,
        other_base,
    ]));

    assert_eq!(
        flattened
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![Some(4), Some(1), Some(3), Some(2)]
    );
}

#[test]
fn camera_loop_extracts_select_each_sequence_descriptor() {
    let base = descriptor(
        -4,
        10,
        CameraRenderType::Base,
        RenderCameraTarget::Headless {
            size: UVec2::new(96, 48),
        },
    )
    .with_layers(RenderLayerSet::layer(2));
    let primary = descriptor(
        8,
        20,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_layers(RenderLayerSet::layer(5));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        empty_scene_snapshot(),
    );
    extract.view = extract.view.with_cameras(vec![primary, base]);

    let extracts = camera_loop_extracts(&extract).expect("active camera sequence");

    assert_eq!(
        extracts
            .iter()
            .map(|extract| extract.view.scene_camera_entity)
            .collect::<Vec<_>>(),
        vec![Some(10), Some(20)]
    );
    assert!(matches!(
        extracts[0].view.selected_camera_target(),
        RenderCameraTarget::Headless {
            size: UVec2 { x: 96, y: 48 }
        }
    ));
    assert_eq!(
        extracts[0]
            .view
            .selected_camera_layers()
            .to_scene_schema_v1_mask_lossy(),
        1 << 2
    );
    assert_eq!(
        extracts[1]
            .view
            .selected_camera_layers()
            .to_scene_schema_v1_mask_lossy(),
        1 << 5
    );
}

#[test]
fn submit_camera_loop_streams_source_extract_and_restores_derived_state() {
    let first = descriptor(
        0,
        11,
        CameraRenderType::Base,
        RenderCameraTarget::Headless {
            size: UVec2::new(96, 48),
        },
    );
    let second = descriptor(
        4,
        22,
        CameraRenderType::Base,
        RenderCameraTarget::Headless {
            size: UVec2::new(160, 80),
        },
    );
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(2),
        empty_scene_snapshot(),
    );
    extract.view.target_size = None;
    extract.view = extract.view.with_cameras(vec![first, second]);
    let submissions = camera_loop_submissions(&extract).expect("active camera sequence");
    let mut observed = Vec::new();

    stream_camera_loop_extract_submissions(
        extract,
        Some(UiRenderExtract::default()),
        submissions,
        |extract, _source_payloads, ui, output_policy| {
            observed.push((
                extract.view.scene_camera_entity,
                extract.view.target_size,
                ui.is_some(),
                ViewportCameraStackOutputPolicy::from(output_policy).owns_viewport_submission(),
            ));
            Arc::make_mut(extract).apply_viewport_size(UVec2::new(999, 777));
            Ok(())
        },
    )
    .expect("streamed camera loop should submit each camera");

    assert_eq!(
        observed,
        vec![
            (Some(11), Some(UVec2::new(96, 48)), false, false),
            (Some(22), Some(UVec2::new(160, 80)), true, true),
        ]
    );
}

#[test]
fn camera_loop_routes_ui_to_last_primary_stack_terminal_only() {
    let first_primary = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack([2]);
    let first_overlay = descriptor(
        0,
        2,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/intermediate-texture",
    ));
    let texture_base = descriptor(
        4,
        3,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(texture),
    );
    let last_primary = descriptor(
        8,
        4,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack([5]);
    let last_primary_overlay = descriptor(
        8,
        5,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        empty_scene_snapshot(),
    );
    extract.view = extract.view.with_cameras(vec![
        first_primary,
        first_overlay,
        texture_base,
        last_primary,
        last_primary_overlay,
    ]);

    let submissions = camera_loop_submissions(&extract).expect("active camera sequence");

    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.camera.entity)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3), Some(4), Some(5)]
    );
    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.receives_terminal_ui)
            .collect::<Vec<_>>(),
        vec![false, false, false, false, true]
    );
}

#[test]
fn camera_loop_routes_ui_to_last_base_when_no_primary_base_exists() {
    let first_texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/offscreen-first",
    ));
    let second_texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/offscreen-second",
    ));
    let first = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(first_texture),
    );
    let second = descriptor(
        2,
        2,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(second_texture),
    );
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        empty_scene_snapshot(),
    );
    extract.view = extract.view.with_cameras(vec![first, second]);

    let submissions = camera_loop_submissions(&extract).expect("active camera sequence");

    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.camera.entity)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2)]
    );
    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.receives_terminal_ui)
            .collect::<Vec<_>>(),
        vec![false, true]
    );
}

#[test]
fn camera_loop_marks_stack_and_viewport_output_owners() {
    let first_primary = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack([2]);
    let first_overlay = descriptor(
        0,
        2,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/output-owner-texture",
    ));
    let texture_base = descriptor(
        4,
        3,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(texture),
    );
    let last_primary = descriptor(
        8,
        4,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack([5]);
    let last_primary_overlay = descriptor(
        8,
        5,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        empty_scene_snapshot(),
    );
    extract.view = extract.view.with_cameras(vec![
        first_primary,
        first_overlay,
        texture_base,
        last_primary,
        last_primary_overlay,
    ]);

    let submissions = camera_loop_submissions(&extract).expect("active camera sequence");

    assert_eq!(
        submissions
            .iter()
            .map(|submission| submission.output_policy)
            .collect::<Vec<_>>(),
        vec![
            CameraLoopOutputPolicy::new(false, false),
            CameraLoopOutputPolicy::new(true, false),
            CameraLoopOutputPolicy::new(true, false),
            CameraLoopOutputPolicy::new(false, false),
            CameraLoopOutputPolicy::new(true, true),
        ]
    );
    assert_eq!(
        submissions
            .iter()
            .map(|submission| {
                let policy = ViewportCameraStackOutputPolicy::from(submission.output_policy);
                (
                    policy.is_stack_terminal(),
                    policy.is_viewport_terminal(),
                    policy.owns_final_target_output(),
                    policy.owns_viewport_submission(),
                    policy.owns_shared_viewport_products(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (false, false, false, false, false),
            (true, false, true, false, false),
            (true, false, true, false, false),
            (false, false, false, false, false),
            (true, true, true, true, true),
        ]
    );
}

#[test]
fn viewport_terminal_camera_target_uses_last_primary_stack_terminal() {
    let first_primary = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack([2]);
    let first_overlay = descriptor(
        0,
        2,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/terminal-texture",
    ));
    let texture_base = descriptor(
        4,
        3,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(texture),
    );
    let last_primary = descriptor(
        8,
        4,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack([5]);
    let last_primary_overlay = descriptor(
        8,
        5,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        empty_scene_snapshot(),
    );
    extract.view = extract.view.with_cameras(vec![
        first_primary,
        first_overlay,
        texture_base,
        last_primary,
        last_primary_overlay,
    ]);

    let target = viewport_terminal_camera_target(&extract).expect("terminal target");

    assert!(matches!(target, RenderCameraTarget::PrimarySurface));
}

#[test]
fn viewport_terminal_camera_target_falls_back_to_last_base_without_primary() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-loop/terminal-no-primary-texture",
    ));
    let texture_base = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(texture),
    );
    let headless = descriptor(
        8,
        2,
        CameraRenderType::Base,
        RenderCameraTarget::Headless {
            size: UVec2::new(64, 32),
        },
    );
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        empty_scene_snapshot(),
    );
    extract.view = extract.view.with_cameras(vec![texture_base, headless]);

    let target = viewport_terminal_camera_target(&extract).expect("terminal target");

    assert!(matches!(
        target,
        RenderCameraTarget::Headless {
            size: UVec2 { x: 64, y: 32 }
        }
    ));
}

fn descriptor(
    order: i32,
    entity: u64,
    render_type: CameraRenderType,
    target: RenderCameraTarget,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_order: order,
        render_type,
        target,
        ..CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        )
    }
}

fn empty_scene_snapshot() -> RenderSceneSnapshot {
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
            clear_color: crate::core::math::Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}

trait DescriptorTestExt {
    fn with_stack(self, stack: impl IntoIterator<Item = u64>) -> Self;
    fn with_layers(self, layers: RenderLayerSet) -> Self;
    fn with_viewport_rect(self, viewport_rect: Option<RenderViewportRect>) -> Self;
}

impl DescriptorTestExt for CameraRenderDescriptor {
    fn with_stack(mut self, stack: impl IntoIterator<Item = u64>) -> Self {
        self.stack = stack.into_iter().collect();
        self
    }

    fn with_layers(mut self, layers: RenderLayerSet) -> Self {
        self.culling_mask = layers;
        self
    }

    fn with_viewport_rect(mut self, viewport_rect: Option<RenderViewportRect>) -> Self {
        self.viewport_rect = viewport_rect;
        self
    }
}
