mod docks;
mod legacy;
mod menus;
mod native_panes;
mod root_frames;
mod scene_layers;
mod skeleton;
mod style;
mod text;
mod welcome;

pub(in crate::ui::retained_host::host_contract) use legacy::{
    draw_legacy_workbench_window, draw_legacy_workbench_window_profiled,
};
pub(in crate::ui::retained_host::host_contract) use scene_layers::{
    draw_componentized_workbench_window, draws_componentized_workbench_window,
};
pub(in crate::ui::retained_host::host_contract) use style::{
    ACCENT, CENTER_BAND, DOCUMENT_PANEL, FLOATING_PANEL, FLOATING_SHADOW, MUTED_TEXT, PANE_EMPTY,
    SEPARATOR, SIDE_PANEL, STATUS_BAR, TOOLBAR, TOP_BAR, VIEWPORT_PANEL,
};
pub(in crate::ui::retained_host::host_contract) use text::first_non_empty;
