pub const FEATURE_ID: &str = zircon_plugin_rendering_ssao_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.rendering.ssao";

#[derive(Clone, Debug)]
pub struct RenderingSsaoEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingSsaoEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "SSAO",
                "zircon_plugin_rendering_ssao_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingSsaoEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingSsaoEditorFeature {
    RenderingSsaoEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_ssao_runtime::feature_manifest()
}
