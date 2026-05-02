use zircon_runtime_interface::ui::layout::UiScrollableBoxConfig;
use zircon_runtime_interface::ui::layout::UiVirtualListWindow;

use super::compute_virtual_list_window;

pub fn virtual_window_for_scrollable_box(
    config: UiScrollableBoxConfig,
    offset: f32,
    child_count: usize,
    viewport_extent: f32,
) -> Option<UiVirtualListWindow> {
    let virtualization = config.virtualization?;
    let step_extent = (virtualization.item_extent + config.gap).max(virtualization.item_extent);
    Some(compute_virtual_list_window(
        offset,
        viewport_extent,
        step_extent,
        child_count,
        virtualization.overscan,
    ))
}
