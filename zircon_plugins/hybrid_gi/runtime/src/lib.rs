use zircon_runtime::graphics::{
    FrameHistoryBinding, FrameHistorySlot, RenderFeatureCapabilityRequirement,
    RenderFeatureDescriptor, RenderFeaturePassDescriptor, RenderPassExecutorRegistration,
    RenderPassStage,
};
use zircon_runtime::render_graph::QueueLane;

mod hybrid_gi;
mod render_pass_executors;
#[cfg(test)]
pub(crate) mod test_support;

pub(crate) use hybrid_gi::{
    HybridGiGpuCompletion, HybridGiRuntimeFeedback, HybridGiRuntimeScenePrepareResources,
    HybridGiRuntimeState, HybridGiSceneInputs,
};
use render_pass_executors::{
    hybrid_gi_history_executor, hybrid_gi_resolve_executor, hybrid_gi_scene_prepare_executor,
    hybrid_gi_trace_schedule_executor,
};

pub const PLUGIN_ID: &str = "hybrid_gi";
pub const HYBRID_GI_FEATURE_NAME: &str = "hybrid_gi";
pub const HYBRID_GI_MODULE_NAME: &str = "HybridGiPluginModule";

#[derive(Clone, Debug)]
pub struct HybridGiRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl HybridGiRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for HybridGiRuntimePlugin {
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
        Ok(())
    }
}

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        HYBRID_GI_MODULE_NAME,
        "Hybrid global illumination render feature plugin",
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(
        HYBRID_GI_FEATURE_NAME,
        vec![
            "view".to_string(),
            "lighting".to_string(),
            "visibility".to_string(),
        ],
        vec![FrameHistoryBinding::read_write(
            FrameHistorySlot::GlobalIllumination,
        )],
        vec![
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-scene-prepare",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.scene-prepare")
            .read_texture("scene-depth")
            .write_buffer("hybrid-gi-scene"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-trace-schedule",
                QueueLane::AsyncCompute,
            )
            .with_executor_id("hybrid-gi.trace-schedule")
            .read_buffer("hybrid-gi-scene")
            .write_buffer("hybrid-gi-trace"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::Lighting,
                "hybrid-gi-resolve",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.resolve")
            .read_buffer("hybrid-gi-trace")
            .write_texture("hybrid-gi-lighting"),
            RenderFeaturePassDescriptor::new(
                RenderPassStage::PostProcess,
                "hybrid-gi-history",
                QueueLane::Graphics,
            )
            .with_executor_id("hybrid-gi.history")
            .read_texture("scene-color")
            .write_external("history-global-illumination"),
        ],
    )
    .with_capability_requirement(RenderFeatureCapabilityRequirement::HybridGlobalIllumination)
}

pub fn render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new(
            "hybrid-gi.scene-prepare",
            hybrid_gi_scene_prepare_executor,
        ),
        RenderPassExecutorRegistration::new(
            "hybrid-gi.trace-schedule",
            hybrid_gi_trace_schedule_executor,
        ),
        RenderPassExecutorRegistration::new("hybrid-gi.resolve", hybrid_gi_resolve_executor),
        RenderPassExecutorRegistration::new("hybrid-gi.history", hybrid_gi_history_executor),
    ]
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Hybrid GI",
        zircon_runtime::RuntimePluginId::HybridGi,
        "zircon_plugin_hybrid_gi_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.hybrid_gi")
}

pub fn runtime_plugin() -> HybridGiRuntimePlugin {
    HybridGiRuntimePlugin::new()
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
    &["runtime.plugin.hybrid_gi"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_gi_registration_contributes_render_feature_descriptor() {
        let report = plugin_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(report
            .extensions
            .modules()
            .iter()
            .any(|module| module.name == HYBRID_GI_MODULE_NAME));
        assert_eq!(
            report.extensions.render_features()[0].name,
            HYBRID_GI_FEATURE_NAME
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
                "lighting".to_string(),
                "visibility".to_string()
            ]
        );
        assert_eq!(
            feature.capability_requirements,
            vec![
                zircon_runtime::graphics::RenderFeatureCapabilityRequirement::HybridGlobalIllumination
            ]
        );
        assert_eq!(
            feature.history_bindings,
            vec![zircon_runtime::graphics::FrameHistoryBinding::read_write(
                zircon_runtime::graphics::FrameHistorySlot::GlobalIllumination
            )]
        );
        assert_eq!(
            feature
                .stage_passes
                .iter()
                .map(|pass| pass.pass_name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "hybrid-gi-scene-prepare",
                "hybrid-gi-trace-schedule",
                "hybrid-gi-resolve",
                "hybrid-gi-history",
            ]
        );
        assert_eq!(report.extensions.render_pass_executors().len(), 4);
        assert_eq!(
            report
                .extensions
                .render_pass_executors()
                .iter()
                .map(|registration| registration.executor_id().as_str())
                .collect::<Vec<_>>(),
            vec![
                "hybrid-gi.scene-prepare",
                "hybrid-gi.trace-schedule",
                "hybrid-gi.resolve",
                "hybrid-gi.history",
            ]
        );
    }
}
