use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginModuleManifest,
};

use crate::capability::{DIST_CRATE_NAME, EDITOR_CAPABILITY, FEATURE_ID, RUNTIME_CAPABILITY};

#[derive(Clone, Debug)]
pub struct SoundRayTracedConvolutionRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for SoundRayTracedConvolutionRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(zircon_runtime::core::ModuleDescriptor::new(
            "SoundRayTracedConvolutionFeatureModule",
            "Sound ray-traced convolution reverb feature",
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

pub fn feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(FEATURE_ID, "Ray Traced Convolution Reverb", "sound")
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ])
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
        .with_capability(RUNTIME_CAPABILITY)
        .with_runtime_module(
            PluginModuleManifest::runtime(
                "sound.ray_traced_convolution_reverb.runtime",
                "zircon_plugin_sound_ray_traced_convolution_runtime",
            )
            .with_target_modes([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capabilities([RUNTIME_CAPABILITY.to_string()]),
        )
        .with_editor_module(
            PluginModuleManifest::editor(
                "sound.ray_traced_convolution_reverb.editor",
                "zircon_plugin_sound_ray_traced_convolution_editor",
            )
            .with_capabilities([EDITOR_CAPABILITY.to_string()]),
        )
        .with_native_module(sound_ray_traced_convolution_dist_module_manifest())
}

pub fn sound_ray_traced_convolution_dist_module_manifest() -> PluginModuleManifest {
    PluginModuleManifest::native("sound.ray_traced_convolution_reverb.dist", DIST_CRATE_NAME)
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([RUNTIME_CAPABILITY.to_string()])
}
