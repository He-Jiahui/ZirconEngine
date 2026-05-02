pub const FEATURE_ID: &str = zircon_plugin_rendering_decals_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.rendering.decals";
pub const DECAL_PROJECTOR_DRAWER_ID: &str = "rendering.Component.DecalProjector.drawer";

#[derive(Clone, Debug)]
pub struct RenderingDecalsEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingDecalsEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Decals",
                "zircon_plugin_rendering_decals_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingDecalsEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingDecalsEditorFeature {
    RenderingDecalsEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_decals_runtime::feature_manifest()
}
