#[path = "command_sort_input.rs"]
mod command_sort_input;
#[path = "geometry_source.rs"]
mod geometry_source;
#[path = "indirect_draw.rs"]
mod indirect_draw;
#[path = "is_skinned.rs"]
mod is_skinned;
#[path = "is_transparent.rs"]
mod is_transparent;
#[path = "../../../mesh/mesh_draw/material_texture_set.rs"]
mod material_texture_set;
#[path = "mesh_draw.rs"]
mod mesh_draw;
#[path = "mesh_pass_batch.rs"]
mod mesh_pass_batch;
#[path = "queue_profile.rs"]
mod queue_profile;
#[path = "uses_indirect_draw.rs"]
mod uses_indirect_draw;
#[path = "../../../../mesh/mesh_draw/virtual_geometry_execution_project.rs"]
mod virtual_geometry_execution_projection;
#[path = "../../../../mesh/mesh_draw/virtual_geometry_submission_detail.rs"]
mod virtual_geometry_submission_detail;

pub(crate) use command_sort_input::MeshCommandSortInput;
pub(crate) use geometry_source::MeshDrawGeometrySource;
pub(in crate::graphics::scene) use material_texture_set::{
    MaterialTextureBinding, MaterialTextureSet,
};
pub(crate) use mesh_draw::MeshDraw;
pub(crate) use queue_profile::{MeshDrawBatchKey, MeshDrawQueuePhase, MeshDrawQueueProfile};
pub(super) use virtual_geometry_submission_detail::VirtualGeometrySubmissionDetail;

#[path = "indexed_plan.rs"]
pub(crate) mod indexed_plan;
pub(crate) use indexed_plan::{VirtualGeometryIndexedPlan, VirtualGeometryIndexedRunRef};
