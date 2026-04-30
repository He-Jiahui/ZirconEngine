use zircon_runtime::ui::layout::UiSize;

use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID;
use crate::ui::template_runtime::{EditorUiHostRuntime, SlintUiHostProjection, SlintUiProjection};

use super::error::BuiltinHostWindowTemplateBridgeError;

pub(super) fn build_builtin_host_window_projection(
    runtime: &EditorUiHostRuntime,
    projection: &SlintUiProjection,
    shell_size: UiSize,
) -> Result<SlintUiHostProjection, BuiltinHostWindowTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    Ok(runtime.build_slint_host_projection_with_surface(projection, &surface)?)
}
