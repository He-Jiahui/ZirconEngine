#[path = "../../identity/classify/architecture.rs"]
mod architecture;
#[path = "../../identity/classify/entry.rs"]
mod entry;
#[path = "../../identity/classify/floor.rs"]
mod floor;
#[path = "../../identity/classify/gate.rs"]
mod gate;
#[path = "../../identity/classify/gizmo.rs"]
mod gizmo;
#[path = "../../identity/classify/lighting.rs"]
mod lighting;
#[path = "../../identity/classify/props.rs"]
mod props;
#[path = "../../identity/classify/surfaces.rs"]
mod surfaces;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::viewport_scene_kind;
