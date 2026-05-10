use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiProjection,
};

use super::error::BuiltinViewportToolbarTemplateBridgeError;

pub(super) fn build_builtin_viewport_toolbar_host_projection(
    runtime: &EditorUiHostRuntime,
    projection: &RetainedUiProjection,
    surface_size: UiSize,
) -> Result<RetainedUiHostProjection, BuiltinViewportToolbarTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
    surface.compute_layout(surface_size)?;
    Ok(runtime.build_retained_host_projection_with_surface(projection, &surface)?)
}
