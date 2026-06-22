use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;

use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};

pub(super) fn dispatch_asset_tree_scroll(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    delta: f32,
) -> bool {
    let PanePointerTarget::AssetTree(mode) = &pointer.target else {
        return false;
    };

    pane_host.invoke_asset_tree_pointer_scrolled(
        mode.clone(),
        pointer.local_x,
        pointer.local_y,
        delta,
        pointer.width,
        pointer.height,
    );
    true
}
