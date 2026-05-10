use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_FLOATING_WINDOW_SOURCE_DOCUMENT_ID;
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

pub(super) fn rebuild_builtin_floating_window_source_surface(
    surface: &mut UiSurface,
    shell_size: UiSize,
) -> Result<(), BuiltinFloatingWindowSourceTemplateBridgeError> {
    for root_id in surface.tree.roots.clone() {
        if let Some(root) = surface.tree.nodes.get_mut(&root_id) {
            root.dirty.layout = true;
            root.dirty.hit_test = true;
            root.dirty.render = true;
        }
    }
    surface.rebuild_dirty(shell_size)?;
    Ok(())
}
