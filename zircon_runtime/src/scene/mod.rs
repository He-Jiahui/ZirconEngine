//! Runtime scene subsystem: level orchestration plus the core ECS world.

#[path = "event_mirror/mod.rs"]
mod event_mirror;
#[path = "fixed_step_failure.rs"]
mod fixed_step_failure;
#[path = "level_system.rs"]
mod level_system;
#[path = "level_system_render_extract.rs"]
mod level_system_render_extract;
#[path = "module/mod.rs"]
mod module;
#[path = "navigation.rs"]
mod navigation;
#[path = "prelude.rs"]
pub mod prelude;
#[path = "runtime_extension/mod.rs"]
mod runtime_extension;
#[path = "runtime_level_traits.rs"]
mod runtime_level_traits;
#[path = "world_publication.rs"]
mod world_publication;
#[path = "world_time/mod.rs"]
mod world_time;

pub use event_mirror::{
    RuntimeEventMirrorDescriptor, RuntimeEventMirrorError, RuntimeEventMirrorRegistration,
    RuntimeEventMirrorSubscription,
};
pub(crate) use event_mirror::{
    RuntimeEventMirrorDrainPage, RuntimeEventMirrorPayload, RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS,
    RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES, RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS,
};
pub use fixed_step_failure::{FixedStepFailurePhase, FixedStepFailureReceipt, LevelTickError};
pub use level_system::{
    AnimationStateTransitionRuntime, LevelLifecycleState, LevelMetadata, LevelSystem,
};
pub(crate) use module::resolve_default_level_manager;
pub use module::{
    create_default_level, create_level, install_world_runtime_extension_plan, load_level_asset,
    module_descriptor, prepare_level, DefaultLevelManager, PreparedLevel,
    PreparedLevelPublishError, SceneModule, WorldDriver, DEFAULT_LEVEL_MANAGER_NAME,
    WORLD_DRIVER_NAME,
};
pub use navigation::{
    SceneNavigationRuntime, SceneNavigationRuntimeHandle, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME,
};
pub use runtime_extension::{
    WorldRuntimeExtensionError, WorldRuntimeExtensionPlan, WorldRuntimeExtensionRegistration,
};
pub use runtime_level_traits::{RuntimeObject, RuntimeSystem};
pub use world_publication::{WorldPublicationError, WorldPublicationSource};
pub use world_time::{
    FixedInterpolationContext, FixedInterpolationState, SimulationTickId, WorldTimeAdvanceError,
    WorldTimeControlError, WorldTimePolicyReceipt, WorldTimeState,
};
pub use zircon_runtime_interface::world_sync::LifecycleTargetChange;

pub type EntityId = u64;
pub type NodeId = EntityId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorldQueryBudgetError {
    EncodedBytes { observed: usize, limit: usize },
    Items { observed: usize, limit: usize },
    NestingDepth { observed: usize, limit: usize },
    ProcessingTime { limit_micros: u64 },
    ReflectValue(String),
    Json(String),
}

#[path = "components/mod.rs"]
pub mod components;
#[path = "dynamic/mod.rs"]
pub mod dynamic_scene;
#[path = "ecs/mod.rs"]
pub mod ecs;
#[path = "inspection/mod.rs"]
pub mod inspection;
#[path = "reflect/mod.rs"]
pub mod reflect;
#[path = "render_extract/mod.rs"]
mod render_extract;
#[path = "semantics/mod.rs"]
pub mod semantics;
#[path = "serializer/mod.rs"]
pub mod serializer;
#[path = "world/mod.rs"]
pub mod world;

pub use dynamic_scene::{
    DynamicComponent, DynamicEntity, DynamicResource, DynamicScene,
    DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure,
    DynamicSceneAssetReloadApplyReport, DynamicSceneAssetReloadDiagnostics,
    DynamicSceneAssetReloadDrainReport, DynamicSceneAssetReloadFrameApplyReport,
    DynamicSceneAssetReloadLimits, DynamicSceneAssetReloadPendingReport,
    DynamicSceneAssetReloadPendingTaskSnapshot, DynamicSceneAssetReloadQueue,
    DynamicSceneAssetReloadSkipReason, DynamicSceneAssetReloadSkippedEvent,
    DynamicSceneAssetReloadStaleResult, DynamicSceneAssetReloadSupersededTask,
    DynamicSceneAssetReloadTask, DynamicSceneError, DynamicSceneSpawnTask, EntityRemap,
    PreparedDynamicSceneSpawn, RuntimeSessionArchive, RuntimeSessionArchiveArtifact,
    RuntimeSessionArchiveArtifactDiagnostics, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, RuntimeSessionArchivePathStatus,
    RuntimeSessionArchivePruneReport, RuntimeSessionArchiveReadArtifact,
    RuntimeSessionArchiveReadOutcome, RuntimeSessionArchiveReadSubmission,
    RuntimeSessionArchiveReader, RuntimeSessionArchiveReaderDiagnostics,
    RuntimeSessionArchiveReaderLimits, RuntimeSessionArchiveReaderSubmitError,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionArchiveSavePreviewReport,
    RuntimeSessionArchiveStatistics, RuntimeSessionArchiveWriteSubmission,
    RuntimeSessionArchiveWriter, RuntimeSessionArchiveWriterLimits,
    RuntimeSessionArchiveWriterSubmitError, RuntimeSessionLevelRestoreReport,
    RuntimeSessionMetadata, RuntimeSessionSlot, RuntimeSessionSlotCapturePreviewReport,
    RuntimeSessionSlotDiffReport, RuntimeSessionSlotExportPreviewReport,
    RuntimeSessionSlotImportPreviewReport, RuntimeSessionSlotMutationPreviewReport,
    RuntimeSessionSlotSelectionReport, RuntimeSessionSlotSelector, RuntimeSessionSlotSummary,
    ScenePatch, ScenePatchPreviewComponentType, ScenePatchPreviewEntityRemap,
    ScenePatchPreviewReport, ScenePatchPreviewResource, MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES,
    RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
};
pub use inspection::{
    WorldInspectionArtifact, WorldInspectionArtifactDiagnostics, WorldInspectionDelta,
    WorldInspectionField, WorldInspectionFieldDelta, WorldInspectionFieldPath,
    WorldInspectionFieldsArtifact, WorldInspectionHierarchyRow, WorldInspectionSummary,
};
pub use reflect::{
    derived_component_registration, derived_component_registration_with_adapter,
    json_from_reflected, reflected_from_json, reflected_from_scene_value,
    scene_value_from_reflected, ReflectComponent, ReflectResource, ReflectedJsonError,
    RuntimeTypeRegistration, TypeRegistry, VmTypeBacking, WorldReflection, ZrReflect,
    ZrReflectValue,
};
pub use world::{
    ComponentTypeRegistry, DetachedEntityBatch, DetachedEntityBatchRemoveError,
    DetachedEntityBatchRestoreError, DynamicComponentInstance, PreparedEntitySubtrees, SceneError,
    SceneResult, World,
};

pub use components::{default_render_layer_mask, Mobility, NodeKind, NodeRecord};

pub use ecs::{
    Added, ArchetypeId, BoxedSceneSystem, Bundle, ChangeTick, ChangeTickWindow, Changed, Command,
    CommandQueue, Commands, CommandsParam, Component, ComponentDescriptor,
    ComponentDescriptorSource, ComponentId, ComponentRegistry, ComponentRemoveResult,
    ComponentTicks, EntityCommands, EntityLocation, EntityRegistryError, EventCapacityMetrics,
    EventPayloadProfile, EventPayloadStorage, EventReader, EventReaderParam, EventStore,
    EventSubscription, EventSubscriptionStatus, EventWriter, EventWriterParam, Events, FnCommand,
    FunctionSceneSystem, IntoSceneSystem, Local, LocalParam, Mut, ParamSet, ParamSetItem,
    ParamSetParam, Query, QueryAccess, QueryAccessError, QueryData, QueryDataAccess, QueryFilter,
    QueryIter, QueryMutData, QueryState, Ref, RemovedComponentEvent, RemovedComponentEventIter,
    RemovedComponentEvents, RemovedComponentReadIter, RemovedComponentReader,
    RemovedComponentRetention, RemovedComponentRetentionMetrics, RemovedComponentWriteReceipt,
    RemovedComponents, RemovedComponentsParam, Res, ResMut, ResMutParam, ResParam, Resource,
    ResourceDescriptor, ResourceId, ResourceRegistry, ResourceStore, SceneSystem,
    SceneSystemClockDomain, SceneSystemDescriptor, SceneSystemMetadata, SceneSystemPauseBehavior,
    SceneSystemRegistry, SceneSystemTickPolicy, Schedule, ScheduleBuildDigest,
    ScheduleBuildReceipt, StableEntityLocation, StorageError, StorageType, SystemParam,
    SystemParamAccess, SystemParamError, SystemStage, SystemState, SystemTickContext, With,
    Without, EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES, EVENT_INLINE_PAYLOAD_MAX_BYTES,
};
pub(crate) use ecs::{ComponentStorage, EntityRegistry, InternalEntity};
pub(crate) use world::{RenderDirtyEntityJournal, RenderDirtyWorldId};

pub type Scene = World;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;
