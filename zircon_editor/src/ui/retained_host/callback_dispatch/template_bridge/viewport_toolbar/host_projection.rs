use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::layout::UiSize;

use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID;
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiProjection,
};

use super::error::BuiltinViewportToolbarTemplateBridgeError;

pub(super) fn build_builtin_viewport_toolbar_surface(
    runtime: &EditorUiHostRuntime,
    surface_size: UiSize,
) -> Result<UiSurface, BuiltinViewportToolbarTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
    surface.compute_layout(surface_size)?;
    Ok(surface)
}

pub(super) fn rebuild_builtin_viewport_toolbar_surface(
    surface: &mut UiSurface,
    surface_size: UiSize,
) -> Result<(), BuiltinViewportToolbarTemplateBridgeError> {
    for root_id in surface.tree.roots.clone() {
        if let Some(root) = surface.tree.nodes.get_mut(&root_id) {
            root.dirty.layout = true;
            root.dirty.hit_test = true;
            root.dirty.render = true;
        }
    }
    surface.rebuild_dirty(surface_size)?;
    Ok(())
}

pub(super) fn project_builtin_viewport_toolbar_host_projection(
    runtime: &EditorUiHostRuntime,
    projection: &RetainedUiProjection,
    surface: &UiSurface,
) -> Result<RetainedUiHostProjection, BuiltinViewportToolbarTemplateBridgeError> {
    Ok(runtime.build_retained_host_projection_with_surface(projection, surface)?)
}
