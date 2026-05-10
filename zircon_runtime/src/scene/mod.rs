//! Runtime scene subsystem: level orchestration plus the core ECS world.

mod level_system;
mod level_system_render_extract;
mod module;
mod runtime_level_traits;

pub use level_system::{
    AnimationStateTransitionRuntime, LevelLifecycleState, LevelMetadata, LevelSystem,
};
pub use module::{
    create_default_level, load_level_asset, module_descriptor, DefaultLevelManager, SceneModule,
    WorldDriver, DEFAULT_LEVEL_MANAGER_NAME, LEVEL_MANAGER_NAME, SCENE_MODULE_NAME,
    WORLD_DRIVER_NAME,
};
pub use runtime_level_traits::{RuntimeObject, RuntimeSystem};

pub type EntityId = u64;
pub type NodeId = EntityId;

pub mod components;
pub mod ecs;
pub mod reflect;
mod render_extract;
pub mod semantics;
pub mod serializer;
pub mod world;

pub use reflect::{
    json_from_reflected, reflected_from_json, reflected_from_scene_value,
    scene_value_from_reflected, ReflectComponent, ReflectResource, RuntimeTypeRegistration,
    TypeRegistry, WorldReflection,
};
pub use world::{ComponentTypeRegistry, DynamicComponentInstance, World};

#[allow(unused_imports)]
pub(crate) use components::{default_render_layer_mask, Mobility, NodeKind, NodeRecord};

pub use ecs::{
    Added, ArchetypeId, Bundle, ChangeTick, ChangeTickWindow, Changed, Command, CommandQueue,
    Commands, CommandsParam, Component, ComponentDescriptor, ComponentDescriptorSource,
    ComponentId, ComponentKey, ComponentRegistry, ComponentRemoveResult, ComponentStorage,
    ComponentTicks, EntityCommands, EntityLocation, EntityRegistry, EntityRegistryError,
    EventReader, EventReaderParam, EventStore, EventWriter, EventWriterParam, Events, FnCommand,
    InternalEntity, Local, LocalParam, Mut, ParamSet, ParamSetItem, ParamSetParam, Query,
    QueryAccess, QueryAccessError, QueryData, QueryDataAccess, QueryFilter, QueryIter,
    QueryMutData, QueryState, Ref, RemovedComponentEvent, RemovedComponentEvents,
    RemovedComponentReader, RemovedComponents, RemovedComponentsParam, Res, ResMut, ResMutParam,
    ResParam, Resource, ResourceDescriptor, ResourceId, ResourceRegistry, ResourceStore,
    SceneSystemDescriptor, SceneSystemRegistry, Schedule, StableEntityLocation, StorageError,
    StorageType, SystemParam, SystemParamAccess, SystemParamError, SystemStage, SystemState, With,
    Without,
};

pub type Scene = World;

#[cfg(test)]
mod tests;
