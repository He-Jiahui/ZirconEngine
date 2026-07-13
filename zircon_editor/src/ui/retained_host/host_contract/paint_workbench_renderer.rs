mod docks;
mod host_window;
mod menus;
mod native_panes;
mod root_frames;
mod scene_layers;
mod skeleton;
mod style;
mod text;
mod welcome;

pub(in crate::ui::retained_host::host_contract) use host_window::{
    draw_host_workbench_window, draw_host_workbench_window_profiled,
};
#[cfg(test)]
pub(crate) use native_panes::paint_scrollbar_component_for_test;
#[cfg(test)]
pub(crate) use scene_layers::paint_componentized_extension_workspace_for_test;
pub(in crate::ui::retained_host::host_contract) use scene_layers::{
    draw_componentized_workbench_window, draws_componentized_workbench_window,
};
pub(in crate::ui::retained_host::host_contract) use style::{
    ACCENT, CENTER_BAND, DOCUMENT_PANEL, FLOATING_PANEL, FLOATING_SHADOW, MUTED_TEXT, PANE_EMPTY,
    SEPARATOR, SIDE_PANEL, STATUS_BAR, TOOLBAR, TOP_BAR, VIEWPORT_PANEL,
};
pub(in crate::ui::retained_host::host_contract) use text::first_non_empty;
