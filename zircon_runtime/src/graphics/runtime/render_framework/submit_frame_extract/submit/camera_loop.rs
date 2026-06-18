use crate::core::framework::render::{
    resolve_camera_sequence, CameraRenderDescriptor, CameraSequenceEntry, RenderCameraTarget,
    RenderFrameExtract, RenderFrameworkError, RenderViewportHandle,
};
use crate::graphics::ViewportCameraStackOutputPolicy;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::super::super::wgpu_render_framework::WgpuRenderFramework;

pub(super) fn submit_camera_loop(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
    ui: Option<UiRenderExtract>,
    submit_selected_camera: impl Fn(
        &WgpuRenderFramework,
        RenderViewportHandle,
        RenderFrameExtract,
        Option<UiRenderExtract>,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    for camera_submit in camera_loop_submissions(&extract)? {
        let selected_ui = if camera_submit.receives_terminal_ui {
            ui.clone()
        } else {
            None
        };
        submit_selected_camera(
            server,
            viewport,
            camera_submit.extract,
            selected_ui,
            camera_submit.output_policy,
        )?;
    }

    Ok(())
}

#[cfg(test)]
fn camera_loop_extracts(
    extract: &RenderFrameExtract,
) -> Result<Vec<RenderFrameExtract>, RenderFrameworkError> {
    let sequence = resolve_camera_sequence(extract.view.cameras.clone());
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

fn camera_loop_submissions(
    extract: &RenderFrameExtract,
) -> Result<Vec<CameraLoopSubmission>, RenderFrameworkError> {
    let sequence = resolve_camera_sequence(extract.view.cameras.clone());
    if sequence.sequence.is_empty() {
        return Err(RenderFrameworkError::UnsupportedCapability {
            capability: "active camera sequence".to_string(),
        });
    }

    Ok(camera_sequence_submission_descriptors(sequence.sequence)
        .into_iter()
        .map(|submission| CameraLoopSubmission {
            receives_terminal_ui: submission.receives_terminal_ui,
            output_policy: submission.output_policy,
            extract: extract
                .clone()
                .with_selected_camera_descriptor(submission.camera),
        })
        .collect())
}

#[cfg(test)]
fn camera_sequence_descriptors(
    sequence: crate::core::framework::render::CameraSequenceReport,
) -> Vec<CameraRenderDescriptor> {
    camera_sequence_submission_descriptors(sequence.sequence)
        .into_iter()
        .map(|submission| submission.camera)
        .collect()
}

fn camera_sequence_submission_descriptors(
    sequence: Vec<CameraSequenceEntry>,
) -> Vec<CameraDescriptorSubmission> {
    let terminal_ui_position = terminal_screen_space_ui_camera_position(&sequence);
    sequence
        .into_iter()
        .enumerate()
        .flat_map(|(base_index, entry)| {
            let stack_terminal_camera_index = entry.overlays.len();
            let base_receives_terminal_ui = terminal_ui_position
                == Some(CameraLoopDescriptorPosition {
                    base_index,
                    camera_index: 0,
                });
            std::iter::once(CameraDescriptorSubmission {
                camera: entry.base,
                receives_terminal_ui: base_receives_terminal_ui,
                output_policy: CameraLoopOutputPolicy::new(
                    stack_terminal_camera_index == 0,
                    terminal_ui_position
                        == Some(CameraLoopDescriptorPosition {
                            base_index,
                            camera_index: 0,
                        }),
                ),
            })
            .chain(entry.overlays.into_iter().enumerate().map(
                move |(overlay_index, camera)| CameraDescriptorSubmission {
                    camera,
                    receives_terminal_ui: terminal_ui_position
                        == Some(CameraLoopDescriptorPosition {
                            base_index,
                            camera_index: overlay_index + 1,
                        }),
                    output_policy: CameraLoopOutputPolicy::new(
                        overlay_index + 1 == stack_terminal_camera_index,
                        terminal_ui_position
                            == Some(CameraLoopDescriptorPosition {
                                base_index,
                                camera_index: overlay_index + 1,
                            }),
                    ),
                },
            ))
        })
        .collect()
}

fn terminal_screen_space_ui_camera_position(
    sequence: &[CameraSequenceEntry],
) -> Option<CameraLoopDescriptorPosition> {
    let base_index = sequence
        .iter()
        .enumerate()
        .rev()
        .find(|(_, entry)| matches!(&entry.base.target, RenderCameraTarget::PrimarySurface))
        .map(|(index, _)| index)
        .or_else(|| sequence.len().checked_sub(1))?;
    Some(CameraLoopDescriptorPosition {
        base_index,
        camera_index: sequence[base_index].overlays.len(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CameraLoopDescriptorPosition {
    base_index: usize,
    camera_index: usize,
}

struct CameraDescriptorSubmission {
    camera: CameraRenderDescriptor,
    receives_terminal_ui: bool,
    output_policy: CameraLoopOutputPolicy,
}

struct CameraLoopSubmission {
    extract: RenderFrameExtract,
    receives_terminal_ui: bool,
    output_policy: CameraLoopOutputPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CameraLoopOutputPolicy {
    stack_terminal: bool,
    viewport_terminal: bool,
}

impl CameraLoopOutputPolicy {
    const fn new(stack_terminal: bool, viewport_terminal: bool) -> Self {
        Self {
            stack_terminal,
            viewport_terminal,
        }
    }
}

impl From<CameraLoopOutputPolicy> for ViewportCameraStackOutputPolicy {
    fn from(value: CameraLoopOutputPolicy) -> Self {
        Self::new(value.stack_terminal, value.viewport_terminal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        CameraRenderType, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderCameraTarget,
        RenderLayerSet, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::UVec2;
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};

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
                .to_legacy_mask_lossy(),
            1 << 2
        );
        assert_eq!(
            extracts[1]
                .view
                .selected_camera_layers()
                .to_legacy_mask_lossy(),
            1 << 5
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
                .map(|submission| submission.extract.view.scene_camera_entity)
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
                .map(|submission| submission.extract.view.scene_camera_entity)
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
                        policy.writes_output_target(),
                        policy.owns_viewport_submission(),
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (false, false, false, false),
                (true, false, true, false),
                (true, false, true, false),
                (false, false, false, false),
                (true, true, true, true),
            ]
        );
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
    }
}
