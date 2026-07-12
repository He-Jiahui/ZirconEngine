mod capability;
mod extension_ids;
mod overlay;
mod plugin;
mod ragdoll_profile_editor;

#[cfg(test)]
mod tests;

pub use capability::{EDITOR_CAPABILITIES, PHYSICS_AUTHORING_CAPABILITY, PLUGIN_ID};
pub use extension_ids::*;
pub use overlay::{build_physics_overlay, PhysicsOverlayColor, PhysicsOverlayPrimitive};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, PhysicsEditorPlugin,
};
pub use ragdoll_profile_editor::{generate_initial_ragdoll_profile, RagdollSkeletonBone};
