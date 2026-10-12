mod docks;
mod menus;
mod native_panes;
mod root_frames;
mod scene_layers;
mod style;
mod text;
mod welcome;

#[cfg(test)]
pub(crate) use native_panes::paint_scrollbar_component_for_test;
pub(in crate::ui::retained_host::host_contract) use native_panes::{
    hierarchy_row_frame, hierarchy_viewport_frame,
};
pub(in crate::ui::retained_host::host_contract) use scene_layers::draw_componentized_workbench_window;
#[cfg(test)]
pub(crate) use scene_layers::paint_componentized_extension_workspace_for_test;
pub(in crate::ui::retained_host::host_contract) use style::{
    ACCENT, CENTER_BAND, DOCUMENT_PANEL, FLOATING_PANEL, FLOATING_SHADOW, MUTED_TEXT, PANE_EMPTY,
    SEPARATOR, SIDE_PANEL, STATUS_BAR, TOOLBAR, TOP_BAR, VIEWPORT_PANEL,
};
pub(in crate::ui::retained_host::host_contract) use text::first_non_empty;
