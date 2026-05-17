use std::sync::MutexGuard;

use crate::core::framework::render::RenderViewportHandle;
use crate::graphics::backend::GraphicsDebuggerCaptureStop;

use super::super::render_framework_state::RenderFrameworkState;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn begin_graphics_debugger_capture(
    state: &mut RenderFrameworkState,
    viewport: RenderViewportHandle,
) -> bool {
    if !state.graphics_debugger.should_capture(viewport) {
        return false;
    }

    state.graphics_debugger.begin_capture();
    state.renderer.start_graphics_debugger_capture();
    true
}

pub(in crate::graphics::runtime::render_framework) fn prepare_graphics_debugger_capture_finish(
    state: &mut RenderFrameworkState,
    active_capture: bool,
) -> Option<GraphicsDebuggerCaptureStop> {
    if !active_capture {
        return None;
    }

    Some(state.renderer.prepare_graphics_debugger_capture_stop())
}

pub(in crate::graphics::runtime::render_framework) fn stop_prepared_graphics_debugger_capture(
    capture_stop: Option<GraphicsDebuggerCaptureStop>,
) -> Option<String> {
    capture_stop.and_then(|capture_stop| capture_stop.stop().err().map(|error| error.to_string()))
}

pub(in crate::graphics::runtime::render_framework) fn record_graphics_debugger_capture_finish(
    state: &mut RenderFrameworkState,
    frame_generation: Option<u64>,
    submit_error: Option<String>,
    stop_error: Option<String>,
) {
    let error = merge_capture_errors(submit_error, stop_error);
    state
        .graphics_debugger
        .finish_capture(frame_generation, error);
}

pub(in crate::graphics::runtime::render_framework) fn finish_active_capture_and_relock<'a>(
    framework: &'a WgpuRenderFramework,
    mut state: MutexGuard<'a, RenderFrameworkState>,
    active_capture: bool,
    frame_generation: Option<u64>,
    submit_error: Option<String>,
) -> MutexGuard<'a, RenderFrameworkState> {
    if !active_capture {
        return state;
    }

    let capture_stop = prepare_graphics_debugger_capture_finish(&mut state, active_capture);
    drop(state);
    // The operation lock remains held while state is unlocked, so no second frame
    // or viewport mutation can enter the active capture before wgpu stop/poll completes.
    let stop_error = stop_prepared_graphics_debugger_capture(capture_stop);
    let mut state = framework.lock_state();
    record_graphics_debugger_capture_finish(&mut state, frame_generation, submit_error, stop_error);
    state
}

pub(in crate::graphics::runtime::render_framework) fn fail_pending_graphics_debugger_capture(
    state: &mut RenderFrameworkState,
    viewport: RenderViewportHandle,
    error: String,
) {
    state
        .graphics_debugger
        .fail_pending_capture(viewport, error);
}

fn merge_capture_errors(
    submit_error: Option<String>,
    stop_error: Option<String>,
) -> Option<String> {
    match (submit_error, stop_error) {
        (Some(submit_error), Some(stop_error)) => Some(format!(
            "{submit_error}; graphics debugger stop failed: {stop_error}"
        )),
        (Some(submit_error), None) => Some(submit_error),
        (None, Some(stop_error)) => Some(stop_error),
        (None, None) => None,
    }
}
