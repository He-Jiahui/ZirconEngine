use crate::{CAPABILITY, FEATURE_ID};

#[derive(Clone, Debug)]
pub struct RenderingSubsurfaceScatteringEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingSubsurfaceScatteringEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Subsurface Scattering",
                "zircon_plugin_rendering_subsurface_scattering_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl Default for RenderingSubsurfaceScatteringEditorFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_editor::EditorPlugin for RenderingSubsurfaceScatteringEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingSubsurfaceScatteringEditorFeature {
    RenderingSubsurfaceScatteringEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_subsurface_scattering_runtime::feature_manifest()
}
