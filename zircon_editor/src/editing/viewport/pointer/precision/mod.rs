mod candidate_score;
mod precision_candidate;
mod precision_candidate_score;
mod precision_shape;
mod precision_shape_depth;
mod precision_shape_hit_frame;
mod precision_shape_score;
mod shared_resolution_state;

pub(in crate::editing::viewport::pointer) use candidate_score::CandidateScore;
pub(in crate::editing::viewport::pointer) use precision_candidate::PrecisionCandidate;
pub(in crate::editing::viewport::pointer) use precision_shape::PrecisionShape;
pub(in crate::editing::viewport::pointer) use shared_resolution_state::SharedResolutionState;
