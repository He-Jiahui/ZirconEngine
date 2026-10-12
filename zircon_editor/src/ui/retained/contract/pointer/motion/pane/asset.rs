use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;

use super::super::super::routing::{PaneAssetSurface, PanePointerRoute, PanePointerTarget};

pub(super) fn dispatch_asset_pane_move(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
) {
    match &pointer.target {
        PanePointerTarget::AssetTree(mode) => pane_host.invoke_asset_tree_pointer_moved(
            mode.as_str().into(),
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        ),
        PanePointerTarget::AssetContent(mode) => pane_host.invoke_asset_content_pointer_moved(
            mode.as_str().into(),
            pointer.local_x,
            pointer.local_y,
            pointer.width,
            pointer.height,
        ),
        PanePointerTarget::AssetReference(mode, list_kind) => {
            match mode {
                PaneAssetSurface::Activity => {
                    pane_host.set_activity_asset_reference_hover_frame(pointer.frame.clone());
                }
                PaneAssetSurface::Browser => {
                    pane_host.set_browser_asset_reference_hover_frame(pointer.frame.clone());
                }
            }
            pane_host.invoke_asset_reference_pointer_moved(
                mode.as_str().into(),
                list_kind.as_str().into(),
                pointer.local_x,
                pointer.local_y,
                pointer.width,
                pointer.height,
            );
        }
        _ => {}
    }
}
