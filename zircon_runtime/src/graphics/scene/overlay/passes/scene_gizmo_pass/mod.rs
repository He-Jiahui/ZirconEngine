#[path = "../../../../scene_gizmo_pass/commit_pending_icon_uploads.rs"]
mod commit_pending_icon_uploads;
#[path = "construct.rs"]
mod construct;
#[path = "prepare.rs"]
mod prepare;
#[path = "record.rs"]
mod record;
#[path = "../../../passes/scene_gizmo_pass/scene_gizmo_pass.rs"]
mod scene_gizmo_pass;

pub(crate) use scene_gizmo_pass::SceneGizmoPass;
