pub const FEATURE_ID: &str = zircon_plugin_sound_ray_traced_convolution_runtime::FEATURE_ID;
pub const CAPABILITY: &str = zircon_plugin_sound_ray_traced_convolution_runtime::EDITOR_CAPABILITY;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_traced_editor_feature_descriptor_matches_runtime_feature_contract() {
        let feature = editor_feature();
        let descriptor = zircon_editor::EditorPlugin::descriptor(&feature);

        assert_eq!(descriptor.package_id, FEATURE_ID);
        assert_eq!(
            descriptor.display_name,
            "Sound Ray-Traced Convolution Reverb"
        );
        assert_eq!(
            descriptor.crate_name,
            "zircon_plugin_sound_ray_traced_convolution_editor"
        );
        assert_eq!(descriptor.capabilities, vec![CAPABILITY.to_string()]);
        assert_eq!(editor_capabilities(), vec![CAPABILITY.to_string()]);
    }

    #[test]
    fn ray_traced_editor_feature_manifest_matches_runtime_provider_manifest() {
        assert_eq!(
            feature_manifest(),
            zircon_plugin_sound_ray_traced_convolution_runtime::feature_manifest()
        );
    }
}
