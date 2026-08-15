use std::sync::TryLockError;

use crate::core::framework::render::{
    CapturedFrame, CapturedHdrFrame, RenderFrameworkError, RenderViewportHandle,
};

use super::super::render_framework_backend_error::render_framework_backend_error;
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
    state
        .renderer
        .wait_for_readback_completions()
        .map_err(render_framework_backend_error)?;
    let completed_frame =
        {
            let record = state.viewports.get_mut(&viewport).ok_or(
                RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                },
            )?;
            record.promote_completed_async_capture();
            match last_generation {
                Some(generation)
                    if record
                        .last_capture()
                        .is_some_and(|capture| capture.generation > generation) =>
                {
                    record.capture_for_inspection()
                }
                _ => None,
            }
        };
    let frame =
        if let Some(frame) = completed_frame {
            frame
        } else {
            let frame = state
                .renderer
                .capture_latest_frame()
                .map_err(render_framework_backend_error)?;
            let Some(frame) = frame else {
                return Ok(None);
            };
            let record = state.viewports.get_mut(&viewport).ok_or(
                RenderFrameworkError::UnknownViewport {
                    viewport: viewport.raw(),
                },
            )?;
            record.store_synchronous_capture(frame);
            let Some(frame) = record.capture_for_inspection() else {
                return Ok(None);
            };
            frame
        };
    if last_generation.is_some_and(|generation| frame.generation <= generation) {
        return Ok(None);
    }
    state.stats.captured_frames += 1;
    Ok(Some(frame))
}

pub(in crate::graphics::runtime::render_framework) fn capture_scene_color_hdr(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
) -> Result<Option<CapturedHdrFrame>, RenderFrameworkError> {
    crate::profile_scope!("runtime", "render_framework", "capture_scene_color_hdr");
    framework.finish_submission()?;
    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
    if !state.viewports.contains_key(&viewport) {
        return Err(RenderFrameworkError::UnknownViewport {
            viewport: viewport.raw(),
        });
    }
    if state.last_retained_scene_color_viewport != Some(viewport) {
        return Ok(None);
    }
    state
        .renderer
        .wait_for_readback_completions()
        .map_err(render_framework_backend_error)?;
    let frame = state
        .renderer
        .capture_latest_scene_color_hdr()
        .map_err(render_framework_backend_error)?;
    if frame.is_some() {
        state.stats.captured_frames += 1;
    }
    Ok(frame)
}

pub(in crate::graphics::runtime::render_framework) fn poll_captured_frame_if_newer(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    last_generation: Option<u64>,
) -> Result<Option<CapturedFrame>, RenderFrameworkError> {
    let _operation_guard = match framework.core.operation_lock.try_lock() {
        Ok(guard) => guard,
        Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(TryLockError::WouldBlock) => return Ok(None),
    };
    let mut state = match framework.core.state.try_lock() {
        Ok(state) => state,
        Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
        Err(TryLockError::WouldBlock) => return Ok(None),
    };
    state.renderer.poll_readback_completions();
    let record =
        state
            .viewports
            .get_mut(&viewport)
            .ok_or(RenderFrameworkError::UnknownViewport {
                viewport: viewport.raw(),
            })?;
    record.promote_completed_async_capture();
    let Some(frame) = record.last_capture() else {
        return Ok(None);
    };
    if last_generation.is_some_and(|generation| frame.generation <= generation) {
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
        let source = include_str!("capture_frame.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();
        let capture_source = source
            .split("pub(in crate::graphics::runtime::render_framework) fn poll_captured_frame")
            .next()
            .unwrap_or_default();
        let generation_check = capture_source
            .find("capture.generation > generation")
            .expect("capture path should compare generation before cloning");
        let frame_clone = capture_source
            .find("capture_for_inspection")
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

    #[test]
    fn hdr_capture_finishes_submission_before_reading_retained_scene_color() {
        let source = include_str!("capture_frame.rs");
        let hdr_capture = source
            .split("fn capture_scene_color_hdr")
            .nth(1)
            .expect("HDR capture entry point should exist")
            .split("fn poll_captured_frame_if_newer")
            .next()
            .expect("HDR capture should end before the nonblocking polling path");
        let finish_submission = hdr_capture
            .find("framework.finish_submission()?;")
            .expect("HDR capture should collect a pending pipelined result");
        let retained_viewport = hdr_capture
            .find("last_retained_scene_color_viewport")
            .expect("HDR capture should reject a retained target from another viewport");
        let readback_wait = hdr_capture
            .find("wait_for_readback_completions")
            .expect("HDR capture should wait for completed GPU work");
        let retained_scene_color = hdr_capture
            .find("capture_latest_scene_color_hdr")
            .expect("HDR capture should read the renderer-owned scene color");

        assert!(finish_submission < readback_wait);
        assert!(retained_viewport < readback_wait);
        assert!(readback_wait < retained_scene_color);
    }

    #[test]
    fn async_viewport_poll_never_finishes_submission_or_waits_for_the_device() {
        let source = include_str!("capture_frame.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();
        let poll_start = source
            .find("fn poll_captured_frame_if_newer")
            .expect("async poll should exist");
        let poll_source = &source[poll_start..];

        assert!(!poll_source.contains("finish_submission"));
        assert!(!poll_source.contains("wait_indefinitely"));
        assert!(poll_source.contains("try_lock"));
        assert!(poll_source.contains("poll_readback_completions"));
    }
}
