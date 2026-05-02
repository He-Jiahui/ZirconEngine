use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};
use zircon_runtime::core::framework::sound::{
    AUDIO_LISTENER_COMPONENT_TYPE, AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};

pub const PLUGIN_ID: &str = zircon_plugin_sound_runtime::PLUGIN_ID;
pub const SOUND_AUTHORING_VIEW_ID: &str = "sound.mixer_console";
pub const SOUND_ACOUSTIC_DEBUG_VIEW_ID: &str = "sound.acoustic_debug";
pub const SOUND_DRAWER_ID: &str = "sound.drawer";
pub const SOUND_TEMPLATE_ID: &str = "sound.mixer_console";
pub const SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID: &str = "sound.acoustic_debug";
pub const SOUND_AUDIO_SOURCE_DRAWER_ID: &str = "sound.Component.AudioSource.drawer";
pub const SOUND_AUDIO_LISTENER_DRAWER_ID: &str = "sound.Component.AudioListener.drawer";
pub const SOUND_AUDIO_VOLUME_DRAWER_ID: &str = "sound.Component.AudioVolume.drawer";

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
                template_document: "plugins://sound/editor/mixer_console.ui.toml",
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
        register_sound_component_drawers(registry)
    }
}

fn register_sound_component_drawers(
    registry: &mut zircon_editor::core::editor_extension::EditorExtensionRegistry,
) -> Result<(), zircon_editor::core::editor_extension::EditorExtensionRegistryError> {
    use zircon_editor::core::editor_extension::{
        ComponentDrawerDescriptor, EditorUiTemplateDescriptor,
    };

    registry.register_ui_template(EditorUiTemplateDescriptor::new(
        SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID,
        "plugins://sound/editor/acoustic_debug.ui.toml",
    ))?;
    registry.register_component_drawer(ComponentDrawerDescriptor::new(
        AUDIO_SOURCE_COMPONENT_TYPE,
        "plugins://sound/editor/audio_source.drawer.ui.toml",
        SOUND_AUDIO_SOURCE_DRAWER_ID,
    ))?;
    registry.register_component_drawer(ComponentDrawerDescriptor::new(
        AUDIO_LISTENER_COMPONENT_TYPE,
        "plugins://sound/editor/audio_listener.drawer.ui.toml",
        SOUND_AUDIO_LISTENER_DRAWER_ID,
    ))?;
    registry.register_component_drawer(ComponentDrawerDescriptor::new(
        AUDIO_VOLUME_COMPONENT_TYPE,
        "plugins://sound/editor/audio_volume.drawer.ui.toml",
        SOUND_AUDIO_VOLUME_DRAWER_ID,
    ))
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(PLUGIN_ID, "Sound", "zircon_plugin_sound_editor")
        .with_capability("editor.extension.sound_authoring")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_editor_plugin_contributes_authoring_extensions() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&"editor.extension.sound_authoring".to_string()));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == SOUND_AUTHORING_VIEW_ID));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == SOUND_ACOUSTIC_DEBUG_VIEW_ID));
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == SOUND_DRAWER_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == SOUND_TEMPLATE_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID));
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| {
                drawer.component_type() == AUDIO_SOURCE_COMPONENT_TYPE
                    && drawer.controller() == SOUND_AUDIO_SOURCE_DRAWER_ID
            }));
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| {
                drawer.component_type() == AUDIO_LISTENER_COMPONENT_TYPE
                    && drawer.controller() == SOUND_AUDIO_LISTENER_DRAWER_ID
            }));
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| {
                drawer.component_type() == AUDIO_VOLUME_COMPONENT_TYPE
                    && drawer.controller() == SOUND_AUDIO_VOLUME_DRAWER_ID
            }));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.sound.mixer_console.Open"));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.sound.acoustic_debug.Open"));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "View.sound.mixer_console.Open"));
    }
}
