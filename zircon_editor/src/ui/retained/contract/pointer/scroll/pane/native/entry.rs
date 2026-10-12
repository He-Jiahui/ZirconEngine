use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;

use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};
use super::browser::dispatch_browser_asset_details_scroll;
use super::hierarchy::dispatch_hierarchy_scroll;
use super::panels::{dispatch_console_scroll, dispatch_inspector_scroll};
use super::welcome::dispatch_welcome_scroll;

pub(in super::super) fn dispatch_native_pane_scroll(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    delta: f32,
) -> bool {
    match &pointer.target {
        PanePointerTarget::Hierarchy => dispatch_hierarchy_scroll(pane_host, pointer, delta),
        PanePointerTarget::Welcome => dispatch_welcome_scroll(pane_host, pointer, delta),
        PanePointerTarget::Console => dispatch_console_scroll(pane_host, pointer, delta),
        PanePointerTarget::Inspector => dispatch_inspector_scroll(pane_host, pointer, delta),
        PanePointerTarget::BrowserAssetDetails => {
            dispatch_browser_asset_details_scroll(pane_host, pointer, delta)
        }
        _ => false,
    }
}
