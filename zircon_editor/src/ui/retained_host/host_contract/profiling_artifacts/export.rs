use std::fs;

use std::path::PathBuf;
use thiserror::Error;

use super::super::chrome_command_stream::{
    build_chrome_command_stream, paint_chrome_command_stream_to_frame,
};
use super::super::data::HostWindowPresentationData;
use super::super::presenter::HostPresenterBackend;
use super::environment::{
    profile_capture_enabled, profile_export_dir, profile_screenshot_capture_enabled,
    ProfileOutputRootError,
};
use super::UiProfileGeometry;
use crate::core::jobs::{
    EditorJob, EditorJobAdmissionRequest, EditorJobBatchAdmissionReservation, EditorJobSpec,
    EditorJobSystem, JobCategory, JobContext, JobError, JobId, JobSubmitError, JobTicket,
};
use crate::ui::retained_host::primitives::PhysicalSize;

const GEOMETRY_FILE: &str = "ui_profile_geometry.json";
const REFERENCE_SCREENSHOT_FILE: &str = "screenshot_reference.png";
const PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES: usize = 256 * 1024;
const RGBA_BYTES_PER_PIXEL: usize = 4;

#[derive(Debug, Error)]
pub(in crate::ui::retained_host::host_contract) enum ProfileArtifactSubmissionError {
    #[error(transparent)]
    InvalidOutputRoot(#[from] ProfileOutputRootError),
    #[error(transparent)]
    Job(#[from] JobSubmitError),
}

struct PresentArtifactExport {
    export_dir: PathBuf,
    geometry: UiProfileGeometry,
    screenshot: Option<ProfileScreenshot>,
}

struct ProfileScreenshot {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

pub(in crate::ui::retained_host::host_contract) fn submit_present_artifacts(
    jobs: &EditorJobSystem,
    size: &PhysicalSize,
    winit_scale_factor: f32,
    capture_sequence: u64,
    backend: HostPresenterBackend,
    submitted_gpu_text: Option<serde_json::Value>,
    materialize_presentation: impl FnOnce() -> HostWindowPresentationData,
) -> Result<Option<JobId>, ProfileArtifactSubmissionError> {
    if !profile_capture_enabled() {
        return Ok(None);
    }
    submit_present_artifacts_with_export_dir(
        jobs,
        size,
        winit_scale_factor,
        capture_sequence,
        backend,
        profile_export_dir(),
        profile_screenshot_capture_enabled(),
        submitted_gpu_text,
        materialize_presentation,
    )
}

fn submit_present_artifacts_with_export_dir(
    jobs: &EditorJobSystem,
    size: &PhysicalSize,
    winit_scale_factor: f32,
    capture_sequence: u64,
    backend: HostPresenterBackend,
    export_dir: Result<Option<PathBuf>, ProfileOutputRootError>,
    screenshot_enabled: bool,
    submitted_gpu_text: Option<serde_json::Value>,
    materialize_presentation: impl FnOnce() -> HostWindowPresentationData,
) -> Result<Option<JobId>, ProfileArtifactSubmissionError> {
    let Some(export_dir) = export_dir? else {
        return Ok(None);
    };
    let export_dir = if capture_sequence == 0 {
        export_dir
    } else {
        export_dir.join(format!("present-{capture_sequence:016}"))
    };
    let estimated_pending_bytes = PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES.saturating_add(
        screenshot_enabled
            .then(|| screenshot_pending_bytes(size))
            .unwrap_or_default(),
    );
    submit_present_artifact_after_admission(jobs, estimated_pending_bytes, || {
        let presentation = materialize_presentation();
        let stream = build_chrome_command_stream(
            &presentation,
            (size.width, size.height),
            None,
            screenshot_enabled,
        );
        let mut geometry = UiProfileGeometry::from_presentation_with_stream_sequence(
            &presentation,
            size,
            winit_scale_factor,
            backend,
            &stream,
            capture_sequence,
        );
        geometry.submitted_gpu_text = submitted_gpu_text;
        let screenshot = screenshot_enabled.then(|| {
            let frame = paint_chrome_command_stream_to_frame(size.width, size.height, &stream);
            ProfileScreenshot {
                width: frame.width(),
                height: frame.height(),
                rgba: frame.into_bytes(),
            }
        });
        PresentArtifactExport {
            export_dir,
            geometry,
            screenshot,
        }
    })
    .map(|ticket| Some(ticket.id()))
    .map_err(Into::into)
}

fn reserve_present_artifact_admission(
    jobs: &EditorJobSystem,
    estimated_pending_bytes: usize,
) -> Result<EditorJobBatchAdmissionReservation, JobSubmitError> {
    jobs.reserve_batch_admission(vec![EditorJobAdmissionRequest::new(
        JobCategory::Export,
        estimated_pending_bytes,
    )])
}

fn submit_present_artifact_after_admission<F>(
    jobs: &EditorJobSystem,
    estimated_pending_bytes: usize,
    materialize: F,
) -> Result<JobTicket<()>, JobSubmitError>
where
    F: FnOnce() -> PresentArtifactExport,
{
    let reservation = reserve_present_artifact_admission(jobs, estimated_pending_bytes)?;
    submit_present_artifact_export(reservation, materialize())
}

fn submit_present_artifact_export(
    reservation: EditorJobBatchAdmissionReservation,
    export: PresentArtifactExport,
) -> Result<JobTicket<()>, JobSubmitError> {
    let estimated_pending_bytes = estimated_pending_bytes(&export);
    let mut tickets = reservation.commit(vec![(
        EditorJobSpec::new("Export UI profile artifacts", JobCategory::Export)
            .with_estimated_bytes(estimated_pending_bytes),
        PresentArtifactExportJob { export },
    )])?;
    Ok(tickets
        .pop()
        .expect("one admitted profile artifact export produces one ticket"))
}

fn estimated_pending_bytes(export: &PresentArtifactExport) -> usize {
    export
        .screenshot
        .as_ref()
        .map_or(0, |screenshot| screenshot.rgba.len())
        .saturating_add(PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES)
}

fn screenshot_pending_bytes(size: &PhysicalSize) -> usize {
    (size.width as usize)
        .saturating_mul(size.height as usize)
        .saturating_mul(RGBA_BYTES_PER_PIXEL)
}

struct PresentArtifactExportJob {
    export: PresentArtifactExport,
}

impl EditorJob for PresentArtifactExportJob {
    type Output = ();

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        write_present_artifacts(self.export, &context)
    }
}

fn write_present_artifacts(
    export: PresentArtifactExport,
    context: &JobContext,
) -> Result<(), JobError> {
    fs::create_dir_all(&export.export_dir).map_err(JobError::failed)?;
    context.check_cancelled()?;
    let bytes = serde_json::to_vec_pretty(&export.geometry).map_err(JobError::failed)?;
    fs::write(export.export_dir.join(GEOMETRY_FILE), bytes).map_err(JobError::failed)?;
    context.check_cancelled()?;
    if let Some(screenshot) = export.screenshot {
        image::save_buffer_with_format(
            export.export_dir.join(REFERENCE_SCREENSHOT_FILE),
            &screenshot.rgba,
            screenshot.width,
            screenshot.height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .map_err(JobError::failed)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/export.rs"]
mod tests;
