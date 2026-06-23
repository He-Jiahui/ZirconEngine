use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

use crate::authoring_bindings::{
    register_sound_authoring_bindings, sound_audio_listener_drawer_descriptor,
    sound_audio_source_drawer_descriptor, sound_audio_volume_drawer_descriptor,
};
use crate::capability::{PLUGIN_ID, SOUND_AUTHORING_CAPABILITY};
use crate::extension_ids::{
    SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID, SOUND_ACOUSTIC_DEBUG_VIEW_ID, SOUND_AUDIO_LISTENER_DRAWER_ID,
    SOUND_AUDIO_SOURCE_DRAWER_ID, SOUND_AUDIO_VOLUME_DRAWER_ID, SOUND_AUTHORING_VIEW_ID,
    SOUND_DRAWER_ID, SOUND_TEMPLATE_ID,
};

#[derive(Clone, Debug)]
pub struct SoundEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl SoundEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for SoundEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }

    fn register_editor_extensions(
        &self,
        registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
    ) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
        register_authoring_extensions(
            registry,
            EditorAuthoringExtensions {
                drawer_id: SOUND_DRAWER_ID,
                drawer_display_name: "Sound Mixer",
                template_id: SOUND_TEMPLATE_ID,
                template_document: "plugins://sound/editor/mixer_console.zui",
                surfaces: &[
                    EditorAuthoringSurface::new(
                        SOUND_AUTHORING_VIEW_ID,
                        "Sound Mixer",
                        "Audio",
                        "Plugins/Sound/Mixer",
                    ),
                    EditorAuthoringSurface::new(
                        SOUND_ACOUSTIC_DEBUG_VIEW_ID,
                        "Acoustic Debug",
                        "Audio",
                        "Plugins/Sound/Acoustic Debug",
                    ),
                ],
            },
        )?;
        register_sound_authoring_bindings(registry)?;
        register_sound_component_drawers(registry)
    }
}

fn register_sound_component_drawers(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    use zircon_editor::core::editor_extension::EditorUiTemplateDescriptor;

    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID,
        "plugins://sound/editor/acoustic_debug.zui",
    ))?;
    registry.register_component_drawer(sound_audio_source_drawer_descriptor())?;
    registry.register_component_drawer(sound_audio_listener_drawer_descriptor())?;
    registry.register_component_drawer(sound_audio_volume_drawer_descriptor())
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(PLUGIN_ID, "Sound", "zircon_plugin_sound_editor")
        .with_capability(SOUND_AUTHORING_CAPABILITY)
}

pub fn editor_plugin() -> SoundEditorPlugin {
    SoundEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_sound_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_sound_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}

pub fn component_drawer_ids() -> [&'static str; 3] {
    [
        SOUND_AUDIO_SOURCE_DRAWER_ID,
        SOUND_AUDIO_LISTENER_DRAWER_ID,
        SOUND_AUDIO_VOLUME_DRAWER_ID,
    ]
}
