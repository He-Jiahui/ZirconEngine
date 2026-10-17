use std::any::TypeId;

use crate::scene::ecs::{
    Component, QueryState, ResMutParam, ResParam, Resource, ScheduleConflictGraph,
    ScheduleConflictNode, SystemParamAccess, SystemParamConflictKind, SystemStage, SystemState,
    With, Without,
};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHealth(u32);

zircon_runtime_interface::impl_retained_owner_v1!(ScheduleHealth { 0 });
impl Component for ScheduleHealth {}

#[derive(Debug, PartialEq, Eq)]
struct SchedulePlayer;

zircon_runtime_interface::impl_retained_owner_v1!(SchedulePlayer {});
impl Component for SchedulePlayer {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleFrameCounter(u32);

impl Resource for ScheduleFrameCounter {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHitEvent;

#[derive(Debug, PartialEq, Eq)]
struct ScheduleNoticeMessage;

#[path = "conflict_graph/access_conflicts.rs"]
mod access_conflicts;
#[path = "conflict_graph/parallel_batches.rs"]
mod parallel_batches;
