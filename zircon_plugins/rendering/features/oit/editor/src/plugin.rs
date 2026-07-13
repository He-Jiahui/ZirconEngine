use crate::{CAPABILITY, FEATURE_ID};

#[derive(Clone, Debug)]
pub struct RenderingOitEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingOitEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Order Independent Transparency",
                "zircon_plugin_rendering_oit_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl Default for RenderingOitEditorFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_editor::EditorPlugin for RenderingOitEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingOitEditorFeature {
    RenderingOitEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_oit_runtime::feature_manifest()
}
