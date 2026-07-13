mod authoring_bindings;
mod capability;
mod extension_ids;
mod live_output;
mod plugin;

pub use authoring_bindings::{
    register_sound_authoring_bindings, sound_editor_command_descriptors,
    SOUND_AUDIO_LISTENER_OPERATION_PATHS, SOUND_AUDIO_SOURCE_OPERATION_PATHS,
    SOUND_AUDIO_VOLUME_OPERATION_PATHS, SOUND_MIXER_OPERATION_PATHS,
};
pub use capability::{EDITOR_CAPABILITIES, PLUGIN_ID, SOUND_AUTHORING_CAPABILITY};
pub use extension_ids::{
    SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID, SOUND_ACOUSTIC_DEBUG_VIEW_ID, SOUND_AUDIO_LISTENER_DRAWER_ID,
    SOUND_AUDIO_SOURCE_DRAWER_ID, SOUND_AUDIO_VOLUME_DRAWER_ID, SOUND_AUTHORING_VIEW_ID,
    SOUND_DRAWER_ID, SOUND_TEMPLATE_ID,
};
pub use live_output::{
    SoundEditorLiveOutputController, SoundEditorOutputAction, SoundEditorOutputActionReport,
    SoundEditorOutputDeviceRow, SoundEditorOutputSnapshot, SoundEditorOutputStatusModel,
};
pub use plugin::{
    component_drawer_ids, editor_capabilities, editor_host_contract_marker, editor_plugin,
    editor_plugin_descriptor, package_manifest, plugin_registration, SoundEditorPlugin,
};

#[cfg(test)]
mod tests;
