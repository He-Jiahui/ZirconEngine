use super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;

mod content;
mod controls;
mod details;
mod mesh_import;
mod references;
mod tree;

pub(super) fn wire_asset_callbacks(
    pane_surface_host: &PaneSurfaceHostContext,
    ui: &UiHostWindow,
    host: &Rc<RefCell<RetainedEditorHost>>,
) {
    mesh_import::wire_mesh_import_callbacks(pane_surface_host, host);
    controls::wire_asset_control_callbacks(pane_surface_host, ui, host);
    tree::wire_asset_tree_callbacks(pane_surface_host, ui, host);
    content::wire_asset_content_callbacks(pane_surface_host, ui, host);
    references::wire_asset_reference_callbacks(pane_surface_host, ui, host);
    details::wire_asset_detail_callbacks(pane_surface_host, ui, host);
}
