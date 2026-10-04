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

mod cached_queries;
mod fixed_ticks;
mod iter_many;
mod mutation_access;
mod read_items;
