use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiProjection,
};

use super::error::BuiltinHostWindowTemplateBridgeError;

pub(super) fn build_builtin_host_window_projection(
    runtime: &EditorUiHostRuntime,
    projection: &RetainedUiProjection,
    shell_size: UiSize,
) -> Result<RetainedUiHostProjection, BuiltinHostWindowTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    Ok(runtime.build_retained_host_projection_with_surface(projection, &surface)?)
}
