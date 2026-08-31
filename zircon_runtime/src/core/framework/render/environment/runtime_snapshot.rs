use std::sync::Arc;

use thiserror::Error;

use super::realtime_ibl_status::RealtimeIblStatusReport;
use crate::core::framework::render::{
    RenderFrameProfile, RenderReflectionProbeWorkloadReport, RenderSceneSubmissionCompletionReport,
};

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
pub enum EnvironmentRuntimeSnapshotError {
    #[error(
        "environment runtime snapshot frame generation {frame_generation} does not match profile generation {profile_generation}"
    )]
    FrameProfileGenerationMismatch {
        frame_generation: u64,
        profile_generation: u64,
    },
}

/// Coherent current environment state projected while the render framework owns its state lock.
///
/// Asynchronous reports retain their own source identities. The current-frame profile is shared by
/// `Arc` so querying this snapshot does not deep-clone pass or subsystem vectors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentRuntimeSnapshot {
    pub frame_generation: Option<u64>,
    pub frame_profile: Option<Arc<RenderFrameProfile>>,
    pub scene_submission: RenderSceneSubmissionCompletionReport,
    pub reflection_probes: RenderReflectionProbeWorkloadReport,
    pub realtime_ibl: RealtimeIblStatusReport,
}

impl EnvironmentRuntimeSnapshot {
    pub fn try_from_current_reports(
        frame_generation: Option<u64>,
        frame_profile: &Arc<RenderFrameProfile>,
        scene_submission: RenderSceneSubmissionCompletionReport,
        reflection_probes: RenderReflectionProbeWorkloadReport,
        realtime_ibl: RealtimeIblStatusReport,
    ) -> Result<Self, EnvironmentRuntimeSnapshotError> {
        let frame_profile = match frame_generation {
            None => None,
            Some(frame_generation) if frame_profile.frame_generation == frame_generation => {
                Some(Arc::clone(frame_profile))
            }
            Some(frame_generation) => {
                return Err(
                    EnvironmentRuntimeSnapshotError::FrameProfileGenerationMismatch {
                        frame_generation,
                        profile_generation: frame_profile.frame_generation,
                    },
                );
            }
        };

        Ok(Self {
            frame_generation,
            frame_profile,
            scene_submission,
            reflection_probes,
            realtime_ibl,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::core::framework::render::{
        IblBakeKey, RealtimeIblReadiness, RenderPassProfileEntry,
        RenderSceneSubmissionCompletionStatus,
    };

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

    #[test]
    fn no_current_frame_does_not_publish_the_default_profile() {
        let profile = Arc::new(RenderFrameProfile::default());

        let snapshot = EnvironmentRuntimeSnapshot::try_from_current_reports(
            None,
            &profile,
            RenderSceneSubmissionCompletionReport::default(),
            RenderReflectionProbeWorkloadReport::default(),
            realtime_ibl_status(),
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
            )
            .expect("repeated projection should remain current");
            assert!(Arc::ptr_eq(
                &profile,
                snapshot.frame_profile.as_ref().expect("matching profile")
            ));
        }
    }
}
