use crate::{CAPABILITY, FEATURE_ID};

#[derive(Clone, Debug)]
pub struct RenderingIrradianceVolumesEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingIrradianceVolumesEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Irradiance Volumes",
                "zircon_plugin_rendering_irradiance_volumes_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl Default for RenderingIrradianceVolumesEditorFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_editor::EditorPlugin for RenderingIrradianceVolumesEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingIrradianceVolumesEditorFeature {
    RenderingIrradianceVolumesEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_irradiance_volumes_runtime::feature_manifest()
}
