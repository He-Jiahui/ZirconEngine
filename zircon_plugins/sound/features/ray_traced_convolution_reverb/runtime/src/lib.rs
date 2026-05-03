pub const FEATURE_ID: &str = "sound.ray_traced_convolution_reverb";

#[derive(Clone, Debug)]
pub struct SoundRayTracedConvolutionRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for SoundRayTracedConvolutionRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(zircon_runtime::core::ModuleDescriptor::new(
            "SoundRayTracedConvolutionFeatureModule",
            "Sound ray-traced convolution reverb feature placeholder",
        ))
    }
}

pub fn runtime_plugin_feature() -> SoundRayTracedConvolutionRuntimeFeature {
    SoundRayTracedConvolutionRuntimeFeature
}

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        FEATURE_ID,
        "Sound Ray-Traced Convolution Reverb",
        "sound",
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        "sound",
        "runtime.plugin.sound",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.plugin.physics",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.capability.physics.raycast",
    ))
    .with_capability("runtime.feature.sound.ray_traced_convolution_reverb")
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "sound.ray_traced_convolution_reverb.runtime",
            "zircon_plugin_sound_ray_traced_convolution_runtime",
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.ray_traced_convolution_reverb".to_string()]),
    )
    .with_editor_module(zircon_runtime::plugin::PluginModuleManifest::editor(
        "sound.ray_traced_convolution_reverb.editor",
        "zircon_plugin_sound_ray_traced_convolution_editor",
    ))
}
