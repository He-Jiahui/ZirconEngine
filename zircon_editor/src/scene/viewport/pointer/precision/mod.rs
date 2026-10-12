//! 精确拾取层保留形状评分、粗筛范围和渲染空间来源，共享状态只承担当前事件结果，不拥有世界编辑权限。

#[path = "candidate_score.rs"]
mod candidate_score;
#[path = "precision_candidate.rs"]
mod precision_candidate;
#[path = "precision_candidate_score.rs"]
mod precision_candidate_score;
#[path = "precision_shape.rs"]
mod precision_shape;
#[path = "precision_shape_depth.rs"]
mod precision_shape_depth;
#[path = "precision_shape_hit_frame.rs"]
mod precision_shape_hit_frame;
#[path = "precision_shape_score.rs"]
mod precision_shape_score;
#[path = "../../../precision/renderer_visible_spatial_pick_source.rs"]
mod renderer_visible_spatial_pick_source;
#[path = "shared_resolution_state.rs"]
mod shared_resolution_state;

pub(in crate::scene::viewport::pointer) use candidate_score::CandidateScore;
pub(in crate::scene::viewport::pointer) use precision_candidate::PrecisionCandidate;
pub(in crate::scene::viewport::pointer) use precision_shape::PrecisionShape;
pub(in crate::scene::viewport::pointer) use renderer_visible_spatial_pick_source::RendererVisibleSpatialPickSource;
pub(in crate::scene::viewport::pointer) use shared_resolution_state::{
    lock_shared_resolution_state, SharedResolutionState,
};
