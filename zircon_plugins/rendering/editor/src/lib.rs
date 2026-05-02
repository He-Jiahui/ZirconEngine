pub const PLUGIN_ID: &str = zircon_plugin_rendering_runtime::PLUGIN_ID;
pub const CAPABILITY: &str = "editor.extension.rendering_authoring";

#[derive(Clone, Debug)]
pub struct RenderingEditorPlugin {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingEditorPlugin {
    pub fn new() -> Self {
        Self {
            descriptor: editor_plugin_descriptor(),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingEditorPlugin {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_plugin_descriptor() -> zircon_editor::EditorPluginDescriptor {
    zircon_editor::EditorPluginDescriptor::new(
        PLUGIN_ID,
        "Rendering",
        "zircon_plugin_rendering_editor",
    )
    .with_capability(CAPABILITY)
}

pub fn editor_plugin() -> RenderingEditorPlugin {
    RenderingEditorPlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_editor::EditorPlugin::package_manifest(
        &editor_plugin(),
        zircon_plugin_rendering_runtime::package_manifest(),
    )
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_plugin()).to_vec()
}
