use crate::core::framework::render::{
    derive_planar_reflection_camera, resolve_camera_sequence_borrowed, CameraRenderDescriptor,
    CameraSequenceEntry, PlanarReflectionUpdateState, RenderCameraTarget, RenderFrameExtract,
    RenderFrameworkError, RenderViewportHandle, UiRenderSubmission,
};
use crate::graphics::visibility::FrameVisibility;
use crate::graphics::{
    ViewportCameraStackOutputPolicy, ViewportRenderFrame, ViewportRenderOutputTarget,
};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use super::super::super::wgpu_render_framework::WgpuRenderFrameworkAccess;
pub(super) fn submit_camera_loop(
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
    ui: Option<Arc<UiRenderSubmission>>,
    submit_started: &Instant,
    submit_selected_camera: impl Fn(
        &dyn WgpuRenderFrameworkAccess,
        RenderViewportHandle,
        &Arc<RenderFrameExtract>,
        Option<Arc<UiRenderSubmission>>,
        &Instant,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let plan = camera_loop_submissions_for_submit(framework, &extract)?;
    let submission_count = plan.submissions.len();
    let result = stream_camera_loop_extract_submissions(
        extract,
        ui,
        plan.submissions,
        |extract, ui, output_policy| {
            submit_selected_camera(
                framework,
                viewport,
                extract,
                ui,
                submit_started,
                output_policy,
            )
        },
    );
    if result.is_ok() {
        record_successful_camera_loop(framework, submission_count, &plan.planar_probe_ids);
    }
    result
}

pub(super) fn viewport_terminal_camera_target(
    extract: &RenderFrameExtract,
) -> Result<RenderCameraTarget, RenderFrameworkError> {
    let sequence = resolve_camera_sequence_borrowed(&extract.view.cameras).sequence;
    if sequence.is_empty() {
        return Err(RenderFrameworkError::UnsupportedCapability {
            capability: "active camera sequence".to_string(),
        });
    }
    let terminal = terminal_screen_space_ui_camera_position(&sequence)
        .and_then(|position| {
            let entry = sequence.get(position.base_index)?;
            if position.camera_index == 0 {
                Some(&entry.base)
            } else {
                entry.overlays.get(position.camera_index - 1)
            }
        })
        .ok_or_else(|| RenderFrameworkError::UnsupportedCapability {
            capability: "viewport-terminal camera".to_string(),
        })?;
    Ok(terminal.target.clone())
}

fn camera_loop_submissions(
    extract: &RenderFrameExtract,
) -> Result<Vec<CameraLoopSubmission>, RenderFrameworkError> {
    camera_loop_submissions_from_cameras(&extract.view.cameras)
}

fn camera_loop_submissions_from_cameras(
    cameras: &[CameraRenderDescriptor],
) -> Result<Vec<CameraLoopSubmission>, RenderFrameworkError> {
    let sequence = resolve_camera_sequence_borrowed(cameras);
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
    framework: &dyn WgpuRenderFrameworkAccess,
    viewport: RenderViewportHandle,
    frame: ViewportRenderFrame,
    submit_started: &Instant,
    fail_preflight_error: impl Fn(
        &dyn WgpuRenderFrameworkAccess,
        RenderViewportHandle,
        &RenderFrameworkError,
    ),
    mut submit_selected_frame: impl FnMut(
        &dyn WgpuRenderFrameworkAccess,
        RenderViewportHandle,
        &mut ViewportRenderFrame,
        &Instant,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let plan = match camera_loop_submissions_for_submit(framework, &frame.extract) {
        Ok(plan) => plan,
        Err(error) => {
            fail_preflight_error(framework, viewport, &error);
            return Err(error);
        }
    };

    let submission_count = plan.submissions.len();
    let result =
        stream_camera_loop_frame_submissions(frame, plan.submissions, |frame, output_policy| {
            submit_selected_frame(framework, viewport, frame, submit_started, output_policy)
        });
    if result.is_ok() {
        record_successful_camera_loop(framework, submission_count, &plan.planar_probe_ids);
    }
    result
}

struct CameraLoopSubmissionPlan {
    submissions: Vec<CameraLoopSubmission>,
    planar_probe_ids: Vec<u64>,
}

fn camera_loop_submissions_for_submit(
    framework: &dyn WgpuRenderFrameworkAccess,
    extract: &RenderFrameExtract,
) -> Result<CameraLoopSubmissionPlan, RenderFrameworkError> {
    let updates = framework.lock_planar_reflection_updates();
    camera_loop_submissions_with_planar_updates(extract, &updates)
}

fn camera_loop_submissions_with_planar_updates(
    extract: &RenderFrameExtract,
    updates: &PlanarReflectionUpdateState,
) -> Result<CameraLoopSubmissionPlan, RenderFrameworkError> {
    let mut augmented_cameras = None::<Vec<CameraRenderDescriptor>>;
    let mut planar_probe_ids = Vec::new();
    let mut submitted_texture_targets = extract
        .view
        .cameras
        .iter()
        .filter_map(|camera| match &camera.target {
            RenderCameraTarget::Texture(target) => Some(target.id()),
            _ => None,
        })
        .collect::<HashSet<_>>();
    if let Some(main_camera) = extract.view.selected_camera_descriptor() {
        for probe in &extract.lighting.advanced_lighting.planar_probes {
            let Some(target) = probe.capture_target() else {
                continue;
            };
            if !updates.should_capture(probe) || submitted_texture_targets.contains(&target.id()) {
                continue;
            }
            if let Some(camera) = derive_planar_reflection_camera(main_camera, probe, target) {
                submitted_texture_targets.insert(target.id());
                augmented_cameras
                    .get_or_insert_with(|| extract.view.cameras.clone())
                    .push(camera);
                planar_probe_ids.push(probe.probe_id);
            }
        }
    }
    let cameras = augmented_cameras
        .as_deref()
        .unwrap_or(&extract.view.cameras);
    Ok(CameraLoopSubmissionPlan {
        submissions: camera_loop_submissions_from_cameras(cameras)?,
        planar_probe_ids,
    })
}

fn record_successful_camera_loop(
    framework: &dyn WgpuRenderFrameworkAccess,
    submission_count: usize,
    planar_probe_ids: &[u64],
) {
    {
        let mut updates = framework.lock_planar_reflection_updates();
        for probe_id in planar_probe_ids {
            updates.mark_captured(*probe_id);
        }
    }
    framework
        .lock_state()
        .stats
        .last_camera_loop_submission_count = submission_count;
}

fn stream_camera_loop_frame_submissions(
    mut frame: ViewportRenderFrame,
    submissions: Vec<CameraLoopSubmission>,
    mut submit_selected_frame: impl FnMut(
        &mut ViewportRenderFrame,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let source_state = (submissions.len() > 1).then(|| CameraLoopFrameSourceState::capture(&frame));
    let mut terminal_ui = frame.ui.take();

    for (submission_index, submission) in submissions.into_iter().enumerate() {
        if submission_index > 0 {
            if let Some(source_state) = source_state.as_ref() {
                source_state.restore_for_submission(&mut frame);
            }
        }
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
    ui: Option<Arc<UiRenderSubmission>>,
    submissions: Vec<CameraLoopSubmission>,
    mut submit_selected_camera: impl FnMut(
        &Arc<RenderFrameExtract>,
        Option<Arc<UiRenderSubmission>>,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let source_extract = extract;
    let mut terminal_ui = ui;

    for submission in submissions {
        let submission_extract = Arc::new(source_extract.for_camera_submission(submission.camera));
        let selected_ui = if submission.receives_terminal_ui {
            terminal_ui.take()
        } else {
            None
        };
        submit_selected_camera(&submission_extract, selected_ui, submission.output_policy)?;
    }

    Ok(())
}

fn select_frame_camera_for_submission(
    frame: &mut ViewportRenderFrame,
    camera: CameraRenderDescriptor,
) {
    frame.select_camera_descriptor(camera);
}

struct CameraLoopFrameSourceState {
    viewport_size: crate::core::math::UVec2,
    extract: Arc<RenderFrameExtract>,
    output_target: ViewportRenderOutputTarget,
    frame_visibility: Option<FrameVisibility>,
}

impl CameraLoopFrameSourceState {
    fn capture(frame: &ViewportRenderFrame) -> Self {
        Self {
            viewport_size: frame.viewport_size,
            extract: Arc::clone(&frame.extract),
            output_target: frame.output_target,
            frame_visibility: frame.frame_visibility.clone(),
        }
    }

    fn restore_for_submission(&self, frame: &mut ViewportRenderFrame) {
        // Per-camera submit mutates derived frame fields; restore only the source fields that
        // affect the next selected-camera context instead of cloning the whole frame.
        frame.viewport_size = self.viewport_size;
        frame.extract = Arc::clone(&self.extract);
        frame.output_target = self.output_target;
        frame.frame_visibility = self.frame_visibility.clone();
    }
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
                )
                .with_viewport_submission_start(base_index == 0),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CameraLoopOutputPolicy {
    viewport_submission_start: bool,
    stack_terminal: bool,
    viewport_terminal: bool,
}

impl CameraLoopOutputPolicy {
    const fn new(stack_terminal: bool, viewport_terminal: bool) -> Self {
        Self {
            viewport_submission_start: false,
            stack_terminal,
            viewport_terminal,
        }
    }

    const fn with_viewport_submission_start(mut self, start: bool) -> Self {
        self.viewport_submission_start = start;
        self
    }
}

impl From<CameraLoopOutputPolicy> for ViewportCameraStackOutputPolicy {
    fn from(value: CameraLoopOutputPolicy) -> Self {
        Self::new(value.stack_terminal, value.viewport_terminal)
            .with_viewport_submission_start(value.viewport_submission_start)
    }
}

#[cfg(test)]
mod tests;
