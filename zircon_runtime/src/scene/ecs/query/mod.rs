mod cached_query_iter;
mod query_access;
mod query_access_error;
mod query_combinations_iter;
mod query_combinations_mut_iter;
mod query_data;
mod query_entity_error;
mod query_filter;
mod query_iter;
mod query_many_iter;
mod query_many_mut_iter;
mod query_many_unique_mut_iter;
mod query_mut_iter;
mod query_single_error;
mod query_state;
mod unique_entities;

pub use cached_query_iter::{
    CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter,
};
pub use query_access::QueryAccess;
pub use query_access_error::QueryAccessError;
pub use query_combinations_iter::QueryCombinationIter;
pub use query_combinations_mut_iter::QueryCombinationMutIter;
pub use query_data::{QueryData, QueryDataAccess, QueryMutData};
pub use query_entity_error::QueryEntityError;
pub use query_filter::{Added, Changed, QueryFilter, With, Without};
pub use query_iter::QueryIter;
pub use query_many_iter::{QueryEntityItem, QueryManyCachedIter, QueryManyIter};
pub use query_many_mut_iter::QueryManyMutIter;
pub use query_many_unique_mut_iter::QueryManyUniqueMutIter;
pub use query_mut_iter::QueryMutIter;
pub use query_single_error::QuerySingleError;
pub use query_state::{
    QueryState, QueryStateCacheStats, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC,
    ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC,
    ECS_QUERY_PLAN_COMPILATIONS_DIAGNOSTIC, ECS_QUERY_PLAN_COMPONENT_MEMBERSHIP_CHECKS_DIAGNOSTIC,
    ECS_QUERY_PLAN_SPARSE_BINDINGS_DIAGNOSTIC, ECS_QUERY_PLAN_TABLE_BINDINGS_DIAGNOSTIC,
};
pub use unique_entities::UniqueEntityArray;

pub(crate) use query_single_error::single_from_iter;
