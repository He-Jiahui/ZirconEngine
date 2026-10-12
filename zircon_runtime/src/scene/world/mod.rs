//! ECS world state, project I/O, and render extraction.

#[path = "bootstrap.rs"]
mod bootstrap;
#[path = "change_detection.rs"]
pub(in crate::scene) mod change_detection;
#[path = "commands.rs"]
mod commands;
#[path = "compiled_binding/mod.rs"]
mod compiled_binding;
#[path = "component_access.rs"]
mod component_access;
#[path = "component_type_registry.rs"]
mod component_type_registry;
#[path = "deferred_structural_segment.rs"]
mod deferred_structural_segment;
#[path = "derived_state.rs"]
mod derived_state;
#[path = "dirty_state.rs"]
mod dirty_state;
#[path = "dynamic_components.rs"]
mod dynamic_components;
#[path = "entity_id_allocator.rs"]
mod entity_id_allocator;
#[path = "error.rs"]
mod error;
#[path = "event_mirror.rs"]
mod event_mirror;
#[path = "events.rs"]
mod events;
#[path = "generation.rs"]
mod generation;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "hierarchy_topology.rs"]
mod hierarchy_topology;
#[path = "hierarchy_validation.rs"]
mod hierarchy_validation;
#[path = "identity.rs"]
mod identity;
#[path = "messages.rs"]
mod messages;
#[path = "observers.rs"]
mod observers;
#[path = "performance_diagnostics.rs"]
mod performance_diagnostics;
#[path = "project_io.rs"]
mod project_io;
#[path = "property_access/mod.rs"]
mod property_access;
#[path = "query.rs"]
mod query;
#[path = "query_order.rs"]
mod query_order;
#[path = "records.rs"]
mod records;
#[path = "render.rs"]
mod render;
#[path = "render_component_changes/mod.rs"]
mod render_component_changes;
#[path = "render_dirty_journal/mod.rs"]
mod render_dirty_journal;
#[path = "render_particles.rs"]
mod render_particles;
#[path = "render_post_process.rs"]
mod render_post_process;
#[path = "render_visibility.rs"]
mod render_visibility;
#[path = "schedule.rs"]
mod schedule;
#[path = "staging_snapshot.rs"]
mod staging_snapshot;
#[path = "transaction.rs"]
mod transaction;
#[path = "transform_validation.rs"]
mod transform_validation;
#[path = "typed_api.rs"]
mod typed_api;
#[path = "world.rs"]
mod world;

pub use compiled_binding::{
    CompiledDescendantNameEntry, CompiledDescendantNameIndex, CompiledScenePropertyAccessStats,
    CompiledScenePropertyTarget, CompiledScenePropertyWriter, ComponentFieldId, PathId,
};
pub use component_type_registry::ComponentTypeRegistry;
pub(crate) use deferred_structural_segment::DeferredStructuralBatch;
pub(in crate::scene) use dynamic_components::json_from_scene_property_value;
pub use dynamic_components::DynamicComponentInstance;
pub use error::{SceneError, SceneResult};
pub use project_io::SceneComponentSerializer;
pub(crate) use project_io::SceneComponentSerializerRegistry;
pub use project_io::SceneProjectError;
pub(crate) use query_order::{StableQueryLocationIter, StableWorldEntityIter};
pub(crate) use render_dirty_journal::{RenderDirtyEntityJournal, RenderDirtyWorldId};
pub(crate) use transaction::DetachPreparationOwner;
pub use transaction::{
    DetachedEntityBatch, DetachedEntityBatchRemoveError, DetachedEntityBatchRestoreError,
    PreparedEntitySubtrees,
};
pub(in crate::scene) use transaction::{PreflightComponentRow, PreflightDynamicComponent};
pub use world::World;
