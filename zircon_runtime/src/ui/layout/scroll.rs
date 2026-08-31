use zircon_runtime_interface::ui::layout::{
    UiScrollState, UiScrollableBoxConfig, UiVirtualListWindow,
};

use super::{compute_virtual_list_window, fixed_extent_virtual_list_step_extent};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollVirtualizationPlan {
    pub scroll_state: UiScrollState,
    pub virtual_window: Option<UiVirtualListWindow>,
    pub visible_range_changed: bool,
}

pub fn virtual_window_for_scrollable_box(
    config: UiScrollableBoxConfig,
    offset: f32,
    child_count: usize,
    viewport_extent: f32,
) -> Option<UiVirtualListWindow> {
    let virtualization = config.virtualization?;
    let step_extent = fixed_extent_virtual_list_step_extent(virtualization.item_extent, config.gap);
    Some(compute_virtual_list_window(
        offset,
        viewport_extent,
        step_extent,
        child_count,
        virtualization.overscan,
    ))
}

pub(crate) fn plan_scrollable_virtual_window(
    config: UiScrollableBoxConfig,
    previous_state: UiScrollState,
    previous_window: Option<UiVirtualListWindow>,
    requested_offset: f32,
    child_count: usize,
    viewport_extent: f32,
    content_extent: f32,
) -> UiScrollVirtualizationPlan {
    let viewport_extent = viewport_extent.max(0.0);
    let content_extent = content_extent.max(0.0);
    let max_offset = (content_extent - viewport_extent).max(0.0);
    let offset = requested_offset.max(0.0).min(max_offset);
    let scroll_state = UiScrollState {
        offset,
        viewport_extent,
        content_extent,
    };
    let virtualization_enabled = config.virtualization.is_some();
    let virtual_window =
        virtual_window_for_scrollable_box(config, offset, child_count, viewport_extent).or(Some(
            UiVirtualListWindow {
                first_visible: 0,
                last_visible_exclusive: child_count,
            },
        ));

    UiScrollVirtualizationPlan {
        scroll_state,
        virtual_window,
        visible_range_changed: virtualization_enabled
            && (previous_window != virtual_window
                || (previous_state.viewport_extent - viewport_extent).abs() > f32::EPSILON
                || (previous_state.content_extent - content_extent).abs() > f32::EPSILON),
    }
}
