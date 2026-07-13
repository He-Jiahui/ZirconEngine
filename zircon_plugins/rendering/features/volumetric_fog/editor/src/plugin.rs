use crate::{CAPABILITY, FEATURE_ID};

#[derive(Clone, Debug)]
pub struct RenderingVolumetricFogEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingVolumetricFogEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Volumetric Fog",
                "zircon_plugin_rendering_volumetric_fog_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl Default for RenderingVolumetricFogEditorFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_editor::EditorPlugin for RenderingVolumetricFogEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingVolumetricFogEditorFeature {
    RenderingVolumetricFogEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_volumetric_fog_runtime::feature_manifest()
}

