use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::super::super::super::routing::{PanePointerRoute, PanePointerTarget};

pub(super) fn dispatch_asset_content_scroll(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    delta: f32,
) -> bool {
    let PanePointerTarget::AssetContent(mode) = &pointer.target else {
        return false;
    };

    if mode.as_str() == "browser" {
        record_current_ui_perf_counter(UiPerfCounter::AssetBrowserScrollDispatchCount, 1.0);
    }
    pane_host.invoke_asset_content_pointer_scrolled(
        mode.as_str().into(),
        pointer.local_x,
        pointer.local_y,
        delta,
        pointer.width,
        pointer.height,
    );
    true
}
