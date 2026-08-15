use std::collections::BTreeMap;
use std::sync::Arc;

use crate::scene::{EntityId, SceneError, World};

/// Stable producer identity for one deferred-command lane. Compiled schedule
/// steps provide this key; direct World commands use the dedicated direct lane.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DeferredSystemKey {
    stage_rank: usize,
    plan_order: i32,
    system_id: Arc<str>,
}

impl DeferredSystemKey {
    pub(crate) fn direct_world() -> Self {
        Self {
            stage_rank: 0,
            plan_order: 0,
            system_id: Arc::from("world-direct"),
        }
    }

    pub(crate) fn direct_system(ordinal: u32) -> Self {
        Self {
            stage_rank: 0,
            plan_order: 0,
            system_id: Arc::from(format!("world-system-{ordinal}")),
        }
    }

    pub(crate) fn compiled(
        stage_rank: usize,
        plan_order: i32,
        system_id: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            stage_rank,
            plan_order,
            system_id: system_id.into(),
        }
    }

    pub(crate) fn registration_placeholder(
        registration_order: i32,
        system_id: impl Into<Arc<str>>,
    ) -> Self {
        Self::compiled(0, registration_order, system_id)
    }

    pub(crate) fn plan_order(&self) -> i32 {
        self.plan_order
    }

    pub(crate) fn system_id(&self) -> &str {
        &self.system_id
    }
}

/// A spawn identity before the deferred barrier has assigned an EntityId.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DeferredSpawnToken {
    key: DeferredSystemKey,
    // A producer can reuse its compiled schedule key in a later apply window.
    // This generation keeps a public handle from one window distinct from a
    // later spawn with the same local ordinal.
    run_generation: u64,
    ordinal: u32,
}

impl DeferredSpawnToken {
    pub(crate) fn new(key: DeferredSystemKey, run_generation: u64, ordinal: u32) -> Self {
        Self {
            key,
            run_generation,
            ordinal,
        }
    }
}

/// An opaque public handle returned by deferred spawn commands.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DeferredEntity {
    token: DeferredSpawnToken,
}

impl DeferredEntity {
    pub(crate) fn new(token: DeferredSpawnToken) -> Self {
        Self { token }
    }

    pub(crate) fn token(&self) -> &DeferredSpawnToken {
        &self.token
    }
}

/// A deferred operation target. Spawns remain tokens until the barrier; an
/// existing entity remains a concrete id because it already belongs to World.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeferredEntityRef {
    Existing(EntityId),
    Spawn(DeferredSpawnToken),
}

impl DeferredEntityRef {
    pub(crate) fn existing(entity: EntityId) -> Self {
        Self::Existing(entity)
    }

    pub(crate) fn spawned(token: DeferredSpawnToken) -> Self {
        Self::Spawn(token)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeferredCommandOperation {
    Spawn,
    Insert,
    InsertBundle,
    Remove,
    Despawn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeferredCommandTarget {
    Resolved(EntityId),
    Pending(DeferredSpawnToken),
}

impl DeferredCommandTarget {
    pub(crate) fn resolved(entity: EntityId) -> Self {
        Self::Resolved(entity)
    }

    pub(crate) fn pending(token: DeferredSpawnToken) -> Self {
        Self::Pending(token)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeferredCommandError {
    operation: DeferredCommandOperation,
    target: DeferredCommandTarget,
    error: SceneError,
}

impl DeferredCommandError {
    pub fn new(
        operation: DeferredCommandOperation,
        target: DeferredCommandTarget,
        error: SceneError,
    ) -> Self {
        Self {
            operation,
            target,
            error,
        }
    }

    pub fn operation(&self) -> DeferredCommandOperation {
        self.operation
    }

    pub fn target(&self) -> &DeferredCommandTarget {
        &self.target
    }

    pub fn error(&self) -> &SceneError {
        &self.error
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeferredCommandReport {
    applied_count: usize,
    errors: Vec<DeferredCommandError>,
    allocation_error: Option<SceneError>,
    resolved_entities: BTreeMap<DeferredSpawnToken, EntityId>,
}

impl DeferredCommandReport {
    pub(crate) fn new(
        applied_count: usize,
        errors: Vec<DeferredCommandError>,
        allocation_error: Option<SceneError>,
        resolved_entities: BTreeMap<DeferredSpawnToken, EntityId>,
    ) -> Self {
        Self {
            applied_count,
            errors,
            allocation_error,
            resolved_entities,
        }
    }

    pub fn applied_count(&self) -> usize {
        self.applied_count
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty() && self.allocation_error.is_none()
    }

    pub fn errors(&self) -> &[DeferredCommandError] {
        &self.errors
    }

    pub fn allocation_error(&self) -> Option<&SceneError> {
        self.allocation_error.as_ref()
    }

    pub fn resolve(&self, entity: &DeferredEntity) -> Option<EntityId> {
        self.resolved_entities.get(entity.token()).copied()
    }
}

pub trait Command: Send + 'static {
    fn apply(self, world: &mut World);
}

pub struct FnCommand<F> {
    command: F,
}

impl<F> FnCommand<F> {
    pub fn new(command: F) -> Self {
        Self { command }
    }
}

impl<F> Command for FnCommand<F>
where
    F: FnOnce(&mut World) + Send + 'static,
{
    fn apply(self, world: &mut World) {
        (self.command)(world);
    }
}

impl<F> Command for F
where
    F: FnOnce(&mut World) + Send + 'static,
{
    fn apply(self, world: &mut World) {
        self(world);
    }
}

pub(crate) trait ErasedCommand: Send {
    fn apply_boxed(self: Box<Self>, world: &mut World);
}

impl<C> ErasedCommand for C
where
    C: Command,
{
    fn apply_boxed(self: Box<Self>, world: &mut World) {
        (*self).apply(world);
    }
}
