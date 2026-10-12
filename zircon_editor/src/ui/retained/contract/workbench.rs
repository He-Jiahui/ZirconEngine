#[path = "workbench/docks.rs"]
mod docks;
#[path = "workbench/menus.rs"]
mod menus;
#[path = "workbench/native_panes.rs"]
mod native_panes;
#[path = "workbench/root_frames.rs"]
mod root_frames;
#[path = "workbench/scene_layers.rs"]
mod scene_layers;
#[path = "workbench/style.rs"]
mod style;
#[path = "workbench/text.rs"]
mod text;
#[path = "workbench/welcome.rs"]
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
