pub const FEATURE_ID: &str = zircon_plugin_sound_ray_traced_convolution_runtime::FEATURE_ID;
pub const CAPABILITY: &str = "editor.feature.sound.ray_traced_convolution_reverb";

#[derive(Clone, Debug)]
pub struct SoundRayTracedConvolutionEditorFeature {
    descriptor: zircon_editor::EditorPluginDescriptor,
}

impl SoundRayTracedConvolutionEditorFeature {
    pub fn new() -> Self {
        Self {
            descriptor: zircon_editor::EditorPluginDescriptor::new(
                FEATURE_ID,
                "Sound Ray-Traced Convolution Reverb",
                "zircon_plugin_sound_ray_traced_convolution_editor",
            )
            .with_capability(CAPABILITY),
        }
    }
}

impl zircon_editor::EditorPlugin for SoundRayTracedConvolutionEditorFeature {
    fn descriptor(&self) -> &zircon_editor::EditorPluginDescriptor {
        &self.descriptor
    }
}

pub fn editor_feature() -> SoundRayTracedConvolutionEditorFeature {
    SoundRayTracedConvolutionEditorFeature::new()
}

pub fn editor_capabilities() -> Vec<String> {
    zircon_editor::EditorPlugin::editor_capabilities(&editor_feature()).to_vec()
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_sound_ray_traced_convolution_runtime::feature_manifest()
}
