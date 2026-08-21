mod constraints;
mod pass;
mod scroll;
mod style_mapping;
mod taffy_bridge;
mod virtualization;

/// Upper bound for asset-authored track counts, slot coordinates/spans, and overscan.
pub(crate) const MAX_UI_LAYOUT_DISCRETE_VALUE: usize = 4_096;

pub use constraints::solve_axis_constraints;
pub(crate) use pass::{
    compute_incremental_layout_tree_with_text_measure_cache,
    compute_layout_tree_with_text_measure_cache,
    compute_layout_tree_with_text_measure_cache_and_slot_index, UiLayoutSlotIndex,
};
pub use pass::{
    compute_layout_tree, ui_layout_pass_stage_names, UiLayoutPassStage, UI_LAYOUT_PASS_ORDER,
};
pub(crate) use scroll::plan_scrollable_virtual_window;
pub use scroll::virtual_window_for_scrollable_box;
pub use style_mapping::{
    taffy_style_from_ui_layout_style, ui_layout_style_from_axis_constraints,
    ui_layout_style_from_container,
};
pub use taffy_bridge::{taffy_display_for_family, taffy_style_for_container};
pub use virtualization::compute_virtual_list_window;
