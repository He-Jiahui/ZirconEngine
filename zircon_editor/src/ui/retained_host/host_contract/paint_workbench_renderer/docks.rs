mod bottom;
mod document;
mod floating_windows;
mod pane;
mod panel_header;
mod rail;
mod side;
mod viewport_toolbar;

pub(in crate::ui::retained_host::host_contract) use self::bottom::draw_bottom_dock;
pub(in crate::ui::retained_host::host_contract) use self::document::draw_document_dock;
pub(in crate::ui::retained_host::host_contract) use self::floating_windows::draw_floating_layer;
pub(in crate::ui::retained_host::host_contract) use self::side::draw_side_dock;
