use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

pub const PLUGIN_ID: &str = zircon_plugin_animation_runtime::PLUGIN_ID;
pub const ANIMATION_SEQUENCE_VIEW_ID: &str = "editor.animation_sequence";
pub const ANIMATION_GRAPH_VIEW_ID: &str = "editor.animation_graph";
pub const ANIMATION_DRAWER_ID: &str = "animation.drawer";
pub const ANIMATION_TEMPLATE_ID: &str = "animation.authoring";

#[derive(Clone, Debug)]
pub struct AnimationEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl AnimationEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for AnimationEditorPlugin {
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
                drawer_id: ANIMATION_DRAWER_ID,
                drawer_display_name: "Animation Tools",
                template_id: ANIMATION_TEMPLATE_ID,
                template_document: "plugins://animation/editor/authoring.ui.toml",
                surfaces: &[
                    EditorAuthoringSurface::new(
                        ANIMATION_SEQUENCE_VIEW_ID,
                        "Animation Sequence",
                        "Animation",
                        "Plugins/Animation/Sequence",
                    ),
                    EditorAuthoringSurface::new(
                        ANIMATION_GRAPH_VIEW_ID,
                        "Animation Graph",
                        "Animation",
                        "Plugins/Animation/Graph",
                    ),
                ],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Animation",
        "zircon_plugin_animation_editor",
    )
    .with_capability(zircon_editor::EDITOR_SUBSYSTEM_ANIMATION_AUTHORING)
}

pub fn editor_plugin() -> AnimationEditorPlugin {
    AnimationEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_animation_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        zircon_plugin_animation_runtime::package_manifest(),
    )
}

pub fn editor_host_contract_marker() -> &'static str {
    zircon_editor::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_editor_plugin_contributes_sequence_and_graph_extensions() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&zircon_editor::EDITOR_SUBSYSTEM_ANIMATION_AUTHORING.to_string()));
        for view_id in [ANIMATION_SEQUENCE_VIEW_ID, ANIMATION_GRAPH_VIEW_ID] {
            assert!(registration
                .extensions
                .views()
                .iter()
                .any(|view| view.id() == view_id));
            let operation_path = format!("View.{view_id}.Open");
            assert!(registration
                .extensions
                .menu_items()
                .iter()
                .any(|menu| menu.operation().as_str() == operation_path));
            assert!(registration
                .extensions
                .operations()
                .descriptors()
                .any(|operation| operation.path().as_str() == operation_path));
        }
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == ANIMATION_DRAWER_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == ANIMATION_TEMPLATE_ID));
    }
}
