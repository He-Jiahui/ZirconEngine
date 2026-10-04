use crate::core::framework::render::{CapturedFrame, RenderFrameProfile, RenderGpuTimingStatus};

use super::attach_profile_to_matching_capture;

#[test]
fn capture_profile_is_attached_only_to_its_matching_generation() {
    let mut capture = CapturedFrame::new(1, 1, vec![0; 4], 7);

    assert!(!attach_profile_to_matching_capture(
        &mut capture,
        &RenderFrameProfile {
            frame_generation: 6,
            ..RenderFrameProfile::default()
        }
    ));
    assert!(capture.frame_profile_json.is_none());
    assert!(attach_profile_to_matching_capture(
        &mut capture,
        &RenderFrameProfile {
            frame_generation: 7,
            ..RenderFrameProfile::default()
        }
    ));
    let profile: RenderFrameProfile = serde_json::from_str(
        capture
            .frame_profile_json
            .as_deref()
            .expect("matching capture contains profile JSON"),
    )
    .expect("capture profile JSON remains decodable");
    assert_eq!(profile.frame_generation, 7);
}

#[test]
fn matching_capture_profile_can_be_backfilled_with_late_gpu_timing() {
    let mut capture = CapturedFrame::new(1, 1, vec![0; 4], 7);
    assert!(attach_profile_to_matching_capture(
        &mut capture,
        &RenderFrameProfile {
            frame_generation: 7,
            ..RenderFrameProfile::default()
        }
    ));
    assert!(attach_profile_to_matching_capture(
        &mut capture,
        &RenderFrameProfile {
            frame_generation: 7,
            gpu_frame_time_us: Some(42),
            gpu_timing_status: RenderGpuTimingStatus::Measured,
            profile_latency_frames: 3,
            ..RenderFrameProfile::default()
        }
    ));

    let profile: RenderFrameProfile = serde_json::from_str(
        capture
            .frame_profile_json
            .as_deref()
            .expect("matching capture contains profile JSON"),
    )
    .expect("backfilled capture profile remains decodable");
    assert_eq!(profile.gpu_frame_time_us, Some(42));
    assert_eq!(profile.gpu_timing_status, RenderGpuTimingStatus::Measured);
    assert_eq!(profile.profile_latency_frames, 3);
}
