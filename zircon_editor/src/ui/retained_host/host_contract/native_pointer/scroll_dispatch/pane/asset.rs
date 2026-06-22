mod content;
mod reference;
mod tree;

use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;

use self::content::dispatch_asset_content_scroll;
use self::reference::dispatch_asset_reference_scroll;
use self::tree::dispatch_asset_tree_scroll;
use super::super::super::routing::PanePointerRoute;

pub(super) fn dispatch_asset_pane_scroll(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    delta: f32,
) -> bool {
    dispatch_asset_tree_scroll(pane_host, pointer, delta)
        || dispatch_asset_content_scroll(pane_host, pointer, delta)
        || dispatch_asset_reference_scroll(pane_host, pointer, delta)
}
