use zircon_runtime::graphics::{
    RenderFeatureCapabilityRequirement, RenderFeatureDescriptor, RenderFeaturePassDescriptor,
    RenderPassExecutorRegistration, RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

mod provider;
mod render_pass_executors;
#[cfg(test)]
pub(crate) mod test_support;
mod virtual_geometry;

use render_pass_executors::{
    virtual_geometry_debug_overlay_executor, virtual_geometry_node_cluster_cull_executor,
    virtual_geometry_page_feedback_executor, virtual_geometry_prepare_executor,
    virtual_geometry_visbuffer_executor,
};
use std::sync::Arc;

pub use provider::PluginVirtualGeometryRuntimeProvider;

pub const PLUGIN_ID: &str = "virtual_geometry";
pub const VIRTUAL_GEOMETRY_FEATURE_NAME: &str = "virtual_geometry";
pub const VIRTUAL_GEOMETRY_MODULE_NAME: &str = "VirtualGeometryPluginModule";

#[derive(Clone, Debug)]
pub struct VirtualGeometryRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl VirtualGeometryRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for VirtualGeometryRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())?;
        registry.register_render_feature(render_feature_descriptor())?;
        for registration in render_pass_executor_registrations() {
            registry.register_render_pass_executor(registration)?;
        }
        registry.register_virtual_geometry_runtime_provider(
            virtual_geometry_runtime_provider_registration(),
        )
    }
}

pub fn virtual_geometry_runtime_provider_registration(
) -> zircon_runtime::graphics::VirtualGeometryRuntimeProviderRegistration {
    zircon_runtime::graphics::VirtualGeometryRuntimeProviderRegistration::new(
        PLUGIN_ID,
        Arc::new(PluginVirtualGeometryRuntimeProvider),
    )
}

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        VIRTUAL_GEOMETRY_MODULE_NAME,
        "Virtual geometry render feature plugin",
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        VIRTUAL_GEOMETRY_FEATURE_NAME,
        vec![
            "view".to_string(),
            "geometry".to_string(),
            "visibility".to_string(),
        ],
        Vec::new(),
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.prepare")
            .write_buffer("virtual-geometry-page-requests"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-node-cluster-cull",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("virtual-geometry.node-cluster-cull")
            .read_buffer("virtual-geometry-page-requests")
            .write_buffer("virtual-geometry-visible-clusters"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-page-feedback",
                QueueLane::AsyncCopy,
            )
            .with_executor_id("virtual-geometry.page-feedback")
            .read_buffer("virtual-geometry-visible-clusters")
            .write_external("virtual-geometry-feedback"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::DepthPrepass,
                "virtual-geometry-visbuffer",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.visbuffer")
            .read_buffer("virtual-geometry-visible-clusters")
            .write_texture("scene-depth"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Overlay,
                "virtual-geometry-debug-overlay",
                QueueLane::Graphics,
            )
            .with_executor_id("virtual-geometry.debug-overlay")
            .read_buffer("virtual-geometry-visible-clusters")
            .read_texture("scene-color")
            .write_texture("scene-color"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::VirtualGeometry)
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new(
            "virtual-geometry.prepare",
            virtual_geometry_prepare_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.node-cluster-cull",
            virtual_geometry_node_cluster_cull_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.page-feedback",
            virtual_geometry_page_feedback_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.visbuffer",
            virtual_geometry_visbuffer_executor,
        ),
        RenderPassExecutorRegistration::new(
            "virtual-geometry.debug-overlay",
            virtual_geometry_debug_overlay_executor,
        ),
    ]
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Virtual Geometry",
        zircon_runtime::RuntimePluginId::VirtualGeometry,
        "zircon_plugin_virtual_geometry_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.virtual_geometry")
}

pub fn runtime_plugin() -> VirtualGeometryRuntimePlugin {
    VirtualGeometryRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &["runtime.plugin.virtual_geometry"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_geometry_registration_contributes_render_feature_descriptor() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == VIRTUAL_GEOMETRY_MODULE_NAME));
        assert_eq!(
            report.extensions.render_features()[0].name,
            VIRTUAL_GEOMETRY_FEATURE_NAME
        );
        assert_eq!(
            report.extensions.virtual_geometry_runtime_providers()[0].provider_id(),
            PLUGIN_ID
        );
        assert_eq!(
            report.package_manifest.modules[0].target_modes,
            vec![
                zircon_runtime::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::RuntimeTargetMode::EditorHost,
            ]
        );
        let feature = &report.extensions.render_features()[0];
        assert_eq!(
            feature.required_extract_sections,
            vec![
                "view".to_string(),
                "geometry".to_string(),
                "visibility".to_string()
            ]
        );
        assert_eq!(
            feature.capability_requirements,
            vec![zircon_runtime::graphics::RenderFeatureCapabilityRequirement::VirtualGeometry]
        );
        assert_eq!(
            feature
                .stage_passes
                .iter()
                .map(|pass| pass.pass_name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "virtual-geometry-prepare",
                "virtual-geometry-node-cluster-cull",
                "virtual-geometry-page-feedback",
                "virtual-geometry-visbuffer",
                "virtual-geometry-debug-overlay",
            ]
        );
        assert_eq!(report.extensions.render_pass_executors().len(), 5);
        assert_eq!(
            report
                .extensions
                .render_pass_executors()
                .iter()
                .map(|registration| registration.executor_id().as_str())
                .collect::<Vec<_>>(),
            vec![
                "virtual-geometry.prepare",
                "virtual-geometry.node-cluster-cull",
                "virtual-geometry.page-feedback",
                "virtual-geometry.visbuffer",
                "virtual-geometry.debug-overlay",
            ]
        );
    }
}
