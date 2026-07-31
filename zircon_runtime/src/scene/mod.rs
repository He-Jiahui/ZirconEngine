//! Runtime scene subsystem: level orchestration plus the core ECS world.

mod event_mirror;
mod level_system;
mod level_system_render_extract;
mod module;
mod navigation;
pub mod prelude;
mod runtime_extension;
mod runtime_hook;
mod runtime_level_traits;

pub use event_mirror::{
    RuntimeEventMirrorDescriptor, RuntimeEventMirrorError, RuntimeEventMirrorRegistration,
    RuntimeEventMirrorSubscription,
};
pub(crate) use event_mirror::{
    RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS, RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
    RUNTIME_EVENT_MIRROR_QUEUE_MAX_EVENTS,
};
pub use level_system::{
    AnimationStateTransitionRuntime, LevelLifecycleState, LevelMetadata, LevelSystem,
};
pub(crate) use module::resolve_default_level_manager;
pub use module::{
    create_default_level, create_level, install_scene_runtime_hooks,
    install_world_runtime_extension_plan, load_level_asset, module_descriptor,
    scene_runtime_hooks_for_stage, DefaultLevelManager, SceneModule, WorldDriver,
    DEFAULT_LEVEL_MANAGER_NAME, WORLD_DRIVER_NAME,
};
pub use navigation::{
    SceneNavigationRuntime, SceneNavigationRuntimeHandle, SCENE_NAVIGATION_RUNTIME_DRIVER_NAME,
};
pub use runtime_extension::{
    WorldRuntimeExtensionError, WorldRuntimeExtensionPlan, WorldRuntimeExtensionRegistration,
};
pub use runtime_hook::{
    SceneRuntimeHook, SceneRuntimeHookContext, SceneRuntimeHookDescriptor,
    SceneRuntimeHookRegistration,
};
pub use runtime_level_traits::{RuntimeObject, RuntimeSystem};

pub type EntityId = u64;
pub type NodeId = EntityId;

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
    DynamicSceneAssetReloadApplyReport, DynamicSceneAssetReloadDrainReport,
    DynamicSceneAssetReloadFrameApplyReport, DynamicSceneAssetReloadPendingReport,
    DynamicSceneAssetReloadPendingTaskSnapshot, DynamicSceneAssetReloadQueue,
    DynamicSceneAssetReloadReadyReport, DynamicSceneAssetReloadResult,
    DynamicSceneAssetReloadSkipReason, DynamicSceneAssetReloadSkippedEvent,
    DynamicSceneAssetReloadStaleResult, DynamicSceneAssetReloadSupersededTask,
    DynamicSceneAssetReloadTask, DynamicSceneAssetReloadTickReport, DynamicSceneError,
    DynamicSceneSpawnTask, EntityRemap, PreparedDynamicSceneSpawn, RuntimeSessionArchive,
    RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveManifest, RuntimeSessionArchiveMergePolicy,
    RuntimeSessionArchiveMergeReport, RuntimeSessionArchivePathStatus,
    RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy,
    RuntimeSessionArchiveSavePreviewReport, RuntimeSessionArchiveStatistics,
    RuntimeSessionLevelRestoreReport, RuntimeSessionMetadata, RuntimeSessionSlot,
    RuntimeSessionSlotCapturePreviewReport, RuntimeSessionSlotDiffReport,
    RuntimeSessionSlotExportPreviewReport, RuntimeSessionSlotImportPreviewReport,
    RuntimeSessionSlotMutationPreviewReport, RuntimeSessionSlotSelectionReport,
    RuntimeSessionSlotSelector, RuntimeSessionSlotSummary, ScenePatch,
    ScenePatchPreviewComponentType, ScenePatchPreviewEntityRemap, ScenePatchPreviewReport,
    ScenePatchPreviewResource, RUNTIME_SESSION_ARCHIVE_FORMAT_VERSION,
};
pub use inspection::{
    WorldInspection, WorldInspectionArtifact, WorldInspectionArtifactDiagnostics,
    WorldInspectionDelta, WorldInspectionField, WorldInspectionFieldDelta,
    WorldInspectionFieldPath, WorldInspectionFieldsArtifact, WorldInspectionHierarchyRow,
    WorldInspectionSummary,
};
pub use reflect::{
    derived_component_registration, derived_component_registration_with_adapter,
    json_from_reflected, reflected_from_json, reflected_from_scene_value,
    scene_value_from_reflected, ReflectComponent, ReflectResource, ReflectedJsonError,
    RuntimeTypeRegistration, TypeRegistry, VmTypeBacking, WorldReflection, ZrReflect,
    ZrReflectValue,
};
pub use world::{ComponentTypeRegistry, DynamicComponentInstance, SceneError, SceneResult, World};

#[allow(unused_imports)]
pub use components::{default_render_layer_mask, Mobility, NodeKind, NodeRecord};

pub use ecs::{
    Added, ArchetypeId, BoxedSceneSystem, Bundle, ChangeTick, ChangeTickWindow, Changed, Command,
    CommandQueue, Commands, CommandsParam, Component, ComponentDescriptor,
    ComponentDescriptorSource, ComponentId, ComponentRegistry, ComponentRemoveResult,
    ComponentStorage, ComponentTicks, EntityCommands, EntityLocation, EntityRegistry,
    EntityRegistryError, EventCapacityMetrics, EventPayloadProfile, EventPayloadStorage,
    EventReader, EventReaderParam, EventStore, EventSubscription, EventSubscriptionStatus,
    EventWriter, EventWriterParam, Events, FnCommand, FunctionSceneSystem, InternalEntity,
    IntoSceneSystem, Local, LocalParam, Mut, ParamSet, ParamSetItem, ParamSetParam, Query,
    QueryAccess, QueryAccessError, QueryData, QueryDataAccess, QueryFilter, QueryIter,
    QueryMutData, QueryState, Ref, RemovedComponentEvent, RemovedComponentEvents,
    RemovedComponentReader, RemovedComponents, RemovedComponentsParam, Res, ResMut, ResMutParam,
    ResParam, Resource, ResourceDescriptor, ResourceId, ResourceRegistry, ResourceStore,
    SceneSystem, SceneSystemDescriptor, SceneSystemMetadata, SceneSystemRegistry, Schedule,
    StableEntityLocation, StorageError, StorageType, SystemParam, SystemParamAccess,
    SystemParamError, SystemStage, SystemState, With, Without,
    EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES, EVENT_INLINE_PAYLOAD_MAX_BYTES,
};

pub type Scene = World;

#[cfg(test)]
mod tests;
