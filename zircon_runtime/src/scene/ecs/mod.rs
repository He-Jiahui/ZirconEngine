//! Local scene ECS scheduling, resource, and event primitives.

mod archetype;
mod bundle;
mod change_detection;
mod commands;
mod component;
mod entity;
mod events;
mod frame_performance_diagnostics;
mod internal_scene_system;
mod lifecycle;
mod messages;
mod observer;
mod query;
mod removal;
mod resource;
mod resource_store;
mod scene_system_descriptor;
mod scene_system_registry;
mod schedule;
mod schedule_conflict_graph;
mod schedule_error;
mod schedule_parallel_executor;
mod schedule_runner;
mod schedule_stage_plan;
mod storage;
mod storage_type;
mod system;
mod system_set;

pub use crate::core::framework::scene::SystemStage;
pub use archetype::{
    ArchetypeId, ArchetypeIndex, ArchetypeMove, ArchetypeRecord, ArchetypeSignature,
};
pub use bundle::Bundle;
pub use change_detection::{
    ChangeDetectionScanStats, ChangeTick, ChangeTickWindow, ComponentTicks, Mut, Ref,
    ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC, ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC,
    ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC,
};
pub use commands::{
    Command, CommandQueue, Commands, CommandsParam, DeferredCommandError, DeferredCommandOperation,
    DeferredCommandReport, EntityCommands, FnCommand,
};
pub use component::{
    Component, ComponentDescriptor, ComponentDescriptorSource, ComponentId, ComponentRegistry,
};
pub use entity::{
    DespawnedEntity, EntityLocation, EntityRegistry, EntityRegistryError, InternalEntity,
    StableEntityLocation,
};
pub use events::{
    Event, EventCapacityMetrics, EventCursor, EventPayloadProfile, EventPayloadStorage,
    EventReadIter, EventStore, EventSubscription, EventSubscriptionStatus, EventTypeId, Events,
    EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES, EVENT_INLINE_PAYLOAD_MAX_BYTES,
};
pub use frame_performance_diagnostics::EcsFramePerformanceDiagnostics;
pub use internal_scene_system::InternalSceneSystem;
pub use lifecycle::{ComponentLifecycleEvent, LifecycleEventKind};
pub use messages::{Message, MessageCursor, MessageId, MessageReadIter, MessageStore, Messages};
pub use observer::{ObserverId, ObserverStore};
pub use query::{
    Added, CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter, Changed,
    QueryAccess, QueryAccessError, QueryCombinationIter, QueryCombinationMutIter, QueryData,
    QueryDataAccess, QueryEntityError, QueryEntityItem, QueryFilter, QueryIter,
    QueryManyCachedIter, QueryManyIter, QueryManyMutIter, QueryManyUniqueMutIter, QueryMutData,
    QueryMutIter, QuerySingleError, QueryState, QueryStateCacheStats, UniqueEntityArray, With,
    Without, ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC,
    ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC, ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC,
};
pub use removal::{RemovedComponentEvent, RemovedComponentEvents, RemovedComponentReader};
pub use resource::{Resource, ResourceDescriptor, ResourceId, ResourceRegistry};
pub use resource_store::ResourceStore;
pub use scene_system_descriptor::{SceneSystemDescriptor, SystemOrderingConstraint, SystemRef};
pub use scene_system_registry::SceneSystemRegistry;
pub use schedule::Schedule;
pub use schedule_conflict_graph::{
    ScheduleConflictEdge, ScheduleConflictGraph, ScheduleConflictNode, ScheduleConflictNodeKind,
    ScheduleParallelBatch,
};
pub use schedule_error::ScheduleError;
pub use schedule_parallel_executor::{
    ScheduleParallelExecutionReport, ScheduleParallelExecutor, ScheduleParallelExecutorError,
    ScheduleParallelTaskRegistry, SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC,
    SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC,
};
pub use storage::{
    ComponentRemoveResult, ComponentStorage, ComponentStorageLocation, StorageError,
};
pub use storage_type::StorageType;
pub use system::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, EventReader, EventReaderParam, EventReaderState,
    EventWriter, EventWriterParam, EventWriterState, FunctionRuntimeSceneSystem,
    FunctionSceneSystem, IntoSceneSystem, Local, LocalParam, MessageReader, MessageReaderParam,
    MessageWriter, MessageWriterParam, ParamSet, ParamSetItem, ParamSetParam, Query,
    RemovedComponents, RemovedComponentsParam, Res, ResMut, ResMutParam, ResParam,
    RuntimeSceneSystem, RuntimeSceneSystemContext, SceneSystem, SceneSystemMetadata, SystemParam,
    SystemParamAccess, SystemParamConflictKind, SystemParamError, SystemState,
};
pub use system_set::{SystemSetId, SystemSetRegistry};

pub(crate) use query::single_from_iter;
pub(crate) use schedule_runner::SceneScheduleRunner;
pub(crate) use schedule_stage_plan::SceneScheduleStagePlan;
pub(crate) use system::{ScheduledSceneStep, ScheduledSceneStepRef};
