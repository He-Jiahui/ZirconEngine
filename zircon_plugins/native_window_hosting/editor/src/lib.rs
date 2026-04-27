use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

pub const PLUGIN_ID: &str = "native_window_hosting";
pub const WORKBENCH_WINDOW_VIEW_ID: &str = "editor.workbench_window";
pub const PREFAB_WINDOW_VIEW_ID: &str = "editor.prefab";
pub const NATIVE_WINDOW_DRAWER_ID: &str = "native_window_hosting.drawer";
pub const NATIVE_WINDOW_TEMPLATE_ID: &str = "native_window_hosting.authoring";

#[derive(Clone, Debug)]
pub struct NativeWindowHostingEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl NativeWindowHostingEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for NativeWindowHostingEditorPlugin {
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
                drawer_id: NATIVE_WINDOW_DRAWER_ID,
                drawer_display_name: "Native Window Tools",
                template_id: NATIVE_WINDOW_TEMPLATE_ID,
                template_document: "pane.native_window_hosting.body",
                surfaces: &[
                    EditorAuthoringSurface::new(
                        WORKBENCH_WINDOW_VIEW_ID,
                        "Workbench",
                        "Window",
                        "Plugins/Native Windows/Workbench",
                    ),
                    EditorAuthoringSurface::new(
                        PREFAB_WINDOW_VIEW_ID,
                        "Prefab Editor",
                        "Window",
                        "Plugins/Native Windows/Prefab",
                    ),
                ],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Native Window Hosting",
        "zircon_plugin_native_window_hosting_editor",
    )
    .with_capability(zircon_editor::EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING)
}

pub fn editor_plugin() -> NativeWindowHostingEditorPlugin {
    NativeWindowHostingEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(&editor_plugin(), base_package_manifest())
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}

pub fn plugin_registration() -> zircon_editor::EditorPluginRegistrationReport {
    zircon_editor::EditorPluginRegistrationReport::from_plugin(
        &editor_plugin(),
        base_package_manifest(),
    )
}

fn base_package_manifest() -> zircon_runtime::PluginPackageManifest {
    zircon_runtime::PluginPackageManifest::new(PLUGIN_ID, "Native Window Hosting")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_window_hosting_plugin_contributes_window_views_and_capability() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&zircon_editor::EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING.to_string()));
        let views = registration.extensions.views();
        assert!(views
            .iter()
            .any(|view| view.id() == WORKBENCH_WINDOW_VIEW_ID));
        assert!(views.iter().any(|view| view.id() == PREFAB_WINDOW_VIEW_ID));
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == NATIVE_WINDOW_DRAWER_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == NATIVE_WINDOW_TEMPLATE_ID));
        for operation_path in [
            "View.editor.workbench_window.Open",
            "View.editor.prefab.Open",
        ] {
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
    }
}
