//! Local scene ECS scheduling, resource, and event primitives.

#[path = "archetype/mod.rs"]
mod archetype;
#[path = "bundle.rs"]
mod bundle;
#[path = "bundle_transaction_diagnostics.rs"]
mod bundle_transaction_diagnostics;
#[path = "change_detection/mod.rs"]
mod change_detection;
#[path = "channel.rs"]
mod channel;
#[path = "commands/mod.rs"]
mod commands;
#[path = "component/mod.rs"]
mod component;
#[path = "entity/mod.rs"]
mod entity;
#[path = "events/mod.rs"]
mod events;
#[path = "frame_performance_diagnostics.rs"]
mod frame_performance_diagnostics;
#[path = "internal_scene_system.rs"]
mod internal_scene_system;
#[path = "lifecycle.rs"]
mod lifecycle;
#[path = "messages/mod.rs"]
mod messages;
#[path = "native_system_schedule_diagnostics.rs"]
mod native_system_schedule_diagnostics;
#[path = "observer/mod.rs"]
mod observer;
#[path = "query/mod.rs"]
mod query;
#[path = "removal.rs"]
mod removal;
#[path = "resource/mod.rs"]
mod resource;
#[path = "resource_store/mod.rs"]
mod resource_store;
#[path = "scene_system_descriptor.rs"]
mod scene_system_descriptor;
#[path = "scene_system_plan_descriptor.rs"]
mod scene_system_plan_descriptor;
#[path = "scene_system_registry.rs"]
mod scene_system_registry;
#[path = "schedule.rs"]
mod schedule;
#[path = "schedule_execution.rs"]
mod schedule_execution;

pub(crate) use schedule_execution::{schedule_owner_changed_error, ScheduleExecution};
#[path = "schedule_build_receipt.rs"]
mod schedule_build_receipt;
#[path = "schedule_conflict_graph.rs"]
mod schedule_conflict_graph;
#[path = "schedule_error.rs"]
mod schedule_error;
#[path = "schedule_parallel_executor.rs"]
mod schedule_parallel_executor;
#[path = "schedule_runner.rs"]
mod schedule_runner;
#[path = "schedule_stage_plan.rs"]
mod schedule_stage_plan;
#[path = "storage/mod.rs"]
mod storage;
#[path = "storage_type.rs"]
mod storage_type;
#[path = "system/mod.rs"]
mod system;
#[path = "system_set.rs"]
mod system_set;
#[path = "tick_context.rs"]
mod tick_context;

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
    ChangeDetectionScanStats, ChangeTick, ChangeTickWindow, ComponentTicks, Mut, Ref,
    ECS_CHANGE_DETECTION_ADDED_MATCHES_DIAGNOSTIC, ECS_CHANGE_DETECTION_CHANGED_MATCHES_DIAGNOSTIC,
    ECS_CHANGE_DETECTION_SCANNED_MARKS_DIAGNOSTIC,
};
pub(crate) use change_detection::{
    ComponentMutationRecord, ComponentMutationRecorder, ComponentMutationSink,
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
pub use entity::{EntityLocation, EntityRegistryError, StableEntityLocation};
pub(crate) use entity::{EntityRegistry, InternalEntity};
pub(crate) use events::EventObserverHandle;
pub(in crate::scene) use events::EventWriterGrant;
pub use events::{
    Event, EventCapacityMetrics, EventCursor, EventPayloadProfile, EventPayloadStorage,
    EventReadIter, EventReaderLease, EventStore, EventSubscription, EventSubscriptionStatus,
    EventTypeId, Events, EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES, EVENT_INLINE_PAYLOAD_MAX_BYTES,
};
pub(crate) use frame_performance_diagnostics::DetachedEntityBatchOperationStats;
pub use frame_performance_diagnostics::{
    DetachedEntityBatchDiagnostics, EcsFramePerformanceDiagnostics, WorldDerivedStateDiagnostics,
    ECS_DERIVED_STATE_ACTIVE_PROPAGATION_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_ACTIVE_PROPAGATION_PASSES_DIAGNOSTIC,
    ECS_DERIVED_STATE_ACTIVE_PROPAGATION_WRITTEN_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_PARENT_CHAIN_STEPS_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_PARENT_SNAPSHOT_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_TOPOLOGY_REBUILDS_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_TOPOLOGY_REBUILD_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_VALIDITY_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_HIERARCHY_VALIDITY_PASSES_DIAGNOSTIC,
    ECS_DERIVED_STATE_NODE_CACHE_REBUILDS_DIAGNOSTIC,
    ECS_DERIVED_STATE_NODE_CACHE_REBUILT_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_ENTITIES_DIAGNOSTIC,
    ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_PASSES_DIAGNOSTIC,
    ECS_DERIVED_STATE_WORLD_MATRIX_PROPAGATION_WRITTEN_ENTITIES_DIAGNOSTIC,
};
pub use internal_scene_system::InternalSceneSystem;
pub use lifecycle::{ComponentLifecycleEvent, LifecycleEventKind};
pub(in crate::scene) use messages::MessageWriterGrant;
pub use messages::{
    Message, MessageCursor, MessageId, MessageReadIter, MessageRetention, MessageRetentionMetrics,
    MessageStore, Messages,
};
pub(crate) use native_system_schedule_diagnostics::NativeSystemCallbackTiming;
pub use native_system_schedule_diagnostics::{
    NativeSystemScheduleDiagnostics, NATIVE_SYSTEM_CALLBACK_COUNT_DIAGNOSTIC,
    NATIVE_SYSTEM_CALLBACK_P95_MS_DIAGNOSTIC, NATIVE_SYSTEM_CONFLICT_COUNT_DIAGNOSTIC,
    NATIVE_SYSTEM_CONSERVATIVE_WORLD_WRITER_COUNT_DIAGNOSTIC,
    NATIVE_SYSTEM_READY_DELAY_MS_DIAGNOSTIC,
    NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_BYTES_DIAGNOSTIC,
    NATIVE_SYSTEM_TEMPORARY_CONTROL_BUFFER_COUNT_DIAGNOSTIC,
    NATIVE_SYSTEM_WORKER_BATCH_COUNT_DIAGNOSTIC, NATIVE_SYSTEM_WORKER_UTILIZATION_DIAGNOSTIC,
};
pub(crate) use observer::DetachedEntityObservers;
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
    ECS_QUERY_PLAN_COMPILATIONS_DIAGNOSTIC, ECS_QUERY_PLAN_COMPONENT_MEMBERSHIP_CHECKS_DIAGNOSTIC,
    ECS_QUERY_PLAN_SPARSE_BINDINGS_DIAGNOSTIC, ECS_QUERY_PLAN_TABLE_BINDINGS_DIAGNOSTIC,
};
pub use removal::{
    RemovedComponentEvent, RemovedComponentEventIter, RemovedComponentEvents,
    RemovedComponentReadIter, RemovedComponentReader, RemovedComponentRetention,
    RemovedComponentRetentionMetrics, RemovedComponentWriteReceipt,
};
pub use resource::{
    Resource, ResourceDescriptor, ResourceDescriptorSource, ResourceId, ResourceRegistry,
};
pub use resource_store::ResourceStore;
pub(crate) use resource_store::TransferredResourceRow;
pub use scene_system_descriptor::{SceneSystemDescriptor, SystemOrderingConstraint, SystemRef};
pub use scene_system_registry::SceneSystemRegistry;
pub use schedule::Schedule;
pub use schedule_build_receipt::{ScheduleBuildDigest, ScheduleBuildReceipt};
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
pub(crate) use storage::ComponentStorage;
pub(crate) use storage::PreflightedComponentInsert;
pub(crate) use storage::PreflightedTransferredComponentRow;
pub(crate) use storage::StoredComponent;
pub(crate) use storage::TransferredComponentRow;
pub use storage::{ComponentRemoveResult, ComponentStorageLocation, StorageError};
pub use storage_type::StorageType;
pub use system::{
    BoxedRuntimeSceneSystem, BoxedSceneSystem, EventReader, EventReaderParam, EventReaderState,
    EventWriter, EventWriterParam, EventWriterState, FunctionRuntimeSceneSystem,
    FunctionSceneSystem, IntoSceneSystem, IntoWorldlessSceneSystem, Local, LocalParam,
    MessageReader, MessageReaderParam, MessageWriter, MessageWriterParam, ParamSet, ParamSetItem,
    ParamSetParam, Query, RemovedComponents, RemovedComponentsParam, Res, ResMut, ResMutParam,
    ResParam, RuntimeSceneSystem, RuntimeSceneSystemContext, SceneSystem, SceneSystemClockDomain,
    SceneSystemMetadata, SceneSystemPauseBehavior, SceneSystemThreadAffinity,
    SceneSystemTickPolicy, SystemParam, SystemParamAccess, SystemParamConflictKind,
    SystemParamError, SystemState, WorldlessFunctionSceneSystem, WorldlessSystemParam,
};
pub use system_set::{SystemSetId, SystemSetRegistry};
pub use tick_context::SystemTickContext;

pub(crate) use query::single_from_iter;
pub(crate) use schedule_build_receipt::ResolvedScheduleEdge;
pub(crate) use schedule_runner::{SceneScheduleRunner, SceneStageRunError};
pub(crate) use schedule_stage_plan::SceneScheduleStagePlan;
pub(crate) use system::{worldless_private, ScheduledSceneStep, ScheduledSceneStepRef};
pub(crate) use tick_context::SceneStageTickContexts;
