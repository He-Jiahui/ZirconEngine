#[path = "assets/frame.rs"]
mod frame;
#[path = "assets/overlay.rs"]
mod overlay;

pub(in crate::ui::retained_host::host_contract) use overlay::draw_activity_asset_tree_hover_overlay;
