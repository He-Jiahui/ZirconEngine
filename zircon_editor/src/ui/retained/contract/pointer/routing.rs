#[path = "routing/chrome.rs"]
mod chrome;
#[path = "routing/chrome_route.rs"]
mod chrome_route;
#[path = "routing/geometry.rs"]
mod geometry;
#[path = "routing/pane_route.rs"]
mod pane_route;
#[path = "routing/panes.rs"]
mod panes;
#[path = "routing/workbench.rs"]
mod workbench;

pub(in crate::ui::retained_host::host_contract) use chrome::route_top_level_chrome;
pub(in crate::ui::retained_host::host_contract) use chrome_route::ChromePointerRoute;
pub(in crate::ui::retained_host::host_contract) use geometry::{
    contains, floating_window_content_frame, side_dock_content_frame, translated,
};
pub(in crate::ui::retained_host::host_contract) use pane_route::{
    PaneAssetReferenceList, PaneAssetSurface, PanePointerRoute, PanePointerTarget,
};
pub(in crate::ui::retained_host::host_contract) use panes::{
    route_pointer_move_to_pane, route_pointer_scroll_to_pane, route_pointer_to_pane,
};
pub(in crate::ui::retained_host::host_contract) use workbench::{
    route_pointer_move_to_workbench_generation, route_pointer_to_workbench_generation,
};
