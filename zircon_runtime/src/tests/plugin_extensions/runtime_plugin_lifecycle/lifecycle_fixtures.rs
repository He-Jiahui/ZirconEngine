use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OptionalDependencyProbeResult {
    pub(super) physics_raycast_available: bool,
    pub(super) physics_status: Option<CapabilityStatus>,
    pub(super) sound_occlusion_available: bool,
}

pub(super) struct OptionalDependencyProbe {
    descriptor: RuntimePluginDescriptor,
    pub(super) result: Cell<Option<OptionalDependencyProbeResult>>,
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

pub(super) struct LifecycleOrderPlugin<'a> {
    descriptor: RuntimePluginDescriptor,
    log: &'a RefCell<Vec<&'static str>>,
}

impl<'a> LifecycleOrderPlugin<'a> {
    pub(super) fn new(log: &'a RefCell<Vec<&'static str>>) -> Self {
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

pub(super) struct ReadyOrderPlugin<'a> {
    descriptor: RuntimePluginDescriptor,
    log: &'a RefCell<Vec<&'static str>>,
    ready_result: bool,
    expect_feature_capability: bool,
}

impl<'a> ReadyOrderPlugin<'a> {
    pub(super) fn new(log: &'a RefCell<Vec<&'static str>>, ready_result: bool) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "weather",
                "Weather",
                RuntimePluginId::Particles,
                "zircon_plugin_weather_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.weather")
            .build(),
            log,
            ready_result,
            expect_feature_capability: false,
        }
    }

    pub(super) fn expect_feature_capability(mut self) -> Self {
        self.expect_feature_capability = true;
        self
    }
}

impl RuntimePlugin for ReadyOrderPlugin<'_> {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("plugin.register");
        registry.register_module(ModuleDescriptor::new(
            "weather.runtime",
            "weather runtime module",
        ))
    }

    fn ready(
        &self,
        context: &PluginReadyContext<'_>,
    ) -> Result<bool, RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("plugin.ready");
        assert!(context
            .registry
            .modules()
            .iter()
            .any(|module| module.name == "weather.runtime"));
        assert!(context.capabilities.has("runtime.plugin.weather"));
        if self.expect_feature_capability {
            assert!(context
                .capabilities
                .has("runtime.feature.weather.occlusion"));
        }
        Ok(self.ready_result)
    }

    fn finish(
        &self,
        _context: &mut PluginFinishContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("plugin.finish");
        Ok(())
    }
}

pub(super) struct OrderedLifecyclePlugin<'a> {
    descriptor: RuntimePluginDescriptor,
    label: &'static str,
    log: &'a RefCell<Vec<&'static str>>,
    activate_error: Option<&'static str>,
}

impl<'a> OrderedLifecyclePlugin<'a> {
    pub(super) fn new(
        package_id: &str,
        display_name: &str,
        crate_name: &str,
        module_descriptor: ModuleDescriptor,
        label: &'static str,
        log: &'a RefCell<Vec<&'static str>>,
    ) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                package_id,
                display_name,
                RuntimePluginId::Particles,
                crate_name,
            )
            .with_module_descriptor(module_descriptor)
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability(format!("runtime.plugin.{package_id}"))
            .build(),
            label,
            log,
            activate_error: None,
        }
    }

    pub(super) fn with_activate_error(mut self, message: &'static str) -> Self {
        self.activate_error = Some(message);
        self
    }
}

impl RuntimePlugin for OrderedLifecyclePlugin<'_> {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        _registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push(match self.label {
            "base" => "base.register",
            "simulation" => "simulation.register",
            _ => "unknown.register",
        });
        Ok(())
    }

    fn finish(
        &self,
        _context: &mut PluginFinishContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push(match self.label {
            "base" => "base.finish",
            "simulation" => "simulation.finish",
            _ => "unknown.finish",
        });
        Ok(())
    }

    fn activate(
        &self,
        _context: &mut PluginRuntimeContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push(match self.label {
            "base" => "base.activate",
            "simulation" => "simulation.activate",
            _ => "unknown.activate",
        });
        if let Some(message) = self.activate_error {
            return Err(RuntimeExtensionRegistryError::InvalidPluginModule(
                message.to_string(),
            ));
        }
        Ok(())
    }

    fn deactivate(&self, _context: &mut PluginRuntimeContext<'_>) {
        self.log.borrow_mut().push(match self.label {
            "base" => "base.deactivate",
            "simulation" => "simulation.deactivate",
            _ => "unknown.deactivate",
        });
    }
}

pub(super) struct LifecycleOrderFeature<'a> {
    log: &'a RefCell<Vec<&'static str>>,
}

impl<'a> LifecycleOrderFeature<'a> {
    pub(super) fn new(log: &'a RefCell<Vec<&'static str>>) -> Self {
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

    fn activate(
        &self,
        _context: &mut PluginRuntimeContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("feature.activate");
        Ok(())
    }

    fn deactivate(&self, _context: &mut PluginRuntimeContext<'_>) {
        self.log.borrow_mut().push("feature.deactivate");
    }
}

pub(super) struct ReadyOrderFeature<'a> {
    log: &'a RefCell<Vec<&'static str>>,
    ready_result: bool,
}

impl<'a> ReadyOrderFeature<'a> {
    pub(super) fn new(log: &'a RefCell<Vec<&'static str>>, ready_result: bool) -> Self {
        Self { log, ready_result }
    }
}

impl RuntimePluginFeature for ReadyOrderFeature<'_> {
    fn manifest(&self) -> PluginFeatureBundleManifest {
        PluginFeatureBundleManifest::new("weather.occlusion", "Weather Occlusion", "weather")
            .with_dependency(PluginFeatureDependency::primary(
                "weather",
                "runtime.plugin.weather",
            ))
            .with_capability("runtime.feature.weather.occlusion")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "weather.occlusion.runtime",
                    "zircon_plugin_weather_occlusion_runtime",
                )
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.feature.weather.occlusion"]),
            )
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("feature.register");
        registry.register_module(ModuleDescriptor::new(
            "weather.occlusion.runtime",
            "weather occlusion runtime module",
        ))
    }

    fn ready(
        &self,
        context: &PluginReadyContext<'_>,
    ) -> Result<bool, RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("feature.ready");
        assert!(context
            .registry
            .modules()
            .iter()
            .any(|module| module.name == "weather.occlusion.runtime"));
        assert!(context.capabilities.has("runtime.plugin.weather"));
        assert!(context
            .capabilities
            .has("runtime.feature.weather.occlusion"));
        Ok(self.ready_result)
    }

    fn finish(
        &self,
        _context: &mut PluginFinishContext<'_>,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        self.log.borrow_mut().push("feature.finish");
        Ok(())
    }
}
