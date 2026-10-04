//! 精确拾取层保留形状评分、粗筛范围和渲染空间来源，共享状态只承担当前事件结果，不拥有世界编辑权限。

mod candidate_score;
mod precision_candidate;
mod precision_candidate_score;
mod precision_shape;
mod precision_shape_depth;
mod precision_shape_hit_frame;
mod precision_shape_score;
mod renderer_visible_spatial_pick_source;
mod shared_resolution_state;

pub(in crate::scene::viewport::pointer) use candidate_score::CandidateScore;
pub(in crate::scene::viewport::pointer) use precision_candidate::PrecisionCandidate;
pub(in crate::scene::viewport::pointer) use precision_shape::PrecisionShape;
pub(in crate::scene::viewport::pointer) use renderer_visible_spatial_pick_source::RendererVisibleSpatialPickSource;
pub(in crate::scene::viewport::pointer) use shared_resolution_state::{
    lock_shared_resolution_state, SharedResolutionState,
};
