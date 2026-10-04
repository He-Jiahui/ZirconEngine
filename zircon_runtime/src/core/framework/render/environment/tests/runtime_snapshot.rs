use std::sync::Arc;

use super::*;
use crate::core::framework::render::{
    IblBakeArtifactRequest, IblBakeKey, RealtimeIblReadiness, RenderEnvironmentCaptureHandle,
    RenderEnvironmentCapturePhase, RenderPassProfileEntry, RenderSceneSubmissionCompletionStatus,
};

fn hydration_report() -> EnvironmentIblHydrationReport {
    let request = IblBakeArtifactRequest::new(IblBakeKey::source_cubemap(23, [5, 6, 7, 8]), 128, 8);
    let mut resident_requests = [None; ENVIRONMENT_IBL_HYDRATION_REPORT_CAPACITY];
    resident_requests[0] = Some(request);
    EnvironmentIblHydrationReport {
        observation_epoch: 17,
        resident_count: 1,
        resident_requests,
        ..EnvironmentIblHydrationReport::default()
    }
}

fn realtime_ibl_status() -> RealtimeIblStatusReport {
    RealtimeIblStatusReport {
        readiness: RealtimeIblReadiness::RefreshingLastGood,
        current_frame_number: 91,
        published_key: Some(IblBakeKey::source_cubemap(17, [1, 2, 3, 4])),
        pending_key: None,
        queued_key: None,
        published_generation_frame_number: Some(83),
        last_good_age_frame_count: Some(8),
        active_generation_start_frame_number: Some(90),
        active_generation_elapsed_frame_count: Some(2),
        active_generation_coalesced_source_change_count: 3,
        failure: None,
    }
}

fn capture_report() -> RenderEnvironmentCaptureReport {
    RenderEnvironmentCaptureReport {
        observation_epoch: 23,
        pending_handle: RenderEnvironmentCaptureHandle::new(4),
        pending_output_generation: Some(9),
        active_phase: Some(RenderEnvironmentCapturePhase::Filtering),
        active_completed_work_items: 6,
        ..RenderEnvironmentCaptureReport::default()
    }
}

fn cubemap_upload_report() -> EnvironmentCubemapUploadReport {
    EnvironmentCubemapUploadReport {
        observation_epoch: 29,
        committed_upload_key: SourceCubemapUploadKey {
            source_revision: 31,
            source_hash: [1; 4],
            pmrem_hash: [2; 4],
            irradiance_cube_hash: [3; 4],
        },
        last_scheduled_upload_bytes: 8_192,
        gpu_staging_capacity_bytes: 65_536,
        ..EnvironmentCubemapUploadReport::default()
    }
}

fn capture_residency_report() -> EnvironmentCaptureResidencyReport {
    EnvironmentCaptureResidencyReport {
        observation_epoch: 31,
        last_published_handle: RenderEnvironmentCaptureHandle::new(33),
        last_published_output_generation: Some(12),
        resident_count: 3,
        resident_gpu_bytes: 12_582_912,
        eviction_count: 2,
    }
}

#[test]
fn no_current_frame_does_not_publish_the_default_profile() {
    let profile = Arc::new(RenderFrameProfile::default());

    let snapshot = EnvironmentRuntimeSnapshot::try_from_current_reports(
        None,
        &profile,
        RenderSceneSubmissionCompletionReport::default(),
        RenderReflectionProbeWorkloadReport::default(),
        realtime_ibl_status(),
        hydration_report(),
        capture_report(),
        capture_residency_report(),
        cubemap_upload_report(),
    )
    .expect("an empty framework state should form an explicit empty-frame snapshot");

    assert_eq!(snapshot.frame_generation, None);
    assert_eq!(snapshot.frame_profile, None);
}

#[test]
fn current_profile_shares_storage_and_delayed_completion_keeps_its_identity() {
    let profile = Arc::new(RenderFrameProfile {
        frame_generation: 42,
        passes: vec![RenderPassProfileEntry::default()],
        ..RenderFrameProfile::default()
    });
    let completion = RenderSceneSubmissionCompletionReport {
        status: RenderSceneSubmissionCompletionStatus::Completed,
        frame_generation: 39,
        ..RenderSceneSubmissionCompletionReport::default()
    };

    let snapshot = EnvironmentRuntimeSnapshot::try_from_current_reports(
        Some(42),
        &profile,
        completion,
        RenderReflectionProbeWorkloadReport {
            active_probe_count: 7,
            ..RenderReflectionProbeWorkloadReport::default()
        },
        realtime_ibl_status(),
        hydration_report(),
        capture_report(),
        capture_residency_report(),
        cubemap_upload_report(),
    )
    .expect("matching current reports should form a snapshot");

    let shared = snapshot
        .frame_profile
        .as_ref()
        .expect("a current frame must expose its matching profile");
    assert!(Arc::ptr_eq(&profile, shared));
    assert_eq!(shared.passes.as_ptr(), profile.passes.as_ptr());
    assert_eq!(snapshot.scene_submission.frame_generation, 39);
    assert_eq!(snapshot.reflection_probes.active_probe_count, 7);
    assert_eq!(snapshot.realtime_ibl.last_good_age_frame_count, Some(8));
    assert_eq!(snapshot.hydration.observation_epoch, 17);
    assert_eq!(snapshot.hydration.resident_count, 1);
    assert_eq!(snapshot.capture.observation_epoch, 23);
    assert_eq!(
        snapshot.capture.pending_handle,
        RenderEnvironmentCaptureHandle::new(4)
    );
    assert_eq!(
        snapshot.capture.active_phase,
        Some(RenderEnvironmentCapturePhase::Filtering)
    );
    assert_eq!(snapshot.capture_residency.resident_count, 3);
    assert_eq!(snapshot.capture_residency.observation_epoch, 31);
    assert_eq!(
        snapshot.capture_residency.last_published_handle,
        RenderEnvironmentCaptureHandle::new(33)
    );
    assert_eq!(
        snapshot.capture_residency.last_published_output_generation,
        Some(12)
    );
    assert_eq!(snapshot.capture_residency.resident_gpu_bytes, 12_582_912);
    assert_eq!(snapshot.capture_residency.eviction_count, 2);
    assert_eq!(snapshot.cubemap_upload.observation_epoch, 29);
    assert_eq!(snapshot.cubemap_upload.last_scheduled_upload_bytes, 8_192);
    assert_eq!(snapshot.cubemap_upload.gpu_staging_capacity_bytes, 65_536);
    assert_eq!(snapshot.cubemap_upload.resident_texture_bytes, 0);
}

#[test]
fn mismatched_current_profile_fails_closed() {
    let profile = Arc::new(RenderFrameProfile {
        frame_generation: 43,
        ..RenderFrameProfile::default()
    });

    assert_eq!(
        EnvironmentRuntimeSnapshot::try_from_current_reports(
            Some(42),
            &profile,
            RenderSceneSubmissionCompletionReport::default(),
            RenderReflectionProbeWorkloadReport::default(),
            realtime_ibl_status(),
            hydration_report(),
            capture_report(),
            capture_residency_report(),
            cubemap_upload_report(),
        ),
        Err(
            EnvironmentRuntimeSnapshotError::FrameProfileGenerationMismatch {
                frame_generation: 42,
                profile_generation: 43,
            }
        )
    );
}

#[test]
fn repeated_projection_reuses_the_same_profile_payload() {
    let profile = Arc::new(RenderFrameProfile {
        frame_generation: 42,
        passes: vec![RenderPassProfileEntry::default(); 64],
        ..RenderFrameProfile::default()
    });

    for _ in 0..16_384 {
        let snapshot = EnvironmentRuntimeSnapshot::try_from_current_reports(
            Some(42),
            &profile,
            RenderSceneSubmissionCompletionReport::default(),
            RenderReflectionProbeWorkloadReport::default(),
            realtime_ibl_status(),
            hydration_report(),
            capture_report(),
            capture_residency_report(),
            cubemap_upload_report(),
        )
        .expect("repeated projection should remain current");
        assert!(Arc::ptr_eq(
            &profile,
            snapshot.frame_profile.as_ref().expect("matching profile")
        ));
    }
}
