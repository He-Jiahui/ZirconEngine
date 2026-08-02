use std::{ops::Deref, sync::Arc};

use zircon_runtime::asset::pipeline::manager::{ProjectAssetManager, ProjectAssetManagerAccess};
use zircon_runtime::core::manager::{manager_service_handle, RegisteredManagerService};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::core::{
    CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
};
use zircon_runtime::graphics::{RenderFeatureDescriptor, WgpuRenderFramework};

const TEST_ASSET_MODULE_NAME: &str = "HybridGiTestAssetRuntime";
const TEST_ASSET_SERVICE_NAME: &str = "HybridGiTestAssetRuntime.Manager.ProjectAssetManager";

pub(crate) struct PluginizedWgpuRenderFrameworkFixture {
    framework: WgpuRenderFramework,
    _asset_runtime: CoreRuntime,
}

impl Deref for PluginizedWgpuRenderFrameworkFixture {
    type Target = WgpuRenderFramework;

    fn deref(&self) -> &Self::Target {
        &self.framework
    }
}

pub(crate) fn hybrid_gi_render_feature_descriptor() -> RenderFeatureDescriptor {
    crate::render_feature_descriptor()
}

pub(crate) fn pluginized_wgpu_render_framework_with_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> PluginizedWgpuRenderFrameworkFixture {
    let asset_runtime = CoreRuntime::new();
    asset_runtime
        .register_module(
            ModuleDescriptor::new(TEST_ASSET_MODULE_NAME, "hybrid GI test asset runtime")
                .with_manager(ManagerDescriptor::new(
                    RegistryName::from_parts(
                        TEST_ASSET_MODULE_NAME,
                        ServiceKind::Manager,
                        "ProjectAssetManager",
                    ),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(
                            Arc::new(RegisteredManagerService::new(Arc::clone(&asset_manager)))
                                as ServiceObject,
                        )
                    }),
                )),
        )
        .expect("hybrid GI test ProjectAssetManager service should register");
    asset_runtime
        .activate_module(TEST_ASSET_MODULE_NAME)
        .expect("hybrid GI test ProjectAssetManager module should activate");
    let core = asset_runtime.handle();
    let handle = manager_service_handle(&core, TEST_ASSET_SERVICE_NAME)
        .expect("hybrid GI test ProjectAssetManager handle should resolve");
    let framework = WgpuRenderFramework::new_with_plugin_render_extensions(
        ProjectAssetManagerAccess::new(core, handle),
        [hybrid_gi_render_feature_descriptor()],
        crate::render_pass_executor_registrations(),
        [crate::runtime_prepare_collector_registration()],
        [crate::hybrid_gi_runtime_provider_registration()],
        Vec::new(),
    )
    .unwrap();
    PluginizedWgpuRenderFrameworkFixture {
        framework,
        _asset_runtime: asset_runtime,
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime::graphics::RenderFeatureCapabilityRequirement;

    use super::*;

    #[test]
    fn render_feature_fixture_uses_plugin_hybrid_gi_descriptor() {
        let descriptor = hybrid_gi_render_feature_descriptor();

        assert_eq!(descriptor.name, crate::HYBRID_GI_FEATURE_NAME);
        assert_eq!(
            descriptor.capability_requirements,
            vec![RenderFeatureCapabilityRequirement::HybridGlobalIllumination]
        );
    }
}
