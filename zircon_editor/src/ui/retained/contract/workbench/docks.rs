#[path = "docks/bottom.rs"]
mod bottom;
#[path = "docks/document.rs"]
mod document;
#[path = "docks/floating_windows.rs"]
mod floating_windows;
#[path = "docks/palette.rs"]
mod palette;
#[path = "docks/pane.rs"]
mod pane;
#[path = "docks/panel_header.rs"]
mod panel_header;
#[path = "docks/rail.rs"]
mod rail;
#[path = "docks/side.rs"]
mod side;
#[path = "docks/viewport_toolbar.rs"]
mod viewport_toolbar;

pub(in crate::ui::retained_host::host_contract) use self::bottom::draw_bottom_dock;
pub(in crate::ui::retained_host::host_contract) use self::document::draw_document_dock;
pub(in crate::ui::retained_host::host_contract) use self::floating_windows::draw_floating_layer;
pub(in crate::ui::retained_host::host_contract) use self::side::draw_side_dock;
