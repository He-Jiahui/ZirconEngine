//! 系统参数测试共享夹具；子模块分别核对命令屏障、事件游标、查询时间窗和状态生命周期。

use crate::scene::components::Name;
use crate::scene::ecs::{
    Added, Changed, CommandsParam, Component, EventReaderParam, EventWriterParam, Local,
    LocalParam, ParamSet, QueryEntityError, QuerySingleError, QueryState, RemovedComponentsParam,
    ResMutParam, ResParam, Resource, SystemParamError, SystemStage, SystemState, UniqueEntityArray,
    With,
};
use crate::scene::{EntityId, World};

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

zircon_runtime_interface::impl_retained_owner_v1!(Health { 0 });
impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Player;

zircon_runtime_interface::impl_retained_owner_v1!(Player {});
impl Component for Player {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

zircon_runtime_interface::impl_retained_owner_v1!(Marker {});
impl Component for Marker {}

#[derive(Debug, PartialEq, Eq)]
struct Score(u32);

impl Resource for Score {}

#[derive(Debug, PartialEq, Eq)]
struct MissingScore(u32);

impl Resource for MissingScore {}

#[derive(Debug, PartialEq, Eq)]
struct HitEvent(u32);

#[derive(Default, Debug, PartialEq, Eq)]
struct LocalCounter(u32);

fn expect_query_error<T>(result: Result<T, QueryEntityError>) -> QueryEntityError {
    match result {
        Ok(_) => panic!("expected query error"),
        Err(error) => error,
    }
}

#[path = "ecs_systems/commands.rs"]
mod commands;
#[path = "ecs_systems/events.rs"]
mod events;
#[path = "ecs_systems/many_single_queries.rs"]
mod many_single_queries;
#[path = "ecs_systems/param_set_commands.rs"]
mod param_set_commands;
#[path = "ecs_systems/removal_local.rs"]
mod removal_local;
#[path = "ecs_systems/run_window_filters.rs"]
mod run_window_filters;
#[path = "ecs_systems/state_params.rs"]
mod state_params;
