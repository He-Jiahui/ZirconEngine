//! Runtime scene subsystem: level orchestration plus the core ECS world.

mod event_mirror;
mod level_system;
mod level_system_render_extract;
mod module;
mod navigation;
pub mod prelude;
mod runtime_extension;
mod runtime_level_traits;

pub(crate) use event_mirror::{
    RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS, RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
    RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS, RuntimeEventMirrorDrainPage, RuntimeEventMirrorPayload,
};
pub use event_mirror::{
    RuntimeEventMirrorDescriptor, RuntimeEventMirrorError, RuntimeEventMirrorRegistration,
    RuntimeEventMirrorSubscription,
};
pub use level_system::{
    AnimationStateTransitionRuntime, LevelLifecycleState, LevelMetadata, LevelSystem,
};
pub(crate) use module::resolve_default_level_manager;
pub use module::{
    DEFAULT_LEVEL_MANAGER_NAME, DefaultLevelManager, SceneModule, WORLD_DRIVER_NAME, WorldDriver,
    create_default_level, create_level, install_world_runtime_extension_plan, load_level_asset,
    module_descriptor,
};
pub use navigation::{
    SCENE_NAVIGATION_RUNTIME_DRIVER_NAME, SceneNavigationRuntime, SceneNavigationRuntimeHandle,
};
pub use runtime_extension::{
    WorldRuntimeExtensionError, WorldRuntimeExtensionPlan, WorldRuntimeExtensionRegistration,
};
pub use runtime_level_traits::{RuntimeObject, RuntimeSystem};

pub type EntityId = u64;
pub type NodeId = EntityId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum WorldQueryBudgetError {
    EncodedBytes { observed: usize, limit: usize },
    Items { observed: usize, limit: usize },
    NestingDepth { observed: usize, limit: usize },
    ProcessingTime { limit_micros: u64 },
    Json(String),
}

pub mod components;
pub mod dynamic_scene;
pub mod ecs;
pub mod inspection;
pub mod reflect;
mod render_extract;
pub mod semantics;
pub mod serializer;
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
    MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES, PreparedDynamicSceneSpawn,
    RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION, RuntimeSessionArchive, RuntimeSessionArchiveArtifact,
    RuntimeSessionArchiveArtifactDiagnostics, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, RuntimeSessionArchivePathStatus,
    RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy,
    RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveStatistics,
    RuntimeSessionArchiveWriteSubmission, RuntimeSessionArchiveWriter,
    RuntimeSessionArchiveWriterLimits, RuntimeSessionArchiveWriterSubmitError,
    RuntimeSessionLevelRestoreReport, RuntimeSessionMetadata, RuntimeSessionSlot,
    RuntimeSessionSlotCapturePreviewReport, RuntimeSessionSlotDiffReport,
    RuntimeSessionSlotExportPreviewReport, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotMutationPreviewReport, RuntimeSessionSlotSelectionReport,
    RuntimeSessionSlotSelector, RuntimeSessionSlotSummary, ScenePatch,
    ScenePatchPreviewComponentType, ScenePatchPreviewEntityRemap, ScenePatchPreviewReport,
    ScenePatchPreviewResource,
};
pub use inspection::{
    WorldInspection, WorldInspectionArtifact, WorldInspectionArtifactDiagnostics,
    WorldInspectionDelta, WorldInspectionField, WorldInspectionFieldDelta,
    WorldInspectionFieldPath, WorldInspectionFieldsArtifact, WorldInspectionHierarchyRow,
    WorldInspectionSummary,
};
pub use reflect::{
    ReflectComponent, ReflectResource, ReflectedJsonError, RuntimeTypeRegistration, TypeRegistry,
    VmTypeBacking, WorldReflection, ZrReflect, ZrReflectValue, derived_component_registration,
    derived_component_registration_with_adapter, json_from_reflected, reflected_from_json,
    reflected_from_scene_value, scene_value_from_reflected,
};
pub use world::{
    ComponentTypeRegistry, DetachedEntityBatch, DetachedEntityBatchRestoreError,
    DynamicComponentInstance, SceneError, SceneResult, World,
};

#[allow(unused_imports)]
pub use components::{Mobility, NodeKind, NodeRecord, default_render_layer_mask};

pub use ecs::{
    Added, ArchetypeId, BoxedSceneSystem, Bundle, ChangeTick, ChangeTickWindow, Changed, Command,
    CommandQueue, Commands, CommandsParam, Component, ComponentDescriptor,
    ComponentDescriptorSource, ComponentId, ComponentRegistry, ComponentRemoveResult,
    ComponentStorage, ComponentTicks, EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES,
    EVENT_INLINE_PAYLOAD_MAX_BYTES, EntityCommands, EntityLocation, EntityRegistry,
    EntityRegistryError, EventCapacityMetrics, EventPayloadProfile, EventPayloadStorage,
    EventReader, EventReaderParam, EventStore, EventSubscription, EventSubscriptionStatus,
    EventWriter, EventWriterParam, Events, FnCommand, FunctionSceneSystem, InternalEntity,
    IntoSceneSystem, Local, LocalParam, Mut, ParamSet, ParamSetItem, ParamSetParam, Query,
    QueryAccess, QueryAccessError, QueryData, QueryDataAccess, QueryFilter, QueryIter,
    QueryMutData, QueryState, Ref, RemovedComponentEvent, RemovedComponentEvents,
    RemovedComponentReader, RemovedComponents, RemovedComponentsParam, Res, ResMut, ResMutParam,
    ResParam, Resource, ResourceDescriptor, ResourceId, ResourceRegistry, ResourceStore,
    SceneSystem, SceneSystemClockDomain, SceneSystemDescriptor, SceneSystemMetadata,
    SceneSystemRegistry, Schedule, StableEntityLocation, StorageError, StorageType, SystemParam,
    SystemParamAccess, SystemParamError, SystemStage, SystemState, With, Without,
};

pub type Scene = World;

#[cfg(test)]
mod tests;
