pub const FEATURE_ID: &str = zircon_plugin_rendering_vfx_graph_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.rendering.vfx_graph";
pub const VFX_GRAPH_ASSET_VIEW_ID: &str = "rendering.vfx_graph.asset";

#[derive(Clone, Debug)]
pub struct RenderingVfxGraphEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingVfxGraphEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "VFX Graph",
                "zircon_plugin_rendering_vfx_graph_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingVfxGraphEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingVfxGraphEditorFeature {
    RenderingVfxGraphEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_vfx_graph_runtime::feature_manifest()
}
