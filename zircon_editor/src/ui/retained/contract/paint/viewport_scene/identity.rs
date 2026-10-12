#[path = "identity/chrome.rs"]
mod chrome;
#[path = "identity/classify.rs"]
mod classify;
#[path = "identity/kind.rs"]
mod kind;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use classify::viewport_scene_kind;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use kind::ViewportSceneKind;
