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
    backend: HostPresenterBackend,
    materialize_presentation: impl FnOnce() -> HostWindowPresentationData,
) -> Result<Option<JobId>, ProfileArtifactSubmissionError> {
    if !profile_capture_enabled() {
        return Ok(None);
    }
    submit_present_artifacts_with_export_dir(
        jobs,
        size,
        backend,
        profile_export_dir(),
        profile_screenshot_capture_enabled(),
        materialize_presentation,
    )
}

fn submit_present_artifacts_with_export_dir(
    jobs: &EditorJobSystem,
    size: &PhysicalSize,
    backend: HostPresenterBackend,
    export_dir: Result<Option<PathBuf>, ProfileOutputRootError>,
    screenshot_enabled: bool,
    materialize_presentation: impl FnOnce() -> HostWindowPresentationData,
) -> Result<Option<JobId>, ProfileArtifactSubmissionError> {
    let Some(export_dir) = export_dir? else {
        return Ok(None);
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
        let geometry =
            UiProfileGeometry::from_presentation_with_stream(&presentation, size, backend, &stream);
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
mod tests {
    use super::*;
    use std::time::Duration;

    use crate::core::jobs::{
        test_job_system, test_job_system_with_limits, EditorJobAdmissionLimits, EditorJobLimits,
    };

    #[test]
    fn present_artifact_export_runs_as_an_injected_export_job() {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-profile-artifact-job-{}-{:x}",
            std::process::id(),
            fixture_nonce()
        ));
        let _ = fs::remove_dir_all(&root);
        let export = PresentArtifactExport {
            export_dir: root.clone(),
            geometry: UiProfileGeometry::from_presentation(
                &HostWindowPresentationData::default(),
                &PhysicalSize::new(640, 480),
                HostPresenterBackend::Gpu,
            ),
            screenshot: None,
        };

        let jobs = test_job_system();
        let ticket = submit_present_artifact_after_admission(
            &jobs,
            estimated_pending_bytes(&export),
            || export,
        )
        .expect("profile artifact job should be admitted");

        assert_eq!(ticket.wait(), Ok(()));
        assert!(root.join(GEOMETRY_FILE).is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn screenshot_pending_bytes_match_rgba_payload_size_without_overflow() {
        assert_eq!(
            screenshot_pending_bytes(&PhysicalSize::new(640, 480)),
            1_228_800
        );
        assert_eq!(
            screenshot_pending_bytes(&PhysicalSize::new(u32::MAX, u32::MAX)),
            usize::MAX
        );
    }

    #[test]
    fn profile_artifact_admission_reservation_bounds_capture_before_materialization() {
        let jobs = test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
            EditorJobAdmissionLimits::new(
                1,
                PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES,
                Duration::from_secs(60),
            ),
        ));

        let reservation =
            reserve_present_artifact_admission(&jobs, PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES)
                .expect("the first artifact capture reserves the only pending admission slot");
        assert!(
            reserve_present_artifact_admission(&jobs, PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES)
                .is_err(),
            "a later capture must be rejected before it materializes a screenshot"
        );

        drop(reservation);
        assert!(
            reserve_present_artifact_admission(&jobs, PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES)
                .is_ok(),
            "dropping an uncommitted capture must return the shared admission capacity"
        );
    }

    #[test]
    fn profile_artifact_rejection_precedes_export_materialization() {
        let jobs = test_job_system_with_limits(EditorJobLimits::default().with_admission_limits(
            EditorJobAdmissionLimits::new(
                1,
                PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES,
                Duration::from_secs(60),
            ),
        ));
        let _occupied =
            reserve_present_artifact_admission(&jobs, PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES)
                .expect("the only admission slot should be occupied before capture");
        let materialized = std::cell::Cell::new(false);

        let result = submit_present_artifact_after_admission(
            &jobs,
            PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES,
            || -> PresentArtifactExport {
                materialized.set(true);
                panic!("rejected admission must not materialize an export payload");
            },
        );

        assert!(matches!(
            result,
            Err(JobSubmitError::AdmissionEntryLimitExceeded { limit: 1 })
        ));
        assert!(
            !materialized.get(),
            "the rejected capture must not allocate or paint a screenshot"
        );
    }

    #[test]
    fn invalid_profile_output_root_precedes_export_materialization() {
        let materialized = std::cell::Cell::new(false);

        let result = submit_present_artifacts_with_export_dir(
            &test_job_system(),
            &PhysicalSize::new(640, 480),
            HostPresenterBackend::Gpu,
            Err(ProfileOutputRootError),
            false,
            || -> HostWindowPresentationData {
                materialized.set(true);
                panic!("an invalid output root must not materialize an export payload");
            },
        );

        assert!(matches!(
            result,
            Err(ProfileArtifactSubmissionError::InvalidOutputRoot(_))
        ));
        assert!(
            !materialized.get(),
            "the invalid root must be rejected before snapshot materialization"
        );
    }

    fn fixture_nonce() -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::thread::current().id().hash(&mut hasher);
        std::time::SystemTime::now().hash(&mut hasher);
        hasher.finish()
    }
}
