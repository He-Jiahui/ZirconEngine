use crate::core::framework::render::{CapturedFrame, RenderFrameworkError, RenderViewportHandle};

use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn capture_frame(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
    capture_frame_if_newer(framework, viewport, None)
}

pub(in crate::graphics::runtime::render_framework) fn capture_frame_if_newer(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    last_generation: Option<u64>,
) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "capture_frame");
    // Readback is a synchronization boundary: propagate an already-started
    // pipelined submission error before exposing its last completed image.
    framework.finish_submission()?;
    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
    let Some(frame) = state
        .viewports
        .get(&viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        })?
        .last_capture()
    else {
        return Ok(None);
    };
    if last_generation == Some(frame.generation) {
        return Ok(None);
    }
    let frame = frame.clone();
    state.stats.captured_frames += 1;
    Ok(Some(frame))
}

#[cfg(test)]
mod tests {
    #[test]
    fn capture_generation_is_checked_before_rgba_clone() {
        let source = include_str!("capture_frame.rs");
        let generation_check = source
            .find(concat!("if last_generation == Some(frame.", "generation)"))
            .expect("capture path should compare generation before cloning");
        let frame_clone = source
            .find(concat!("let frame = frame.", "clone();"))
            .expect("new capture should still return an owned frame");

        assert!(generation_check < frame_clone);
    }

    #[test]
    fn capture_finishes_pipelined_submission_before_reading_the_frame() {
        let source = include_str!("capture_frame.rs");
        let finish_submission = source
            .find("framework.finish_submission()?;")
            .expect("capture should collect a pending pipelined result");
        let operation_lock = source
            .find("let _operation_guard = framework.lock_operation();")
            .expect("capture should still serialize frame state access");

        assert!(finish_submission < operation_lock);
    }
}
