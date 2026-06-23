use crate::scene::components::{Name, RenderLayerMask};
use crate::scene::ecs::{
    ArchetypeId, Changed, Component, ComponentStorageLocation, Mut, QueryDataAccess,
    QueryEntityError, QueryFilter, QuerySingleError, QueryState, Ref, StableEntityLocation,
    StorageType, UniqueEntityArray, With, Without,
};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Enemy;

impl Component for Enemy {}

#[derive(Debug, PartialEq, Eq)]
struct Player;

impl Component for Player {}

#[derive(Debug, PartialEq, Eq)]
struct SparseScore(u32);

impl Component for SparseScore {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

fn expect_query_error<T>(result: Result<T, QueryEntityError>) -> QueryEntityError {
    match result {
        Ok(_) => panic!("expected query error"),
        Err(error) => error,
    }
}

fn cached_component_locations_for<D, F>(
    query: &QueryState<D, F>,
    index: usize,
) -> &[ComponentStorageLocation]
where
    D: QueryDataAccess,
    F: QueryFilter,
{
    let offsets = query.cached_component_location_offsets();
    let start = *offsets
        .get(index)
        .expect("cached component location start offset");
    let end = *offsets
        .get(index + 1)
        .expect("cached component location end offset");
    &query.cached_component_locations()[start..end]
}

mod cache_helpers;
mod fixed_ticks;
mod iter_many;
mod mutation_access;
mod read_items;
