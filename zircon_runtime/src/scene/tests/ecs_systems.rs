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

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Player;

impl Component for Player {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

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

mod commands;
mod events;
mod many_single_queries;
mod removal_local;
mod run_window_filters;
mod state_params;
