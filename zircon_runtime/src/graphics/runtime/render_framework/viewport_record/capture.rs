use std::sync::Arc;

use crate::core::framework::render::{CapturedFrame, RenderFrameProfile};

use crate::graphics::CompiledRenderPipeline;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn last_capture(
        &self,
    ) -> Option<&CapturedFrame> {
        self.last_capture.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn store_capture(
        &mut self,
        compiled_pipeline: Arc<CompiledRenderPipeline>,
        capture: CapturedFrame,
    ) {
        self.last_capture_pipeline = Some(Arc::clone(&compiled_pipeline));
        self.compiled_pipeline = Some(compiled_pipeline);
        self.last_capture = Some(capture);
    }

    pub(in crate::graphics::runtime::render_framework) fn store_presented_pipeline(
        &mut self,
        compiled_pipeline: Arc<CompiledRenderPipeline>,
    ) {
        self.compiled_pipeline = Some(compiled_pipeline);
    }

    pub(in crate::graphics::runtime::render_framework) fn attach_capture_frame_profile(
        &mut self,
        profile: &RenderFrameProfile,
    ) -> bool {
        self.last_capture
            .as_mut()
            .is_some_and(|capture| attach_profile_to_matching_capture(capture, profile))
    }

    pub(in crate::graphics::runtime::render_framework) fn capture_graph_dump(
        &self,
        compiled_pipeline: &Arc<CompiledRenderPipeline>,
    ) -> String {
        if self
            .last_capture_pipeline
            .as_ref()
            .is_some_and(|previous| Arc::ptr_eq(previous, compiled_pipeline))
        {
            if let Some(graph_dump) = self
                .last_capture
                .as_ref()
                .and_then(|capture| capture.graph_dump.as_ref())
            {
                return graph_dump.clone();
            }
        }

        compiled_pipeline.graph().dump().to_text()
    }
}

fn attach_profile_to_matching_capture(
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
    use crate::core::framework::render::{CapturedFrame, RenderFrameProfile};

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
        assert_eq!(profile.profile_latency_frames, 3);
    }
}
