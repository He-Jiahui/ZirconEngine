use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::super::super::super::routing::PanePointerRoute;

pub(super) fn dispatch_hierarchy_scroll(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    delta: f32,
) -> bool {
    record_current_ui_perf_counter(UiPerfCounter::HierarchyScrollDispatchCount, 1.0);
    pane_host.invoke_hierarchy_pointer_scrolled(
        pointer.local_x,
        pointer.local_y,
        delta,
        pointer.width,
        pointer.height,
    );
    true
}
