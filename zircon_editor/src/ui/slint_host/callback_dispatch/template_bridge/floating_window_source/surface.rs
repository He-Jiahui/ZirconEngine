use zircon_runtime::ui::{layout::UiSize, surface::UiSurface};

use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID;
use crate::ui::template_runtime::EditorUiHostRuntime;

use super::error::BuiltinFloatingWindowSourceTemplateBridgeError;

pub(super) fn build_builtin_floating_window_source_surface(
    runtime: &EditorUiHostRuntime,
    shell_size: UiSize,
) -> Result<UiSurface, BuiltinFloatingWindowSourceTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    Ok(surface)
}
