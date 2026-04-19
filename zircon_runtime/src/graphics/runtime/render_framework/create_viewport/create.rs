use crate::core::framework::render::{
    RenderFrameworkError, RenderViewportDescriptor, RenderViewportHandle,
};

use super::super::viewport_record::ViewportRecord;
use super::super::wgpu_render_framework::WgpuRenderFramework;

pub(in crate::graphics::runtime::render_framework) fn create_viewport(
    server: &WgpuRenderFramework,
    descriptor: RenderViewportDescriptor,
) -> Result<RenderViewportHandle, RenderFrameworkError> {
    let mut state = server.state.lock().unwrap();
    let handle = RenderViewportHandle::new(state.next_viewport_id);
    state.next_viewport_id += 1;
    state
        .viewports
        .insert(handle, ViewportRecord::new(descriptor));
    state.stats.active_viewports = state.viewports.len();
    Ok(handle)
}
