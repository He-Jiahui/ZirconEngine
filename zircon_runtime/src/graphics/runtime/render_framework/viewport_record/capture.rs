use std::sync::Arc;

use crate::core::framework::render::{CapturedFrame, RenderFrameProfile};

use crate::graphics::{CompiledRenderPipeline, ViewportFrame};

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn last_capture(
        &self,
    ) -> Option<&CapturedFrame> {
        self.last_capture.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn store_presented_pipeline(
        &mut self,
        compiled_pipeline: Arc<CompiledRenderPipeline>,
    ) {
        self.compiled_pipeline = Some(compiled_pipeline);
    }

    pub(in crate::graphics::runtime::render_framework) fn capture_for_inspection(
        &mut self,
    ) -> Option<CapturedFrame> {
        if self
            .last_capture
            .as_ref()
            .is_some_and(|capture| capture.graph_dump.is_none())
        {
            let graph_dump = self
                .last_capture_pipeline
                .as_ref()
                .map(|pipeline| pipeline.graph_dump_text());
            if let Some(capture) = self.last_capture.as_mut() {
                capture.graph_dump = graph_dump;
            }
        }
        self.last_capture.clone()
    }

    pub(in crate::graphics::runtime::render_framework) fn store_synchronous_capture(
        &mut self,
        frame: ViewportFrame,
    ) {
        if self
            .last_promoted_capture_generation
            .is_some_and(|generation| frame.generation < generation)
        {
            return;
        }
        self.last_capture_pipeline = self.compiled_pipeline.clone();
        self.last_promoted_capture_generation = Some(frame.generation);
        self.last_capture = Some(CapturedFrame::with_capture_report(
            frame.width,
            frame.height,
            frame.rgba,
            frame.generation,
            frame.capture_report,
        ));
    }

    pub(in crate::graphics::runtime::render_framework) fn attach_capture_frame_profile(
        &mut self,
        profile: &RenderFrameProfile,
    ) -> bool {
        if self
            .last_capture
            .as_mut()
            .is_some_and(|capture| attach_profile_to_matching_capture(capture, profile))
        {
            return true;
        }
        let capture_is_pending = self
            .capture_mailbox
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .has_pending(profile.frame_generation);
        if !capture_is_pending {
            return false;
        }
        self.pending_capture_profiles
            .entry(profile.frame_generation)
            .or_default()
            .push(profile.clone());
        while self.pending_capture_profiles.len()
            > super::capture_mailbox::viewport_capture_pending_limit()
        {
            let Some(generation) = self.pending_capture_profiles.keys().next().copied() else {
                break;
            };
            self.pending_capture_profiles.remove(&generation);
        }
        true
    }
}

pub(super) fn capture_generation_is_newer(previous: Option<u64>, candidate: u64) -> bool {
    previous.map_or(true, |generation| candidate > generation)
}

pub(super) fn attach_profile_to_matching_capture(
    capture: &mut CapturedFrame,
    profile: &RenderFrameProfile,
) -> bool {
    if capture.generation != profile.frame_generation {
        return false;
    }
    let Ok(profile_json) = serde_json::to_string(profile) else {
        return false;
    };
    capture.frame_profile_json = Some(profile_json);
    true
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CapturedFrame, RenderFrameProfile, RenderGpuTimingStatus,
    };

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
}
