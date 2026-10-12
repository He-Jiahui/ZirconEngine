#[path = "animation_fields.rs"]
mod compiled_scene_animation_fields;
#[path = "camera_light_fields.rs"]
mod compiled_scene_camera_light_fields;
#[path = "compiled_scene_dynamic_field.rs"]
mod compiled_scene_dynamic_field;
#[path = "compiled_scene_light_writer.rs"]
mod compiled_scene_light_writer;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "generation.rs"]
mod generation;
#[path = "index.rs"]
mod index;
#[path = "property_path.rs"]
mod property_path;
#[path = "scene_binding_topology.rs"]
mod scene_binding_topology;

pub(super) use diagnostics::CompiledScenePropertyAccessDiagnostics;
pub use diagnostics::CompiledScenePropertyAccessStats;
pub(super) use generation::SceneBindingGenerations;
pub use index::{CompiledDescendantNameEntry, CompiledDescendantNameIndex};
pub use property_path::{
    CompiledScenePropertyTarget, CompiledScenePropertyWriter, ComponentFieldId, PathId,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
