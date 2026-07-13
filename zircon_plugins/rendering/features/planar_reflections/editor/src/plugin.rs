use crate::{CAPABILITY, FEATURE_ID};

#[derive(Clone, Debug)]
pub struct RenderingPlanarReflectionsEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl RenderingPlanarReflectionsEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Planar Reflections",
                "zircon_plugin_rendering_planar_reflections_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl Default for RenderingPlanarReflectionsEditorFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl zircon_editor::EditorPlugin for RenderingPlanarReflectionsEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> RenderingPlanarReflectionsEditorFeature {
    RenderingPlanarReflectionsEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_planar_reflections_runtime::feature_manifest()
}
