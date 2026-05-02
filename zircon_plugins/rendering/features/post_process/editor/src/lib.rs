pub const FEATURE_ID: &str = zircon_plugin_rendering_post_process_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.rendering.post_process";

#[derive(Clone, Debug)]
pub struct RenderingPostProcessEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingPostProcessEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Post Process",
                "zircon_plugin_rendering_post_process_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingPostProcessEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingPostProcessEditorFeature {
    RenderingPostProcessEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_post_process_runtime::feature_manifest()
}
