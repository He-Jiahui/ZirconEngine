use std::cell::{Cell, RefCell};

use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::core::ModuleDescriptor;
use crate::plugin::{
    CapabilityStatus, CapabilityStatusManifest, CapabilityView, PluginFeatureBundleManifest,
    PluginFeatureDependency, PluginFinishContext, PluginModuleManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginCatalog,
    RuntimePluginDescriptor, RuntimePluginFeature, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

#[test]
fn runtime_plugin_lifecycle_hard_cuts_to_register_hook() {
    let plugin_trait = include_str!("../../plugin/runtime_plugin/runtime_plugin/plugin.rs");
    let feature_trait = include_str!("../../plugin/runtime_plugin/runtime_plugin/feature.rs");
    let plugin_report = include_str!("../../plugin/runtime_plugin/registration_report/plugin.rs");
    let feature_report =
        include_str!("../../plugin/runtime_plugin/feature_registration_report/feature.rs");

    for source in [plugin_trait, feature_trait, plugin_report, feature_report] {
        assert!(!source.contains("register_runtime_extensions"));
    }
    assert!(plugin_trait.contains("fn register("));
    assert!(feature_trait.contains("fn register("));
    assert!(plugin_report.contains("plugin.register(&mut extensions)"));
    assert!(feature_report.contains("feature.register(&mut extensions)"));
}

#[test]
fn optional_dependency_probe_sees_all_registered_capabilities() {
    let physics_registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("physics", "Physics")
            .with_capability("runtime.capability.physics.raycast")
            .with_capability_status(CapabilityStatusManifest::new(
                "runtime.capability.physics.raycast",
                CapabilityStatus::Complete,
            ))
            .with_runtime_module(
                PluginModuleManifest::runtime("physics.runtime", "zircon_plugin_physics_runtime")
                    .with_capabilities(["runtime.capability.physics.collider_world"]),
            ),
    );
    let sound_feature_registration =
        RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
            PluginFeatureBundleManifest::new("sound.occlusion", "Sound Occlusion", "sound")
                .with_capability("runtime.capability.sound.occlusion")
                .with_runtime_module(
                    PluginModuleManifest::runtime(
                        "sound.occlusion.runtime",
                        "zircon_plugin_sound_occlusion_runtime",
                    )
                    .with_capabilities(["runtime.capability.sound.occlusion.debug"]),
                ),
            Some("sound".to_string()),
        );
    let capability_view = CapabilityView::from_registration_reports(
        [&physics_registration],
        [&sound_feature_registration],
    );

    assert!(capability_view.has("runtime.capability.physics.raycast"));
    assert!(capability_view.has("runtime.capability.physics.collider_world"));
    assert!(capability_view.has("runtime.capability.sound.occlusion"));
    assert!(capability_view.has("runtime.capability.sound.occlusion.debug"));
    assert_eq!(
        capability_view.status("runtime.capability.physics.raycast"),
        Some(CapabilityStatus::Complete)
    );
    assert_eq!(
        capability_view.status("runtime.capability.sound.occlusion"),
        None
    );

    let probe = OptionalDependencyProbe::default();
    let mut registry = RuntimeExtensionRegistry::default();
    let mut context = PluginFinishContext::new(&mut registry, &capability_view);

    probe.finish(&mut context).unwrap();

    assert_eq!(
        probe.result.get(),
        Some(OptionalDependencyProbeResult {
            physics_raycast_available: true,
            physics_status: Some(CapabilityStatus::Complete),
            sound_occlusion_available: true,
        })
    );
}

#[test]
fn feature_register_runs_before_finish() {
    let log = RefCell::new(Vec::new());
    let plugin = LifecycleOrderPlugin::new(&log);
    let feature = LifecycleOrderFeature::new(&log);

    let catalog = RuntimePluginCatalog::from_lifecycle_plugins(
        [&plugin as &dyn RuntimePlugin],
        [&feature as &dyn RuntimePluginFeature],
    );

    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    assert_eq!(
        log.borrow().as_slice(),
        &[
            "plugin.register",
            "feature.register",
            "plugin.finish",
            "feature.finish",
        ]
    );
    assert!(catalog.registrations()[0]
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == "LifecycleFinishModule"));
    assert!(catalog.feature_registrations()[0]
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == "LifecycleFeatureFinishModule"));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OptionalDependencyProbeResult {
    physics_raycast_available: bool,
    physics_status: Option<CapabilityStatus>,
    sound_occlusion_available: bool,
}

struct OptionalDependencyProbe {
    descriptor: RuntimePluginDescriptor,
    result: Cell<Option<OptionalDependencyProbeResult>>,
}

impl Default for OptionalDependencyProbe {
    fn default() -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "sound",
                "Sound",
                RuntimePluginId::Sound,
                "zircon_plugin_sound_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.sound")
            .build(),
            result: Cell::new(None),
        }
    }
}

impl RuntimePlugin for OptionalDependencyProbe {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn finish(
        &self,
        context: &mut PluginFinishContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.result.set(Some(OptionalDependencyProbeResult {
            physics_raycast_available: context
                .capabilities
                .has("runtime.capability.physics.raycast"),
            physics_status: context
                .capabilities
                .status("runtime.capability.physics.raycast"),
            sound_occlusion_available: context
                .capabilities
                .has("runtime.capability.sound.occlusion"),
        }));
        Ok(())
    }
}

struct LifecycleOrderPlugin<'a> {
    descriptor: RuntimePluginDescriptor,
    log: &'a RefCell<Vec<&'static str>>,
}

impl<'a> LifecycleOrderPlugin<'a> {
    fn new(log: &'a RefCell<Vec<&'static str>>) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "sound",
                "Sound",
                RuntimePluginId::Sound,
                "zircon_plugin_sound_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.sound")
            .build(),
            log,
        }
    }
}

impl RuntimePlugin for LifecycleOrderPlugin<'_> {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("plugin.register");
        registry.register_module(ModuleDescriptor::new(
            "LifecycleRegisterModule",
            "lifecycle register module",
        ))
    }

    fn finish(
        &self,
        context: &mut PluginFinishContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        assert!(context.capabilities.has("runtime.feature.sound.occlusion"));
        self.log.borrow_mut().push("plugin.finish");
        context.registry.register_module(ModuleDescriptor::new(
            "LifecycleFinishModule",
            "lifecycle finish module",
        ))
    }
}

struct LifecycleOrderFeature<'a> {
    log: &'a RefCell<Vec<&'static str>>,
}

impl<'a> LifecycleOrderFeature<'a> {
    fn new(log: &'a RefCell<Vec<&'static str>>) -> Self {
        Self { log }
    }
}

impl RuntimePluginFeature for LifecycleOrderFeature<'_> {
    fn manifest(&self) -> PluginFeatureBundleManifest {
        PluginFeatureBundleManifest::new("sound.occlusion", "Sound Occlusion", "sound")
            .with_dependency(PluginFeatureDependency::primary(
                "sound",
                "runtime.plugin.sound",
            ))
            .with_capability("runtime.feature.sound.occlusion")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "sound.occlusion.runtime",
                    "zircon_plugin_sound_occlusion_runtime",
                )
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.feature.sound.occlusion"]),
            )
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("feature.register");
        registry.register_module(ModuleDescriptor::new(
            "LifecycleFeatureRegisterModule",
            "lifecycle feature register module",
        ))
    }

    fn finish(
        &self,
        context: &mut PluginFinishContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        assert!(context.capabilities.has("runtime.plugin.sound"));
        self.log.borrow_mut().push("feature.finish");
        context.registry.register_module(ModuleDescriptor::new(
            "LifecycleFeatureFinishModule",
            "lifecycle feature finish module",
        ))
    }
}
