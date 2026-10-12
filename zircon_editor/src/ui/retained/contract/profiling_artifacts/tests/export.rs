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
            1.0,
            HostPresenterBackend::Gpu,
        ),
        screenshot: None,
    };

    let jobs = test_job_system();
    let ticket =
        submit_present_artifact_after_admission(&jobs, estimated_pending_bytes(&export), || export)
            .expect("profile artifact job should be admitted");

    assert_eq!(ticket.wait(), Ok(()));
    assert!(root.join(GEOMETRY_FILE).is_file());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn dynamic_present_sequences_keep_distinct_receipts_and_legacy_root() {
    let root = std::env::temp_dir().join(format!(
        "zircon-editor-profile-sequence-{}-{:x}",
        std::process::id(),
        fixture_nonce()
    ));
    let jobs = test_job_system();
    for capture_sequence in [7, 8, 0] {
        let submitted = submit_present_artifacts_with_export_dir(
            &jobs,
            &PhysicalSize::new(640, 480),
            1.5,
            capture_sequence,
            HostPresenterBackend::Gpu,
            Ok(Some(root.clone())),
            false,
            None,
            HostWindowPresentationData::default,
        )
        .expect("sequence export should be admitted");
        assert!(submitted.is_some());
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let complete = [7, 8, 0].into_iter().all(|capture_sequence| {
            let directory = if capture_sequence == 0 {
                root.clone()
            } else {
                root.join(format!("present-{capture_sequence:016}"))
            };
            fs::read(directory.join(GEOMETRY_FILE))
                .ok()
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                .is_some_and(|geometry| {
                    geometry["capture_sequence"].as_u64() == Some(capture_sequence)
                })
        });
        if complete {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "sequence exports did not complete"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(jobs.shutdown(deadline).is_empty());
    for capture_sequence in [7, 8, 0] {
        let directory = if capture_sequence == 0 {
            root.clone()
        } else {
            root.join(format!("present-{capture_sequence:016}"))
        };
        let geometry: serde_json::Value = serde_json::from_slice(
            &fs::read(directory.join(GEOMETRY_FILE)).expect("sequence receipt exists"),
        )
        .expect("sequence receipt is valid JSON");
        assert_eq!(
            geometry["capture_sequence"].as_u64(),
            Some(capture_sequence)
        );
        assert_eq!(geometry["presenter_backend"], "gpu");
        assert_eq!(geometry["window_client_size"]["width"], 640);
        assert_eq!(geometry["winit_scale_factor"], 1.5);
    }
    fs::remove_dir_all(root).expect("sequence fixture cleanup");
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
        reserve_present_artifact_admission(&jobs, PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES).is_err(),
        "a later capture must be rejected before it materializes a screenshot"
    );

    drop(reservation);
    assert!(
        reserve_present_artifact_admission(&jobs, PROFILE_ARTIFACT_GEOMETRY_PENDING_BYTES).is_ok(),
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
        1.0,
        0,
        HostPresenterBackend::Gpu,
        Err(ProfileOutputRootError),
        false,
        None,
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
