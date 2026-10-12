//! World::query 创建持久查询状态；子模块覆盖直接查询、系统运行窗口、缓存路径和读写别名约束。
use crate::scene::components::{Name, RenderLayerMask};
use crate::scene::ecs::{
    ArchetypeId, Changed, Component, Mut, QueryEntityError, QuerySingleError, QueryState, Ref,
    StableEntityLocation, StorageType, UniqueEntityArray, With, Without,
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

#[path = "ecs_query/cached_queries.rs"]
mod cached_queries;
#[path = "ecs_query/fixed_ticks.rs"]
mod fixed_ticks;
#[path = "ecs_query/iter_many.rs"]
mod iter_many;
#[path = "ecs_query/mutation_access.rs"]
mod mutation_access;
#[path = "ecs_query/read_items.rs"]
mod read_items;
