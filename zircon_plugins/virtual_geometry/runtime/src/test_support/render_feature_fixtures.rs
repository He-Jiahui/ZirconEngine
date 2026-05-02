use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::graphics::{RenderFeatureDescriptor, WgpuRenderFramework};

pub(crate) fn virtual_geometry_render_feature_descriptor() -> RenderFeatureDescriptor {
    crate::render_feature_descriptor()
}

#[allow(dead_code)]
pub(crate) fn pluginized_wgpu_render_framework_with_asset_manager(
    asset_manager: Arc<ProjectAssetManager>,
) -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_features(
        asset_manager,
        [virtual_geometry_render_feature_descriptor()],
        crate::render_pass_executor_registrations(),
        [crate::virtual_geometry_runtime_provider_registration()],
    )
    .unwrap()
}

#[cfg(test)]
mod tests {
    use zircon_runtime::graphics::RenderFeatureCapabilityRequirement;

    use super::*;

    #[test]
    fn render_feature_fixture_uses_plugin_virtual_geometry_descriptor() {
        let descriptor = virtual_geometry_render_feature_descriptor();

        assert_eq!(descriptor.name, crate::VIRTUAL_GEOMETRY_FEATURE_NAME);
        assert_eq!(
            descriptor.capability_requirements,
            vec![RenderFeatureCapabilityRequirement::VirtualGeometry]
        );
    }
}
