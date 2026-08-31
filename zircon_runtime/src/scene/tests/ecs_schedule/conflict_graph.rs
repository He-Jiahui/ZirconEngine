use std::any::TypeId;

use crate::scene::World;
use crate::scene::ecs::{
    Component, QueryState, ResMutParam, ResParam, Resource, ScheduleConflictGraph,
    ScheduleConflictNode, SystemParamAccess, SystemParamConflictKind, SystemStage, SystemState,
    With, Without,
};

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHealth(u32);

impl Component for ScheduleHealth {}

#[derive(Debug, PartialEq, Eq)]
struct SchedulePlayer;

impl Component for SchedulePlayer {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleFrameCounter(u32);

impl Resource for ScheduleFrameCounter {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHitEvent;

#[derive(Debug, PartialEq, Eq)]
struct ScheduleNoticeMessage;

mod access_conflicts;
mod parallel_batches;
