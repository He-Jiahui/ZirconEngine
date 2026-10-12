use super::*;
use crate::ui::retained_host::PaneSurfaceHostContext;

#[path = "assets/content.rs"]
mod content;
#[path = "../../pane_surface/assets/controls.rs"]
mod controls;
#[path = "assets/details.rs"]
mod details;
#[path = "../../pane_surface/assets/mesh_import.rs"]
mod mesh_import;
#[path = "../../pane_surface/assets/references.rs"]
mod references;
#[path = "assets/tree.rs"]
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
