use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;

use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};

pub(super) fn dispatch_asset_reference_scroll(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    delta: f32,
) -> bool {
    let PanePointerTarget::AssetReference(mode, list_kind) = &pointer.target else {
        return false;
    };

    pane_host.invoke_asset_reference_pointer_scrolled(
        mode.as_str().into(),
        list_kind.as_str().into(),
        pointer.local_x,
        pointer.local_y,
        delta,
        pointer.width,
        pointer.height,
    );
    true
}
