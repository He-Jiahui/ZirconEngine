//! Local scene ECS scheduling, resource, and event primitives.

mod archetype;
mod bundle;
mod bundle_transaction_diagnostics;
mod change_detection;
mod commands;
mod component;
mod entity;
mod events;
mod frame_performance_diagnostics;
mod internal_scene_system;
mod lifecycle;
mod messages;
mod native_system_schedule_diagnostics;
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
    ArchetypeId, ArchetypeIndex, ArchetypeIndexPerformanceStats, ArchetypeRecord,
    ArchetypeSignature, ECS_ARCHETYPE_COMPONENT_INDEX_PROBES_DIAGNOSTIC,
    ECS_ARCHETYPE_ROW_APPENDS_DIAGNOSTIC, ECS_ARCHETYPE_SIGNATURE_MEMBERSHIP_CHECKS_DIAGNOSTIC,
};
pub use bundle::Bundle;
pub use bundle::BundleStaging;
pub use bundle_transaction_diagnostics::{
    BundleTransactionDiagnostics, ECS_BUNDLE_FINAL_ARCHETYPE_TRANSITIONS_DIAGNOSTIC,
    ECS_BUNDLE_INTERMEDIATE_SIGNATURES_DIAGNOSTIC, ECS_BUNDLE_LIFECYCLE_EVENTS_DIAGNOSTIC,
    ECS_BUNDLE_STAGING_ALLOCATIONS_DIAGNOSTIC, ECS_BUNDLE_STORAGE_MOVES_DIAGNOSTIC,
    ECS_BUNDLE_TRANSACTION_COUNT_DIAGNOSTIC,
};
pub use change_detection::{
    ChangeDetectionScanStats, ChangeTick, ChangeTickWindow, ComponentTicks,
    ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC, ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC,
    ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC, Mut, Ref,
};
pub use commands::{
    Command, CommandQueue, CommandQueueMetrics, Commands, CommandsParam, DeferredCommandError,
    DeferredCommandOperation, DeferredCommandReport, DeferredCommandTarget, DeferredEntity,
    DeferredEntityRef, DeferredSpawnToken, DeferredSystemKey, EntityCommands, FnCommand,
    WorkerCommandBuffer, WorkerCommandBufferMergeError,
};
pub(crate) use commands::{
    DeferredStructuralKind, DeferredStructuralMetadata, QueuedStructuralCommand,
};
pub use component::{
    Component, ComponentDescriptor, ComponentDescriptorSource, ComponentId, ComponentRegistry,
};
pub(crate) use component::{
    PreflightedTransferredDescriptorImports, TransferredComponentDescriptor,
};
pub use entity::{
    DespawnedEntity, EntityLocation, EntityRegistry, EntityRegistryError, InternalEntity,
    StableEntityLocation,
};
pub(crate) use events::EventObserverHandle;
pub use events::{
    EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES, EVENT_INLINE_PAYLOAD_MAX_BYTES, Event,
    EventCapacityMetrics, EventCursor, EventPayloadProfile, EventPayloadStorage, EventReadIter,
    EventStore, EventSubscription, EventSubscriptionStatus, EventTypeId, Events,
};
pub(crate) use frame_performance_diagnostics::DetachedEntityBatchOperationStats;
pub use frame_performance_diagnostics::{
    DetachedEntityBatchDiagnostics, ECS_DERIVED_STATE_ACTIVE_PROPAGATION_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_ACTIVE_PROPAGATION_PASSES_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_PARENT_CHAIN_STEPS_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_PARENT_SNAPSHOT_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_TOPOLOGY_REBUILD_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_TOPOLOGY_REBUILDS_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_VALIDITY_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_VALIDITY_PASSES_DIAGNOSTIC,
    ECS_DERIVED_STATE_NODE_CACHE_REBUILDS_DIAGNOSTIC,
    ECS_DERIVED_STATE_NODE_CACHE_REBUILT_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_PASSES_DIAGNOSTIC, EcsFramePerformanceDiagnostics,
    WorldDerivedStateDiagnostics,
};
pub use internal_scene_system::InternalSceneSystem;
pub use lifecycle::{ComponentLifecycleEvent, LifecycleEventKind};
pub use messages::{
    Message, MessageCursor, MessageId, MessageReadIter, MessageRetention, MessageRetentionMetrics,
    MessageStore, Messages,
};
pub(crate) use native_system_schedule_diagnostics::NativeSystemCallbackTiming;
pub use native_system_schedule_diagnostics::{
    NATIVE_SYSTEM_CALLBACK_COUNT_DIAGNOSTIC, NATIVE_SYSTEM_CALLBACK_P95_MS_DIAGNOSTIC,
    NATIVE_SYSTEM_CONFLICT_COUNT_DIAGNOSTIC,
    NATIVE_SYSTEM_CONSERVATIVE_WORLD_WRITER_COUNT_DIAGNOSTIC,
    NATIVE_SYSTEM_READY_DELAY_MS_DIAGNOSTIC,
    NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_BYTES_DIAGNOSTIC,
    NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_COUNT_DIAGNOSTIC,
    NATIVE_SYSTEM_WORKER_BATCH_COUNT_DIAGNOSTIC, NATIVE_SYSTEM_WORKER_UTILIZATION_DIAGNOSTIC,
    NativeSystemScheduleDiagnostics,
};
pub(crate) use observer::DetachedEntityObservers;
pub use observer::{ObserverId, ObserverStore};
pub use query::{
    Added, CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter, Changed,
    ECS_QUERY_ARCHETYPE_CACHE_HITS_DIAGNOSTIC, ECS_QUERY_ARCHETYPE_CACHE_MISSES_DIAGNOSTIC,
    ECS_QUERY_ARCHETYPE_CACHE_REBUILDS_DIAGNOSTIC, ECS_QUERY_CANDIDATE_ENTITIES_DIAGNOSTIC,
    ECS_QUERY_MATCHED_ENTITIES_DIAGNOSTIC, ECS_QUERY_PLAN_COMPILATIONS_DIAGNOSTIC,
    ECS_QUERY_PLAN_COMPONENT_MEMBERSHIP_CHECKS_DIAGNOSTIC,
    ECS_QUERY_PLAN_SPARSE_BINDINGS_DIAGNOSTIC, ECS_QUERY_PLAN_TABLE_BINDINGS_DIAGNOSTIC,
    QueryAccess, QueryAccessError, QueryCombinationIter, QueryCombinationMutIter, QueryData,
    QueryDataAccess, QueryEntityError, QueryEntityItem, QueryFilter, QueryIter,
    QueryManyCachedIter, QueryManyIter, QueryManyMutIter, QueryManyUniqueMutIter, QueryMutData,
    QueryMutIter, QuerySingleError, QueryState, QueryStateCacheStats, UniqueEntityArray, With,
    Without,
};
pub use removal::{RemovedComponentEvent, RemovedComponentEvents, RemovedComponentReader};
pub use resource::{
    Resource, ResourceDescriptor, ResourceDescriptorSource, ResourceId, ResourceRegistry,
};
pub use resource_store::ResourceStore;
pub(crate) use resource_store::TransferredResourceRow;
pub use scene_system_descriptor::{SceneSystemDescriptor, SystemOrderingConstraint, SystemRef};
pub use scene_system_registry::SceneSystemRegistry;
pub use schedule::Schedule;
pub use schedule_conflict_graph::{
    ScheduleConflictEdge, ScheduleConflictGraph, ScheduleConflictNode, ScheduleConflictNodeKind,
    ScheduleParallelBatch,
};
pub use schedule_error::ScheduleError;
pub use schedule_parallel_executor::{
    SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC, SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC,
    ScheduleParallelExecutionReport, ScheduleParallelExecutor, ScheduleParallelExecutorError,
    ScheduleParallelTaskRegistry,
};
pub(crate) use storage::PreflightedComponentInsert;
pub(crate) use storage::PreflightedTransferredComponentRow;
pub(crate) use storage::StoredComponent;
pub(crate) use storage::TransferredComponentRow;
pub use storage::{
    ComponentRemoveResult, ComponentStorage, ComponentStorageLocation, StorageError,
};
pub use storage_type::StorageType;
pub use system::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, EventReader, EventReaderParam, EventReaderState,
    EventWriter, EventWriterParam, EventWriterState, FunctionRuntimeSceneSystem,
    FunctionSceneSystem, IntoSceneSystem, IntoWorldlessSceneSystem, Local, LocalParam,
    MessageReader, MessageReaderParam, MessageWriter, MessageWriterParam, ParamSet, ParamSetItem,
    ParamSetParam, Query, RemovedComponents, RemovedComponentsParam, Res, ResMut, ResMutParam,
    ResParam, RuntimeSceneSystem, RuntimeSceneSystemContext, SceneSystem, SceneSystemClockDomain,
    SceneSystemMetadata, SceneSystemThreadAffinity, SystemParam, SystemParamAccess,
    SystemParamConflictKind, SystemParamError, SystemState, WorldlessFunctionSceneSystem,
    WorldlessSystemParam,
};
pub use system_set::{SystemSetId, SystemSetRegistry};

pub(crate) use query::single_from_iter;
pub(crate) use schedule_runner::SceneScheduleRunner;
pub(crate) use schedule_stage_plan::SceneScheduleStagePlan;
pub(crate) use system::{ScheduledSceneStep, ScheduledSceneStepRef, worldless_private};
