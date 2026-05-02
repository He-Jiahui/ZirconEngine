use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID;
use crate::ui::template_runtime::{EditorUiHostRuntime, SlintUiHostProjection, SlintUiProjection};

use super::error::BuiltinViewportToolbarTemplateBridgeError;

pub(super) fn build_builtin_viewport_toolbar_host_projection(
    runtime: &EditorUiHostRuntime,
    projection: &SlintUiProjection,
    surface_size: UiSize,
) -> Result<SlintUiHostProjection, BuiltinViewportToolbarTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
    surface.compute_layout(surface_size)?;
    Ok(runtime.build_slint_host_projection_with_surface(projection, &surface)?)
}
