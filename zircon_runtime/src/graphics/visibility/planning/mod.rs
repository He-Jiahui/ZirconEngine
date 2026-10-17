#[path = "build_bvh_update_plan.rs"]
pub(super) mod build_bvh_update_plan;
#[path = "build_draw_commands.rs"]
pub(super) mod build_draw_commands;
#[path = "build_instance_upload_plan.rs"]
pub(super) mod build_instance_upload_plan;
#[path = "build_particle_upload_plan.rs"]
pub(super) mod build_particle_upload_plan;
#[path = "geometry_plan/mod.rs"]
pub(super) mod build_virtual_geometry_plan;

pub(crate) use build_virtual_geometry_plan::cluster_ids_for_stable_instance_key;
