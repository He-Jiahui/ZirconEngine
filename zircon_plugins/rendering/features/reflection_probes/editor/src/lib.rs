pub const FEATURE_ID: &str = zircon_plugin_rendering_reflection_probes_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.rendering.reflection_probes";

#[derive(Clone, Debug)]
pub struct RenderingReflectionProbesEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingReflectionProbesEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Reflection Probes",
                "zircon_plugin_rendering_reflection_probes_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for RenderingReflectionProbesEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingReflectionProbesEditorFeature {
    RenderingReflectionProbesEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_reflection_probes_runtime::feature_manifest()
}
