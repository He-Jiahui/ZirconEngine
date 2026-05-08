//! Local scene ECS scheduling, resource, and event primitives.

mod archetype_id;
mod bundle;
mod change_detection;
mod commands;
mod component;
mod component_id;
mod component_registry;
mod despawned_entity;
mod entity_location;
mod entity_registry;
mod entity_registry_error;
mod events;
mod internal_entity;
mod internal_scene_system;
mod query;
mod resource;
mod resource_id;
mod resource_registry;
mod resource_store;
mod scene_system_descriptor;
mod scene_system_registry;
mod schedule;
mod schedule_error;
mod schedule_runner;
mod stable_entity_location;
mod storage;
mod storage_type;
mod system;
mod system_stage;

pub use archetype_id::ArchetypeId;
pub use bundle::Bundle;
pub use change_detection::{ChangeTick, ChangeTickWindow, ComponentTicks};
pub use commands::{CommandQueue, Commands, CommandsParam};
pub use component::Component;
pub use component_id::ComponentId;
pub use component_registry::{
    ComponentDescriptor, ComponentDescriptorSource, ComponentKey, ComponentRegistry,
};
pub use despawned_entity::DespawnedEntity;
pub use entity_location::EntityLocation;
pub use entity_registry::EntityRegistry;
pub use entity_registry_error::EntityRegistryError;
pub use events::{EventStore, Events};
pub use internal_entity::InternalEntity;
pub use internal_scene_system::InternalSceneSystem;
pub use query::{
    Added, Changed, QueryAccess, QueryAccessError, QueryData, QueryDataAccess, QueryFilter,
    QueryIter, QueryMutData, QueryState, With, Without,
};
pub use resource::Resource;
pub use resource_id::ResourceId;
pub use resource_registry::{ResourceDescriptor, ResourceRegistry};
pub use resource_store::ResourceStore;
pub use scene_system_descriptor::SceneSystemDescriptor;
pub use scene_system_registry::SceneSystemRegistry;
pub use schedule::Schedule;
pub use schedule_error::ScheduleError;
pub use stable_entity_location::StableEntityLocation;
pub use storage::{ComponentRemoveResult, ComponentStorage, StorageError};
pub use storage_type::StorageType;
pub use system::{
    Local, LocalParam, Query, Res, ResMut, ResMutParam, ResParam, SystemParam, SystemParamAccess,
    SystemParamError, SystemState,
};
pub use system_stage::SystemStage;

pub(crate) use schedule_runner::SceneScheduleRunner;
