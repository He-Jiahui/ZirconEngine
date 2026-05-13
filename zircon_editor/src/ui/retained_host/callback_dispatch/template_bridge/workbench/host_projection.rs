use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiProjection,
};

use super::error::BuiltinHostWindowTemplateBridgeError;

pub(super) fn build_builtin_host_window_surface(
    runtime: &EditorUiHostRuntime,
    shell_size: UiSize,
) -> Result<UiSurface, BuiltinHostWindowTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    Ok(surface)
}

pub(super) fn rebuild_builtin_host_window_surface(
    surface: &mut UiSurface,
    shell_size: UiSize,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
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

pub(super) fn project_builtin_host_window_projection(
    runtime: &EditorUiHostRuntime,
    projection: &RetainedUiProjection,
    surface: &UiSurface,
) -> Result<RetainedUiHostProjection, BuiltinHostWindowTemplateBridgeError> {
    Ok(runtime.build_retained_host_projection_with_surface(projection, surface)?)
}
