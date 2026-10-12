#[path = "callbacks/host.rs"]
mod host;
#[path = "callbacks/pane.rs"]
mod pane;
#[path = "callbacks/types.rs"]
mod types;

pub(in crate::ui::retained_host::host_contract) use host::UiHostCallbacks;
pub(in crate::ui::retained_host::host_contract) use pane::PaneSurfaceCallbacks;
