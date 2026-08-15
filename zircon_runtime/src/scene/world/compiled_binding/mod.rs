mod compiled_scene_animation_fields;
mod compiled_scene_camera_light_fields;
mod compiled_scene_dynamic_field;
mod compiled_scene_light_writer;
mod diagnostics;
mod generation;
mod index;
mod property_path;
mod scene_binding_topology;

pub(super) use diagnostics::CompiledScenePropertyAccessDiagnostics;
pub use diagnostics::CompiledScenePropertyAccessStats;
pub(super) use generation::SceneBindingGenerations;
pub use index::{CompiledDescendantNameEntry, CompiledDescendantNameIndex};
pub use property_path::{
    CompiledScenePropertyTarget, CompiledScenePropertyWriter, ComponentFieldId, PathId,
};

#[cfg(test)]
mod tests;
