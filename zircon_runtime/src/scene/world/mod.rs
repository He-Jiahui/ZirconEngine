//! ECS world state, project I/O, and render extraction.

mod bootstrap;
mod change_detection;
mod commands;
mod compiled_binding;
mod component_access;
mod component_type_registry;
mod deferred_structural_segment;
mod derived_state;
mod dirty_state;
mod dynamic_components;
mod entity_id_allocator;
mod error;
mod event_mirror;
mod events;
mod generation;
mod hierarchy;
mod hierarchy_topology;
mod hierarchy_validation;
mod identity;
mod messages;
mod observers;
mod performance_diagnostics;
mod project_io;
mod property_access;
mod query;
mod query_order;
mod records;
mod render;
mod render_component_changes;
mod render_dirty_journal;
mod render_particles;
mod render_post_process;
mod render_visibility;
mod schedule;
mod staging_snapshot;
mod transaction;
mod transform_validation;
mod typed_api;
mod world;

pub use compiled_binding::{
    CompiledDescendantNameEntry, CompiledDescendantNameIndex, CompiledScenePropertyAccessStats,
    CompiledScenePropertyTarget, CompiledScenePropertyWriter, ComponentFieldId, PathId,
};
pub use component_type_registry::ComponentTypeRegistry;
pub(crate) use deferred_structural_segment::DeferredStructuralBatch;
pub use dynamic_components::DynamicComponentInstance;
pub(in crate::scene) use dynamic_components::json_from_scene_property_value;
pub use error::{SceneError, SceneResult};
pub use project_io::SceneProjectError;
pub(crate) use query_order::{StableQueryLocationIter, StableWorldEntityIter};
pub(crate) use render_dirty_journal::{RenderDirtyEntityJournal, RenderDirtyWorldId};
pub use transaction::{DetachedEntityBatch, DetachedEntityBatchRestoreError};
pub(in crate::scene) use transaction::{PreflightComponentRow, PreflightDynamicComponent};
pub use world::World;
