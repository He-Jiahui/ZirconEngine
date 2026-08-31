use crate::core::framework::render::{
    RenderFrameworkError, RenderViewportPickDisposition, RenderViewportPickRequest,
    RenderViewportPickResult, RenderViewportPickTicket,
};
use std::sync::TryLockError;

use super::render_framework_backend_error::render_framework_backend_error;
use super::wgpu_render_framework::WgpuRenderFramework;
use crate::graphics::scene::{
    SceneHitProxyCompletion, SceneHitProxyProduct, SceneHitProxySubmission,
};

pub(in crate::graphics::runtime::render_framework) fn request_viewport_pick(
    framework: &WgpuRenderFramework,
    request: RenderViewportPickRequest,
) -> Result<RenderViewportPickTicket, RenderFrameworkError> {
    if !request.is_valid() {
        return Err(RenderFrameworkError::InvalidViewportPickRequest);
    }
    framework.finish_submission()?;
    let _operation_guard = framework.lock_operation();
    let mut state = framework.lock_state();
    state
        .renderer
        .poll_readback_completions()
        .map_err(render_framework_backend_error)?;
    let requires_hit_proxies = state
        .viewports
        .get(&request.viewport)
        .ok_or(RenderFrameworkError::UnknownViewport {
            viewport: request.viewport.raw(),
        })?
        .requires_hit_proxies();
    if !requires_hit_proxies {
        return state.viewport_picks.request_terminal(
            request,
            0,
            RenderViewportPickDisposition::Unavailable,
        );
    }
    let Some(frame) = state
        .viewport_pick_frames
        .resolve(request.viewport, request.frame_generation)
    else {
        return state.viewport_picks.request_terminal(
            request,
            0,
            RenderViewportPickDisposition::StaleFrame,
        );
    };
    let world_generation = frame.world_generation();
    let (ticket, completion_sender) = state
        .viewport_picks
        .request_pending(request, world_generation)?;
    let completion_frame = std::sync::Arc::clone(&frame);
    let completion = SceneHitProxyCompletion::new(Box::new(move |result| {
        match result {
            Ok(product) => complete_pick_product(completion_sender, &completion_frame, product),
            Err(_) => completion_sender.complete_terminal(RenderViewportPickDisposition::Rejected),
        };
    }));
    let render_frame = frame.render_frame();
    match state.renderer.submit_hit_proxy_product(
        &render_frame,
        request.pixel,
        request.policy,
        frame.virtual_geometry_enabled(),
        frame.as_ref(),
        completion.clone(),
    ) {
        Ok(SceneHitProxySubmission::Submitted) => {}
        Ok(SceneHitProxySubmission::OutsideRenderRegion) => {
            completion.complete(Ok(SceneHitProxyProduct {
                token: 0,
                depth: 0.0,
                world_position: [0.0; 3],
                world_normal: [0.0; 3],
            }));
        }
        Err(error) => {
            completion.complete(Err(error.to_string()));
        }
    }
    Ok(ticket)
}

fn complete_pick_product(
    completion: super::render_framework_state::ViewportPickCompletionSender,
    frame: &super::render_framework_state::ViewportPickFrameSnapshot,
    product: SceneHitProxyProduct,
) -> bool {
    if product.token == 0 {
        return completion.complete_terminal(RenderViewportPickDisposition::NoHit);
    }
    let Some(identity) = frame.resolve_hit_proxy_token(product.token) else {
        return completion.complete_terminal(RenderViewportPickDisposition::Rejected);
    };
    completion.complete_hit(
        identity.entity,
        identity.instance,
        identity.subobject,
        product.depth,
        product.world_position,
        product.world_normal,
    )
}

pub(in crate::graphics::runtime::render_framework) fn poll_viewport_pick(
    framework: &WgpuRenderFramework,
    ticket: RenderViewportPickTicket,
) -> Result<Option<RenderViewportPickResult>, RenderFrameworkError> {
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
    state.viewport_picks.poll(ticket)
}

pub(in crate::graphics::runtime::render_framework) fn cancel_viewport_pick(
    framework: &WgpuRenderFramework,
    ticket: RenderViewportPickTicket,
) -> Result<(), RenderFrameworkError> {
    framework.lock_state().viewport_picks.cancel(ticket)
}

#[cfg(test)]
mod tests {
    #[test]
    fn viewport_pick_poll_is_non_blocking_and_pumps_the_backend_timeline() {
        let source = include_str!("viewport_pick.rs");
        let poll = source
            .split("fn poll_viewport_pick(")
            .nth(1)
            .and_then(|source| source.split("fn cancel_viewport_pick(").next())
            .expect("viewport pick poll function");

        assert!(poll.contains("try_lock()"));
        assert!(poll.contains("TryLockError::WouldBlock"));
        assert!(poll.contains("poll_readback_completions()"));
        assert!(!poll.contains("finish_submission()"));
        assert!(!poll.contains("wait_for_readback_completions()"));
    }
}
