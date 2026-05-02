pub const FEATURE_ID: &str = zircon_plugin_rendering_baked_lighting_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.rendering.baked_lighting";

#[derive(Clone, Debug)]
pub struct RenderingBakedLightingEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingBakedLightingEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Baked Lighting",
                "zircon_plugin_rendering_baked_lighting_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingBakedLightingEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingBakedLightingEditorFeature {
    RenderingBakedLightingEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_baked_lighting_runtime::feature_manifest()
}
