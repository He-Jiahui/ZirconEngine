#[path = "capability.rs"]
mod capability;
#[path = "extension_ids.rs"]
mod extension_ids;
#[path = "overlay.rs"]
mod overlay;
#[path = "plugin.rs"]
mod plugin;
#[path = "ragdoll_profile_editor.rs"]
mod ragdoll_profile_editor;
#[path = "runtime_mirror.rs"]
mod runtime_mirror;
#[path = "viewport_overlay_provider.rs"]
mod viewport_overlay_provider;

#[cfg(test)]
#[path = "tests/overlay_provider_tests.rs"]
mod overlay_provider_tests;
#[cfg(test)]
#[path = "tests/ragdoll_body_translation_tests.rs"]
mod ragdoll_body_translation_tests;
#[cfg(test)]
#[path = "tests/ragdoll_bound_edit_tests.rs"]
mod ragdoll_bound_edit_tests;
#[cfg(test)]
#[path = "tests/ragdoll_profile_edit_tests.rs"]
mod ragdoll_profile_edit_tests;
#[cfg(test)]
#[path = "tests/ragdoll_profile_source_tests.rs"]
mod ragdoll_profile_source_tests;
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
pub use ragdoll_profile_editor::{
    generate_initial_ragdoll_profile, generate_ragdoll_profile_from_skeleton,
    prepare_bound_ragdoll_body_translations, prepare_bound_ragdoll_profile_edits,
    prepare_ragdoll_body_translations, prepare_ragdoll_profile_edits, RagdollBodyTranslation,
    RagdollBodyTranslationError, RagdollBoneEdit, RagdollBoneProperty, RagdollBoundBodyTranslation,
    RagdollBoundBoneEdit, RagdollBoundEditError, RagdollProfileEditCandidate,
    RagdollProfileEditError, RagdollProfileFromSkeleton, RagdollProfileFromSkeletonError,
    RagdollSkeletonBone, RagdollSkeletonBoneKey,
};
pub use runtime_mirror::{
    PhysicsPieFrame, PhysicsPieMirror, PhysicsPieMirrorApply, PHYSICS_OVERLAY_CONSUMER_ID,
    PHYSICS_OVERLAY_FRAME_EVENT_ID, PHYSICS_OVERLAY_FRAME_PAYLOAD_SCHEMA,
};
