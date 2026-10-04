//! 捕获 API 从最近完成的视口产品取图像；异步读取和非阻塞查询不得返回仍在写入的帧。
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
    state
        .renderer
        .poll_readback_completions()
        .map_err(render_framework_backend_error)?;
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
#[path = "tests/capture_frame.rs"]
mod tests;
