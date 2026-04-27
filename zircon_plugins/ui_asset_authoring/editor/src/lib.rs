use zircon_plugin_editor_support::{
    register_authoring_extensions, EditorAuthoringExtensions, EditorAuthoringSurface,
};

pub const PLUGIN_ID: &str = "ui_asset_authoring";
pub const UI_ASSET_VIEW_ID: &str = "editor.ui_asset";
pub const UI_ASSET_DRAWER_ID: &str = "ui_asset_authoring.drawer";
pub const UI_ASSET_TEMPLATE_ID: &str = "editor.template.ui_asset_authoring";

#[derive(Clone, Debug)]
pub struct UiAssetAuthoringEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl UiAssetAuthoringEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for UiAssetAuthoringEditorPlugin {
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
                drawer_id: UI_ASSET_DRAWER_ID,
                drawer_display_name: "UI Asset Tools",
                template_id: UI_ASSET_TEMPLATE_ID,
                template_document: "pane.ui_asset_authoring.body",
                surfaces: &[EditorAuthoringSurface::new(
                    UI_ASSET_VIEW_ID,
                    "UI Asset",
                    "Assets",
                    "Plugins/UI Asset",
                )],
            },
        )
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "UI Asset Authoring",
        "zircon_plugin_ui_asset_authoring_editor",
    )
    .with_capability(zircon_editor::EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING)
}

pub fn editor_plugin() -> UiAssetAuthoringEditorPlugin {
    UiAssetAuthoringEditorPlugin::new()
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
    zircon_runtime::PluginPackageManifest::new(PLUGIN_ID, "UI Asset Authoring")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_asset_authoring_plugin_contributes_view_template_and_capability() {
        let registration = plugin_registration();

        assert!(registration.is_success(), "{:?}", registration.diagnostics);
        assert!(registration
            .capabilities
            .contains(&zircon_editor::EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING.to_string()));
        assert!(registration
            .extensions
            .views()
            .iter()
            .any(|view| view.id() == UI_ASSET_VIEW_ID));
        assert!(registration
            .extensions
            .ui_templates()
            .iter()
            .any(|template| template.id() == UI_ASSET_TEMPLATE_ID));
        assert!(registration
            .extensions
            .drawers()
            .iter()
            .any(|drawer| drawer.id() == UI_ASSET_DRAWER_ID));
        assert!(registration
            .extensions
            .menu_items()
            .iter()
            .any(|menu| menu.operation().as_str() == "View.editor.ui_asset.Open"));
        assert!(registration
            .extensions
            .operations()
            .descriptors()
            .any(|operation| operation.path().as_str() == "View.editor.ui_asset.Open"));
    }
}
