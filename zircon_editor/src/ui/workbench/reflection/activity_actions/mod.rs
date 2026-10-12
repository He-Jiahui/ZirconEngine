#[path = "asset_actions.rs"]
mod asset_actions;
#[path = "common_actions.rs"]
mod common_actions;
#[path = "inspector_actions.rs"]
mod inspector_actions;
#[path = "resolve.rs"]
mod resolve;
#[path = "viewport_actions.rs"]
mod viewport_actions;

pub(crate) use resolve::activity_actions_for_tab;
