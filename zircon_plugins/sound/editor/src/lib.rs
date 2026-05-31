use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

mod authoring_bindings;
mod live_output;

pub use authoring_bindings::{
    register_sound_authoring_bindings, sound_editor_operation_descriptors,
    SOUND_AUDIO_LISTENER_OPERATION_PATHS, SOUND_AUDIO_SOURCE_OPERATION_PATHS,
    SOUND_AUDIO_VOLUME_OPERATION_PATHS, SOUND_MIXER_OPERATION_PATHS,
};
pub use live_output::{
    SoundEditorLiveOutputController, SoundEditorOutputAction, SoundEditorOutputActionReport,
    SoundEditorOutputDeviceRow, SoundEditorOutputSnapshot, SoundEditorOutputStatusModel,
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
    registry
        .register_component_drawer(authoring_bindings::sound_audio_source_drawer_descriptor())?;
    registry
        .register_component_drawer(authoring_bindings::sound_audio_listener_drawer_descriptor())?;
    registry.register_component_drawer(authoring_bindings::sound_audio_volume_drawer_descriptor())
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
    use std::collections::BTreeSet;
    use zircon_runtime::core::framework::sound::{
        AUDIO_LISTENER_COMPONENT_TYPE, AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
    };

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
                    && drawer
                        .bindings()
                        .iter()
                        .map(String::as_str)
                        .eq(SOUND_AUDIO_SOURCE_OPERATION_PATHS.iter().copied())
            }));
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| {
                drawer.component_type() == AUDIO_LISTENER_COMPONENT_TYPE
                    && drawer.controller() == SOUND_AUDIO_LISTENER_DRAWER_ID
                    && drawer
                        .bindings()
                        .iter()
                        .map(String::as_str)
                        .eq(SOUND_AUDIO_LISTENER_OPERATION_PATHS.iter().copied())
            }));
        assert!(registration
            .extensions
            .component_drawers()
            .iter()
            .any(|drawer| {
                drawer.component_type() == AUDIO_VOLUME_COMPONENT_TYPE
                    && drawer.controller() == SOUND_AUDIO_VOLUME_DRAWER_ID
                    && drawer
                        .bindings()
                        .iter()
                        .map(String::as_str)
                        .eq(SOUND_AUDIO_VOLUME_OPERATION_PATHS.iter().copied())
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
        for path in SOUND_MIXER_OPERATION_PATHS
            .iter()
            .chain(SOUND_AUDIO_SOURCE_OPERATION_PATHS)
            .chain(SOUND_AUDIO_LISTENER_OPERATION_PATHS)
            .chain(SOUND_AUDIO_VOLUME_OPERATION_PATHS)
        {
            let operation = registration
                .extensions
                .operations()
                .descriptors()
                .find(|operation| operation.path().as_str() == *path)
                .unwrap_or_else(|| panic!("missing sound editor operation {path}"));
            assert!(operation
                .payload_schema_id()
                .is_some_and(|schema| schema.starts_with("sound.")));
        }
        let create_track = registration
            .extensions
            .operations()
            .descriptors()
            .find(|operation| operation.path().as_str() == "Sound.Mixer.Track.Create")
            .expect("create track operation");
        assert!(create_track.undoable().is_some());
    }

    #[test]
    fn sound_editor_ui_template_routes_are_registered_operations() {
        let registration = plugin_registration();
        let registered_operations = registration
            .extensions
            .operations()
            .descriptors()
            .map(|operation| operation.path().as_str().to_string())
            .collect::<BTreeSet<_>>();
        let routes = ui_template_routes(&[
            (
                "mixer_console.v2.ui.toml",
                include_str!("../mixer_console.v2.ui.toml"),
            ),
            (
                "acoustic_debug.v2.ui.toml",
                include_str!("../acoustic_debug.v2.ui.toml"),
            ),
            (
                "audio_source.drawer.v2.ui.toml",
                include_str!("../audio_source.drawer.v2.ui.toml"),
            ),
            (
                "audio_listener.drawer.v2.ui.toml",
                include_str!("../audio_listener.drawer.v2.ui.toml"),
            ),
            (
                "audio_volume.drawer.v2.ui.toml",
                include_str!("../audio_volume.drawer.v2.ui.toml"),
            ),
        ]);

        assert!(
            !routes.is_empty(),
            "sound editor templates should expose at least one routed control"
        );
        for (template, route) in routes {
            assert!(
                registered_operations.contains(&route),
                "sound editor template {template} routes to unregistered operation {route}"
            );
        }
    }

    #[test]
    fn sound_editor_ui_template_asset_ids_match_registered_surfaces() {
        assert_template_asset_id(
            "mixer_console.v2.ui.toml",
            include_str!("../mixer_console.v2.ui.toml"),
            SOUND_TEMPLATE_ID,
        );
        assert_template_asset_id(
            "acoustic_debug.v2.ui.toml",
            include_str!("../acoustic_debug.v2.ui.toml"),
            SOUND_ACOUSTIC_DEBUG_TEMPLATE_ID,
        );
        assert_template_asset_id(
            "audio_source.drawer.v2.ui.toml",
            include_str!("../audio_source.drawer.v2.ui.toml"),
            "sound.audio_source.drawer",
        );
        assert_template_asset_id(
            "audio_listener.drawer.v2.ui.toml",
            include_str!("../audio_listener.drawer.v2.ui.toml"),
            "sound.audio_listener.drawer",
        );
        assert_template_asset_id(
            "audio_volume.drawer.v2.ui.toml",
            include_str!("../audio_volume.drawer.v2.ui.toml"),
            "sound.audio_volume.drawer",
        );
    }

    fn ui_template_routes(
        templates: &[(&'static str, &'static str)],
    ) -> Vec<(&'static str, String)> {
        let mut routes = Vec::new();
        for (template, source) in templates {
            let mut remaining = *source;
            while let Some(index) = remaining.find("route = \"") {
                let route_start = index + "route = \"".len();
                let route_source = &remaining[route_start..];
                let route_end = route_source
                    .find('"')
                    .expect("sound editor template route should close string");
                routes.push((*template, route_source[..route_end].to_string()));
                remaining = &route_source[route_end..];
            }
        }
        routes
    }

    fn assert_template_asset_id(template: &str, source: &str, expected: &str) {
        assert_eq!(
            template_asset_id(source).as_deref(),
            Some(expected),
            "sound editor template {template} should keep its asset id aligned"
        );
    }

    fn template_asset_id(source: &str) -> Option<String> {
        source.lines().map(str::trim).find_map(|line| {
            line.strip_prefix("id = \"")
                .and_then(|value| value.strip_suffix('"'))
                .map(str::to_string)
        })
    }
}
