mod capability;
mod extension_ids;
mod overlay;
mod plugin;
mod ragdoll_profile_editor;
mod runtime_mirror;
mod viewport_overlay_provider;

#[cfg(test)]
#[path = "tests/overlay_provider_tests.rs"]
mod overlay_provider_tests;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub use capability::{EDITOR_CAPABILITIES, PHYSICS_AUTHORING_CAPABILITY, PLUGIN_ID};
pub use extension_ids::*;
pub use overlay::{
    build_physics_overlay, build_physics_overlay_extracts, PhysicsOverlayColor,
    PhysicsOverlayPrimitive, PHYSICS_OVERLAY_PROVIDER_ID,
};
pub use plugin::{
    editor_capabilities, editor_host_contract_marker, editor_plugin, editor_plugin_descriptor,
    package_manifest, plugin_registration, PhysicsEditorPlugin,
};
pub use ragdoll_profile_editor::{generate_initial_ragdoll_profile, RagdollSkeletonBone};
pub use runtime_mirror::{
    PhysicsPieFrame, PhysicsPieMirror, PhysicsPieMirrorApply, PHYSICS_OVERLAY_CONSUMER_ID,
    PHYSICS_OVERLAY_FRAME_EVENT_ID, PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA,
};
