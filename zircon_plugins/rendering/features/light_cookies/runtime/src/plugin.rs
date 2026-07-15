use crate::{render_feature_descriptor, render_pass_executor_registrations};

#[derive(Clone, Debug)]
pub struct RenderingLightCookiesRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for RenderingLightCookiesRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    fn register(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_render_feature(render_feature_descriptor())?;
        for registration in render_pass_executor_registrations() {
            registry.register_render_pass_executor(registration)?;
        }
        Ok(())
    }
}

pub fn runtime_plugin_feature() -> RenderingLightCookiesRuntimeFeature {
    RenderingLightCookiesRuntimeFeature
}

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_runtime::feature_manifest(
        zircon_plugin_rendering_runtime::RenderingFeatureKind::LightCookies,
    )
}
