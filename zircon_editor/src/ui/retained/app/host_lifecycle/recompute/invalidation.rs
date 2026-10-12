#[path = "../../recompute/invalidation/decision.rs"]
mod decision;
#[path = "../../recompute/invalidation/fast_path.rs"]
mod fast_path;
#[path = "../../recompute/invalidation/slow_path.rs"]
mod slow_path;

pub(in crate::ui::retained_host::app::host_lifecycle::recompute) use decision::RecomputeInvalidationTarget;
