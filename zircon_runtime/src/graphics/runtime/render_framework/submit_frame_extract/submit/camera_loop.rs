use crate::core::framework::render::{
    resolve_camera_sequence_borrowed, CameraRenderDescriptor, CameraSequenceEntry,
    PostProcessExtract, PostProcessPassGraph, PostProcessStackDescriptor, PostProcessVolumeExtract,
    RenderBloomSettings, RenderCameraTarget, RenderColorGradingSettings, RenderFrameExtract,
    RenderFrameworkError, RenderHybridGiExtract, RenderPostProcessEffectStackSettings,
    RenderViewportHandle, RenderVirtualGeometryExtract,
};
use crate::graphics::visibility::FrameVisibility;
use crate::graphics::{
    ViewportCameraStackOutputPolicy, ViewportRenderFrame, ViewportRenderOutputTarget,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::surface::UiRenderExtract;

use super::super::super::wgpu_render_framework::WgpuRenderFramework;
use super::super::build_frame_submission_context::FrameSubmissionSourcePayloads;

pub(super) fn submit_camera_loop(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
    ui: Option<UiRenderExtract>,
    submit_selected_camera: impl Fn(
        &WgpuRenderFramework,
        RenderViewportHandle,
        &mut Arc<RenderFrameExtract>,
        Option<FrameSubmissionSourcePayloads<'_>>,
        Option<UiRenderExtract>,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let submissions = camera_loop_submissions(&extract)?;
    stream_camera_loop_extract_submissions(
        extract,
        ui,
        submissions,
        |extract, source_payloads, ui, output_policy| {
            submit_selected_camera(
                framework,
                viewport,
                extract,
                source_payloads,
                ui,
                output_policy,
            )
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

fn camera_loop_submissions(
    extract: &RenderFrameExtract,
) -> Result<Vec<CameraLoopSubmission>, RenderFrameworkError> {
    let sequence = resolve_camera_sequence_borrowed(&extract.view.cameras);
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
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    frame: ViewportRenderFrame,
    fail_preflight_error: impl Fn(&WgpuRenderFramework, RenderViewportHandle, &RenderFrameworkError),
    mut submit_selected_frame: impl FnMut(
        &WgpuRenderFramework,
        RenderViewportHandle,
        &mut ViewportRenderFrame,
        Option<FrameSubmissionSourcePayloads<'_>>,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let submissions = match camera_loop_submissions(&frame.extract) {
        Ok(submissions) => submissions,
        Err(error) => {
            fail_preflight_error(framework, viewport, &error);
            return Err(error);
        }
    };

    stream_camera_loop_frame_submissions(
        frame,
        submissions,
        |frame, source_payloads, output_policy| {
            submit_selected_frame(framework, viewport, frame, source_payloads, output_policy)
        },
    )
}

fn stream_camera_loop_frame_submissions(
    mut frame: ViewportRenderFrame,
    submissions: Vec<CameraLoopSubmission>,
    mut submit_selected_frame: impl FnMut(
        &mut ViewportRenderFrame,
        Option<FrameSubmissionSourcePayloads<'_>>,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let source_state =
        (submissions.len() > 1).then(|| CameraLoopFrameSourceState::capture(&mut frame));
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
        submit_selected_frame(
            &mut frame,
            source_state
                .as_ref()
                .map(CameraLoopFrameSourceState::source_payloads),
            submission.output_policy,
        )?;
    }

    Ok(())
}

fn stream_camera_loop_extract_submissions(
    extract: RenderFrameExtract,
    ui: Option<UiRenderExtract>,
    submissions: Vec<CameraLoopSubmission>,
    mut submit_selected_camera: impl FnMut(
        &mut Arc<RenderFrameExtract>,
        Option<FrameSubmissionSourcePayloads<'_>>,
        Option<UiRenderExtract>,
        CameraLoopOutputPolicy,
    ) -> Result<(), RenderFrameworkError>,
) -> Result<(), RenderFrameworkError> {
    let mut source_extract = Arc::new(extract);
    let source_state = if submissions.len() > 1 {
        let source = Arc::make_mut(&mut source_extract);
        Some(CameraLoopExtractSourceState::capture(source))
    } else {
        None
    };
    let mut terminal_ui = ui;

    for (submission_index, submission) in submissions.into_iter().enumerate() {
        let extract = Arc::make_mut(&mut source_extract);
        if submission_index > 0 {
            if let Some(source_state) = source_state.as_ref() {
                source_state.restore_for_submission(extract);
            }
        }
        extract.select_camera_descriptor(submission.camera);
        let selected_ui = if submission.receives_terminal_ui {
            terminal_ui.take()
        } else {
            None
        };
        submit_selected_camera(
            &mut source_extract,
            source_state
                .as_ref()
                .map(CameraLoopExtractSourceState::source_payloads),
            selected_ui,
            submission.output_policy,
        )?;
    }

    Ok(())
}

fn select_frame_camera_for_submission(
    frame: &mut ViewportRenderFrame,
    camera: CameraRenderDescriptor,
) {
    frame.extract_mut().select_camera_descriptor(camera);
}

struct CameraLoopExtractSourceState {
    view_target_size: Option<crate::core::math::UVec2>,
    post_process: CameraLoopPostProcessSourceState,
    virtual_geometry: Option<RenderVirtualGeometryExtract>,
    hybrid_global_illumination: Option<RenderHybridGiExtract>,
}

impl CameraLoopExtractSourceState {
    fn capture(extract: &mut RenderFrameExtract) -> Self {
        Self {
            view_target_size: extract.view.target_size,
            post_process: CameraLoopPostProcessSourceState::capture(&extract.post_process),
            virtual_geometry: extract.geometry.virtual_geometry.take(),
            hybrid_global_illumination: extract.lighting.hybrid_global_illumination.take(),
        }
    }

    fn source_payloads(&self) -> FrameSubmissionSourcePayloads<'_> {
        FrameSubmissionSourcePayloads {
            virtual_geometry: self.virtual_geometry.as_ref(),
            hybrid_global_illumination: self.hybrid_global_illumination.as_ref(),
        }
    }

    fn restore_for_submission(&self, extract: &mut RenderFrameExtract) {
        extract.view.target_size = self.view_target_size;
        self.post_process.restore_to(&mut extract.post_process);
    }
}

struct CameraLoopFrameSourceState {
    viewport_size: crate::core::math::UVec2,
    view_target_size: Option<crate::core::math::UVec2>,
    output_target: ViewportRenderOutputTarget,
    frame_visibility: Option<FrameVisibility>,
    post_process: CameraLoopPostProcessSourceState,
    virtual_geometry: Option<RenderVirtualGeometryExtract>,
    hybrid_global_illumination: Option<RenderHybridGiExtract>,
}

impl CameraLoopFrameSourceState {
    fn capture(frame: &mut ViewportRenderFrame) -> Self {
        let (virtual_geometry, hybrid_global_illumination) = {
            let extract = frame.extract_mut();
            (
                extract.geometry.virtual_geometry.take(),
                extract.lighting.hybrid_global_illumination.take(),
            )
        };
        Self {
            viewport_size: frame.viewport_size,
            view_target_size: frame.extract.view.target_size,
            output_target: frame.output_target,
            frame_visibility: frame.frame_visibility.clone(),
            post_process: CameraLoopPostProcessSourceState::capture(&frame.extract.post_process),
            virtual_geometry,
            hybrid_global_illumination,
        }
    }

    fn source_payloads(&self) -> FrameSubmissionSourcePayloads<'_> {
        FrameSubmissionSourcePayloads {
            virtual_geometry: self.virtual_geometry.as_ref(),
            hybrid_global_illumination: self.hybrid_global_illumination.as_ref(),
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
        self.post_process.restore_to(&mut extract.post_process);
    }
}

struct CameraLoopPostProcessSourceState {
    bloom: RenderBloomSettings,
    color_grading: RenderColorGradingSettings,
    effect_stack: RenderPostProcessEffectStackSettings,
    volumes: Vec<PostProcessVolumeExtract>,
    stack: PostProcessStackDescriptor,
    graph: PostProcessPassGraph,
}

impl CameraLoopPostProcessSourceState {
    fn capture(post_process: &PostProcessExtract) -> Self {
        Self {
            bloom: post_process.bloom,
            color_grading: post_process.color_grading,
            effect_stack: post_process.effect_stack,
            volumes: post_process.volumes.clone(),
            stack: post_process.stack.clone(),
            graph: post_process.graph.clone(),
        }
    }

    fn restore_to(&self, post_process: &mut PostProcessExtract) {
        post_process.bloom = self.bloom;
        post_process.color_grading = self.color_grading;
        post_process.effect_stack = self.effect_stack;
        post_process.volumes.clone_from(&self.volumes);
        post_process.stack = self.stack.clone();
        post_process.graph = self.graph.clone();
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
mod tests;
