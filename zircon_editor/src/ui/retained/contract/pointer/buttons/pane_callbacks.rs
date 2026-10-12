#[path = "../../../../contract/pointer/buttons/pane_callbacks/asset_panes.rs"]
mod asset_panes;
#[path = "../../../../contract/pointer/buttons/pane_callbacks/fallback.rs"]
mod fallback;
#[path = "pane_callbacks/kind.rs"]
mod kind;
#[path = "../../../../contract/pointer/buttons/pane_callbacks/native_panes.rs"]
mod native_panes;
#[path = "pane_callbacks/target.rs"]
mod target;
#[path = "pane_callbacks/nodes.rs"]
mod template_nodes;
#[path = "../../../../contract/pointer/buttons/pane_callbacks/viewport.rs"]
mod viewport;

pub(in crate::ui::retained_host::host_contract) use self::target::dispatch_pane_button;
