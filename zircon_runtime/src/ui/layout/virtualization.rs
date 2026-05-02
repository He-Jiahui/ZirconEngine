use zircon_runtime_interface::ui::layout::UiVirtualListWindow;

pub fn compute_virtual_list_window(
    offset: f32,
    viewport_extent: f32,
    item_extent: f32,
    item_count: usize,
    overscan: usize,
) -> UiVirtualListWindow {
    if item_count == 0 || item_extent <= 0.0 || viewport_extent <= 0.0 {
        return UiVirtualListWindow::default();
    }

    let offset = offset.max(0.0);
    let first_visible = (offset / item_extent).floor() as usize;
    let last_visible_exclusive = ((offset + viewport_extent) / item_extent).ceil() as usize;

    UiVirtualListWindow {
        first_visible: first_visible.saturating_sub(overscan),
        last_visible_exclusive: last_visible_exclusive
            .saturating_add(overscan)
            .min(item_count),
    }
}
