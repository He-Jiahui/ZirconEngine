//! 候选层只转换呈现几何与粗拾取参数，不自行决定活动世界授权或最终命中目标。

#[path = "candidate_z_index.rs"]
mod candidate_z_index;
#[path = "gizmo_axis.rs"]
mod gizmo_axis;
#[path = "handle_candidate.rs"]
mod handle_candidate;
#[path = "../../../pointer/candidates/precision_candidates_from_layout.rs"]
mod precision_candidates_from_layout;
#[path = "projected_ring_segments.rs"]
mod projected_ring_segments;
#[path = "renderable_candidate.rs"]
mod renderable_candidate;
#[path = "renderable_candidates.rs"]
mod renderable_candidates;
#[path = "renderable_pick_radius.rs"]
mod renderable_pick_radius;
#[path = "scene_gizmo_candidate.rs"]
mod scene_gizmo_candidate;
#[path = "state_flags.rs"]
mod state_flags;

pub(in crate::scene::viewport::pointer) use candidate_z_index::candidate_z_index;
pub(in crate::scene::viewport::pointer) use gizmo_axis::gizmo_axis;
pub(in crate::scene::viewport::pointer) use handle_candidate::handle_candidate;
pub(in crate::scene::viewport::pointer) use precision_candidates_from_layout::precision_candidates_from_layout;
pub(in crate::scene::viewport) use projected_ring_segments::projected_ring_segments;
pub(in crate::scene::viewport::pointer) use renderable_candidate::renderable_candidate;
pub(in crate::scene::viewport::pointer) use renderable_candidates::renderable_candidates;
pub(in crate::scene::viewport::pointer) use renderable_pick_radius::renderable_pick_radius;
pub(in crate::scene::viewport::pointer) use scene_gizmo_candidate::scene_gizmo_candidate;
pub(in crate::scene::viewport::pointer) use state_flags::{
    interactive_state_flags, passive_state_flags,
};
