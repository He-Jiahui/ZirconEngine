use crate::core::framework::render::{
    resolve_camera_sequence, CameraRenderDescriptor, CameraSequenceEntry, PostProcessExtract,
    RenderCameraTarget, RenderFrameExtract, RenderFrameworkError, RenderHybridGiExtract,
    RenderViewExtract, RenderViewportHandle, RenderVirtualGeometryExtract,
};
use crate::graphics::visibility::FrameVisibility;
use crate::graphics::{
    ViewportCameraStackOutputPolicy, ViewportRenderFrame, ViewportRenderOutputTarget,
};
use std::sync::Arc;
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
        &mut Arc<RenderFrameExtract>,
        Option<UiRenderExtract>,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let submissions = camera_loop_submissions(&extract)?;
    stream_camera_loop_extract_submissions(
        extract,
        ui,
        submissions,
        |extract, ui, output_policy| {
            submit_selected_camera(server, viewport, extract, ui, output_policy)
        },
    )
}

pub(super) fn viewport_terminal_camera_target(
    extract: &RenderFrameExtract,
) -> Result<RenderCameraTarget, RenderFrameworkError> {
    camera_loop_submissions(extract)?
        .into_iter()
        .find(|submission| {
            ViewportCameraStackOutputPolicy::from(submission.output_policy)
                .owns_viewport_submission()
        })
        .map(|submission| submission.camera.target.clone())
        .ok_or_else(|| RenderFrameworkError::UnsupportedCapability {
            capability: "viewport-terminal camera".to_string(),
        })
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
        .map(CameraLoopSubmission::from)
        .collect())
}

pub(super) fn submit_camera_loop_frame(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    frame: ViewportRenderFrame,
    fail_preflight_error: impl Fn(&WgpuRenderFramework, RenderViewportHandle, &RenderFrameworkError),
    mut submit_selected_frame: impl FnMut(
        &WgpuRenderFramework,
        RenderViewportHandle,
        &mut ViewportRenderFrame,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let submissions = match camera_loop_submissions(&frame.extract) {
        Ok(submissions) => submissions,
        Err(error) => {
            fail_preflight_error(server, viewport, &error);
            return Err(error);
        }
    };

    stream_camera_loop_frame_submissions(frame, submissions, |frame, output_policy| {
        submit_selected_frame(server, viewport, frame, output_policy)
    })
}

fn stream_camera_loop_frame_submissions(
    mut frame: ViewportRenderFrame,
    submissions: Vec<CameraLoopSubmission>,
    mut submit_selected_frame: impl FnMut(
        &mut ViewportRenderFrame,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let source_state = CameraLoopFrameSourceState::capture(&frame);
    let mut terminal_ui = frame.ui.take();

    for submission in submissions {
        source_state.restore_for_submission(&mut frame);
        select_frame_camera_for_submission(&mut frame, submission.camera);
        frame.ui = if submission.receives_terminal_ui {
            terminal_ui.take()
        } else {
            None
        };
        submit_selected_frame(&mut frame, submission.output_policy)?;
    }

    Ok(())
}

fn stream_camera_loop_extract_submissions(
    extract: RenderFrameExtract,
    ui: Option<UiRenderExtract>,
    submissions: Vec<CameraLoopSubmission>,
    mut submit_selected_camera: impl FnMut(
        &mut Arc<RenderFrameExtract>,
        Option<UiRenderExtract>,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let source_state = CameraLoopExtractSourceState::capture(&extract);
    let mut source_extract = Arc::new(extract);
    let mut terminal_ui = ui;

    for submission in submissions {
        let extract = Arc::make_mut(&mut source_extract);
        source_state.restore_for_submission(extract);
        extract.select_camera_descriptor(submission.camera);
        let selected_ui = if submission.receives_terminal_ui {
            terminal_ui.take()
        } else {
            None
        };
        submit_selected_camera(&mut source_extract, selected_ui, submission.output_policy)?;
    }

    Ok(())
}

fn select_frame_camera_for_submission(
    frame: &mut ViewportRenderFrame,
    camera: CameraRenderDescriptor,
) {
    frame.extract_mut().select_camera_descriptor(camera);
}

#[cfg(test)]
pub(super) fn camera_loop_frame_submissions(
    frame: ViewportRenderFrame,
) -> Result<Vec<CameraLoopFrameSubmission>, RenderFrameworkError> {
    let submissions = camera_loop_submissions(&frame.extract)?;
    let terminal_submission_index = submissions.len().saturating_sub(1);
    let mut source_frame = Some(frame);
    let mut frame_submissions = Vec::with_capacity(submissions.len());

    for (index, submission) in submissions.into_iter().enumerate() {
        let receives_terminal_ui = submission.receives_terminal_ui;
        let mut projected_frame = if index == terminal_submission_index {
            let Some(frame) = source_frame.take() else {
                return Err(camera_loop_source_frame_consumed_error());
            };
            project_owned_frame_to_selected_camera(frame, submission.camera)
        } else {
            let Some(frame) = source_frame.as_ref() else {
                return Err(camera_loop_source_frame_consumed_error());
            };
            project_borrowed_frame_to_selected_camera(frame, submission.camera)
        };
        if !receives_terminal_ui {
            projected_frame = projected_frame.with_ui(None);
        }
        frame_submissions.push(CameraLoopFrameSubmission {
            frame: projected_frame,
            receives_terminal_ui,
            output_policy: submission.output_policy,
        });
    }

    Ok(frame_submissions)
}

#[cfg(test)]
fn project_borrowed_frame_to_selected_camera(
    frame: &ViewportRenderFrame,
    camera: CameraRenderDescriptor,
) -> ViewportRenderFrame {
    let extract = frame
        .extract
        .as_ref()
        .clone()
        .with_selected_camera_descriptor(camera);
    let mut projected = ViewportRenderFrame::from_extract(extract, frame.viewport_size)
        .with_shader_quality(frame.shader_quality())
        .with_output_target(frame.output_target())
        .with_ui(frame.ui.clone())
        .with_previous_motion_vector_camera(frame.previous_motion_vector_camera().cloned())
        .with_virtual_geometry_debug_snapshot(frame.virtual_geometry_debug_snapshot.clone())
        .with_camera_stack_output_policy(frame.camera_stack_output_policy());
    if let Some(frame_visibility) = frame.frame_visibility.clone() {
        projected = projected.with_frame_visibility(frame_visibility);
    }
    projected.scene = frame.scene.clone();
    projected
}

#[cfg(test)]
fn project_owned_frame_to_selected_camera(
    frame: ViewportRenderFrame,
    camera: CameraRenderDescriptor,
) -> ViewportRenderFrame {
    let ViewportRenderFrame {
        scene,
        extract,
        viewport_size,
        shader_quality,
        ui,
        output_target,
        previous_motion_vector_camera,
        frame_visibility,
        virtual_geometry_debug_snapshot,
        prepared_runtime_sidebands,
        camera_stack_output_policy,
        ..
    } = frame;
    let extract = std::sync::Arc::try_unwrap(extract)
        .unwrap_or_else(|extract| (*extract).clone())
        .with_selected_camera_descriptor(camera);
    let mut projected = ViewportRenderFrame::from_extract(extract, viewport_size)
        .with_shader_quality(shader_quality)
        .with_output_target(output_target)
        .with_ui(ui)
        .with_previous_motion_vector_camera(previous_motion_vector_camera)
        .with_virtual_geometry_debug_snapshot(virtual_geometry_debug_snapshot)
        .with_prepared_runtime_sidebands(prepared_runtime_sidebands)
        .with_camera_stack_output_policy(camera_stack_output_policy);
    if let Some(frame_visibility) = frame_visibility {
        projected = projected.with_frame_visibility(frame_visibility);
    }
    projected.scene = scene;
    projected
}

struct CameraLoopExtractSourceState {
    view: RenderViewExtract,
    post_process: PostProcessExtract,
    virtual_geometry: Option<RenderVirtualGeometryExtract>,
    hybrid_global_illumination: Option<RenderHybridGiExtract>,
}

impl CameraLoopExtractSourceState {
    fn capture(extract: &RenderFrameExtract) -> Self {
        Self {
            view: extract.view.clone(),
            post_process: extract.post_process.clone(),
            virtual_geometry: extract.geometry.virtual_geometry.clone(),
            hybrid_global_illumination: extract.lighting.hybrid_global_illumination.clone(),
        }
    }

    fn restore_for_submission(&self, extract: &mut RenderFrameExtract) {
        extract.view = self.view.clone();
        extract.post_process = self.post_process.clone();
        extract.geometry.virtual_geometry = self.virtual_geometry.clone();
        extract.lighting.hybrid_global_illumination = self.hybrid_global_illumination.clone();
    }
}

struct CameraLoopFrameSourceState {
    viewport_size: crate::core::math::UVec2,
    view_target_size: Option<crate::core::math::UVec2>,
    output_target: ViewportRenderOutputTarget,
    frame_visibility: Option<FrameVisibility>,
    post_process: PostProcessExtract,
    virtual_geometry: Option<RenderVirtualGeometryExtract>,
    hybrid_global_illumination: Option<RenderHybridGiExtract>,
}

impl CameraLoopFrameSourceState {
    fn capture(frame: &ViewportRenderFrame) -> Self {
        Self {
            viewport_size: frame.viewport_size,
            view_target_size: frame.extract.view.target_size,
            output_target: frame.output_target,
            frame_visibility: frame.frame_visibility.clone(),
            post_process: frame.extract.post_process.clone(),
            virtual_geometry: frame.extract.geometry.virtual_geometry.clone(),
            hybrid_global_illumination: frame.extract.lighting.hybrid_global_illumination.clone(),
        }
    }

    fn restore_for_submission(&self, frame: &mut ViewportRenderFrame) {
        // Per-camera submit mutates derived frame fields; restore only the source fields that
        // affect the next selected-camera context instead of cloning the whole frame.
        frame.viewport_size = self.viewport_size;
        frame.extract_mut().view.target_size = self.view_target_size;
        frame.output_target = self.output_target;
        frame.frame_visibility = self.frame_visibility.clone();
        let extract = frame.extract_mut();
        extract.post_process = self.post_process.clone();
        extract.geometry.virtual_geometry = self.virtual_geometry.clone();
        extract.lighting.hybrid_global_illumination = self.hybrid_global_illumination.clone();
    }
}

#[cfg(test)]
fn camera_loop_source_frame_consumed_error() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: "camera-loop source frame consumed before terminal camera".to_string(),
    }
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
    camera: CameraRenderDescriptor,
    receives_terminal_ui: bool,
    output_policy: CameraLoopOutputPolicy,
}

impl From<CameraDescriptorSubmission> for CameraLoopSubmission {
    fn from(value: CameraDescriptorSubmission) -> Self {
        Self {
            camera: value.camera,
            receives_terminal_ui: value.receives_terminal_ui,
            output_policy: value.output_policy,
        }
    }
}

#[cfg(test)]
pub(super) struct CameraLoopFrameSubmission {
    pub(super) frame: ViewportRenderFrame,
    pub(super) receives_terminal_ui: bool,
    pub(super) output_policy: CameraLoopOutputPolicy,
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
        RenderLayerSet, RenderOverlayExtract, RenderParticleGpuReadbackOutputs,
        RenderPluginRendererOutputs, RenderPreparedRuntimeSidebands, RenderSceneGeometryExtract,
        RenderSceneSnapshot, RenderViewportRect, RenderVirtualGeometryExtract,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec4};
    use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
    use crate::graphics::ViewportRenderFrame;
    use zircon_runtime_interface::ui::surface::UiRenderExtract;

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
            |extract, ui, output_policy| {
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

    #[test]
    fn camera_loop_frame_submissions_project_selected_children_and_terminal_ui() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "tests/camera-loop/frame-texture",
        ));
        let base = descriptor(
            0,
            1,
            CameraRenderType::Base,
            RenderCameraTarget::Texture(texture),
        )
        .with_viewport_rect(Some(RenderViewportRect::new(
            UVec2::new(0, 0),
            UVec2::new(32, 64),
        )))
        .with_stack([2]);
        let overlay = descriptor(0, 2, CameraRenderType::Overlay, base.target.clone())
            .with_viewport_rect(Some(RenderViewportRect::new(
                UVec2::new(8, 0),
                UVec2::new(24, 64),
            )));
        let primary = descriptor(
            4,
            3,
            CameraRenderType::Base,
            RenderCameraTarget::PrimarySurface,
        )
        .with_viewport_rect(Some(RenderViewportRect::new(
            UVec2::new(32, 0),
            UVec2::new(32, 64),
        )));
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(2),
            empty_scene_snapshot(),
        );
        extract.view = extract.view.with_cameras(vec![base, overlay, primary]);
        let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
            .with_ui(Some(UiRenderExtract::default()))
            .with_prepared_runtime_sidebands(RenderPreparedRuntimeSidebands::new(
                RenderPluginRendererOutputs {
                    particles: RenderParticleGpuReadbackOutputs {
                        alive_count: 7,
                        spawned_total: 7,
                        indirect_draw_args: [6, 7, 0, 0],
                        ..RenderParticleGpuReadbackOutputs::default()
                    },
                    ..RenderPluginRendererOutputs::default()
                },
                vec![9],
                vec![13],
            ));
        frame.scene.preview.clear_color = Vec4::new(0.25, 0.5, 0.75, 1.0);

        let submissions = camera_loop_frame_submissions(frame).expect("frame submissions");

        assert_eq!(submissions.len(), 3);
        assert_eq!(
            submissions
                .iter()
                .map(|submission| submission.frame.camera().entity)
                .collect::<Vec<_>>(),
            vec![Some(1), Some(2), Some(3)]
        );
        assert_eq!(
            submissions
                .iter()
                .map(|submission| submission.receives_terminal_ui)
                .collect::<Vec<_>>(),
            vec![false, false, true]
        );
        assert_eq!(
            submissions
                .iter()
                .map(|submission| submission.frame.ui.is_some())
                .collect::<Vec<_>>(),
            vec![false, false, true]
        );
        assert!(
            !ViewportCameraStackOutputPolicy::from(submissions[0].output_policy)
                .owns_final_target_output()
        );
        assert!(
            ViewportCameraStackOutputPolicy::from(submissions[1].output_policy)
                .owns_final_target_output()
        );
        assert!(
            !ViewportCameraStackOutputPolicy::from(submissions[1].output_policy)
                .owns_viewport_submission()
        );
        assert!(
            ViewportCameraStackOutputPolicy::from(submissions[2].output_policy)
                .owns_viewport_submission()
        );
        assert_eq!(
            submissions[0].frame.render_region().physical_size(),
            UVec2::new(32, 64)
        );
        assert_eq!(
            submissions[2].frame.render_region().physical_position(),
            UVec2::new(32, 0)
        );
        assert_eq!(
            submissions
                .iter()
                .map(|submission| submission.frame.scene.preview.clear_color)
                .collect::<Vec<_>>(),
            vec![Vec4::new(0.25, 0.5, 0.75, 1.0); 3]
        );
        assert!(submissions[0].frame.prepared_runtime_sidebands.is_empty());
        assert!(submissions[1].frame.prepared_runtime_sidebands.is_empty());
        assert_eq!(
            submissions[2]
                .frame
                .prepared_runtime_sidebands
                .particle_readback_outputs()
                .alive_count,
            7
        );
        assert_eq!(
            submissions[2]
                .frame
                .prepared_runtime_sidebands
                .hybrid_gi_evictable_probe_ids(),
            &[9]
        );
        assert_eq!(
            submissions[2]
                .frame
                .prepared_runtime_sidebands
                .virtual_geometry_evictable_page_ids(),
            &[13]
        );
    }

    #[test]
    fn submit_camera_loop_frame_streams_selected_children_and_restores_source_fields() {
        let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "tests/camera-loop/stream-texture",
        ));
        let base = descriptor(
            0,
            1,
            CameraRenderType::Base,
            RenderCameraTarget::Texture(texture),
        )
        .with_stack([2]);
        let overlay = descriptor(0, 2, CameraRenderType::Overlay, base.target.clone());
        let primary = descriptor(
            4,
            3,
            CameraRenderType::Base,
            RenderCameraTarget::PrimarySurface,
        );
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(3),
            empty_scene_snapshot(),
        );
        extract.view = extract.view.with_cameras(vec![base, overlay, primary]);
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
            .with_ui(Some(UiRenderExtract::default()));
        let submissions = camera_loop_submissions(&frame.extract).expect("frame submissions");
        let mut seen_cameras = Vec::new();
        let mut terminal_ui = Vec::new();
        let mut output_owners = Vec::new();

        stream_camera_loop_frame_submissions(frame, submissions, |frame, output_policy| {
            seen_cameras.push(frame.camera().entity);
            terminal_ui.push(frame.ui.is_some());
            output_owners.push(
                ViewportCameraStackOutputPolicy::from(output_policy).owns_viewport_submission(),
            );
            assert!(
                frame.extract.geometry.virtual_geometry.is_none(),
                "streaming submit should restore source advanced extract state before each child"
            );
            frame.extract_mut().geometry.virtual_geometry =
                Some(RenderVirtualGeometryExtract::default());
            frame.viewport_size = UVec2::new(7, 7);
            frame.extract_mut().view.target_size = Some(UVec2::new(7, 7));
            Ok(())
        })
        .expect("streamed frame submissions");

        assert_eq!(seen_cameras, vec![Some(1), Some(2), Some(3)]);
        assert_eq!(terminal_ui, vec![false, false, true]);
        assert_eq!(output_owners, vec![false, false, true]);
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
}
