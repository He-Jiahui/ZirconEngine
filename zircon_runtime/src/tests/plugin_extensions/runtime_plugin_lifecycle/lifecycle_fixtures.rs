use std::sync::{Arc, Mutex};

use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::core::{CoreResult, ModuleContext, ModuleDescriptor, ModuleLifecycle};
use crate::plugin::{RuntimePlugin, RuntimePluginDescriptor};

#[derive(Debug)]
pub(super) struct RecordingModuleLifecycle {
    calls: Arc<Mutex<Vec<&'static str>>>,
}

impl RecordingModuleLifecycle {
    pub(super) fn new(calls: Arc<Mutex<Vec<&'static str>>>) -> Self {
        Self { calls }
    }

    fn record(&self, call: &'static str) {
        self.calls.lock().unwrap().push(call);
    }
}

impl ModuleLifecycle for RecordingModuleLifecycle {
    fn build(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.record("build");
        Ok(())
    }

    fn ready(&self, _context: &ModuleContext) -> CoreResult<bool> {
        self.record("ready");
        Ok(true)
    }

    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.record("finish");
        Ok(())
    }

    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        self.record("cleanup");
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) struct KernelLifecyclePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl KernelLifecyclePlugin {
    pub(super) fn new(lifecycle: Arc<dyn ModuleLifecycle>) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "weather",
                "Weather",
                RuntimePluginId::Particles,
                "zircon_plugin_weather_runtime",
            )
            .with_module_descriptor(
                ModuleDescriptor::new("weather.runtime", "Weather runtime")
                    .with_lifecycle(lifecycle),
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.weather")
            .build(),
        }
    }
}

impl RuntimePlugin for KernelLifecyclePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }
}
