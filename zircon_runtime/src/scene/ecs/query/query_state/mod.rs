// QueryState 子模块分别负责计划、缓存、访问模式和系统参数；本文件只汇总公开入口。
#[path = "archetype_plan.rs"]
mod archetype_plan;
#[path = "cache.rs"]
mod cache;
#[path = "cached_direct.rs"]
mod cached_direct;
#[path = "many_item_array.rs"]
mod many_item_array;
#[path = "mutable.rs"]
mod mutable;
#[path = "read_only.rs"]
mod read_only;
#[path = "read_only_cached.rs"]
mod read_only_cached;
#[path = "state.rs"]
mod state;
#[path = "stats.rs"]
mod stats;
#[path = "system_param.rs"]
mod system_param;

pub(crate) use archetype_plan::{
    find_cached_archetype_plan, project_entity_from_plans, CachedArchetypePlan,
    QueryComponentBinding,
};
pub use state::QueryState;
pub use stats::{
    QueryStateCacheStats, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC,
    ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC,
    ECS_QUERY_PLAN_COMPILATIONS_DIAGNOSTIC, ECS_QUERY_PLAN_COMPONENT_MEMBERSHIP_CHECKS_DIAGNOSTIC,
    ECS_QUERY_PLAN_SPARSE_BINDINGS_DIAGNOSTIC, ECS_QUERY_PLAN_TABLE_BINDINGS_DIAGNOSTIC,
};
