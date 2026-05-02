pub const FEATURE_ID: &str = zircon_plugin_rendering_shader_graph_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.rendering.shader_graph";
pub const SHADER_GRAPH_ASSET_VIEW_ID: &str = "rendering.shader_graph.asset";

#[derive(Clone, Debug)]
pub struct RenderingShaderGraphEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingShaderGraphEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Shader Graph",
                "zircon_plugin_rendering_shader_graph_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingShaderGraphEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingShaderGraphEditorFeature {
    RenderingShaderGraphEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_shader_graph_runtime::feature_manifest()
}
