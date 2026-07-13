use std::sync::{Arc, Mutex};

use zircon_runtime::core::{
    CoreResult, CoreRuntime, ModuleContext, ModuleDescriptor, ModuleLifecycle,
};
use zircon_runtime::plugin::{
    RuntimePlugin, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};
use zircon_runtime::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

#[derive(Debug)]
struct RecordingLifecycle {
    calls: Arc<Mutex<Vec<&'static str>>>,
}

impl RecordingLifecycle {
    fn record(&self, call: &'static str) {
        self.calls.lock().unwrap().push(call);
    }
}

impl ModuleLifecycle for RecordingLifecycle {
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
struct IntegrationPlugin {
    descriptor: RuntimePluginDescriptor,
}

impl IntegrationPlugin {
    fn new(lifecycle: Arc<dyn ModuleLifecycle>) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "frameworks02_integration",
                "Frameworks 02 Integration",
                RuntimePluginId::Particles,
                "zircon_plugin_frameworks02_integration",
            )
            .with_module_descriptor(
                ModuleDescriptor::new(
                    "frameworks02_integration.runtime",
                    "Frameworks 02 lifecycle integration module",
                )
                .with_lifecycle(lifecycle),
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.frameworks02_integration")
            .build(),
        }
    }
}

impl RuntimePlugin for IntegrationPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }
}

#[test]
fn report_registers_embedded_descriptor_once_and_core_runs_shared_lifecycle() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle: Arc<dyn ModuleLifecycle> = Arc::new(RecordingLifecycle {
        calls: Arc::clone(&calls),
    });
    let plugin = IntegrationPlugin::new(lifecycle);
    let report = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(report.extensions.modules().len(), 1);
    assert_eq!(
        report.extensions.modules()[0].name,
        plugin.module_descriptor().name
    );

    let runtime = CoreRuntime::new();
    runtime
        .register_module(report.extensions.modules()[0].clone())
        .unwrap();
    runtime.activate_registered_modules().unwrap();
    runtime
        .deactivate_module("frameworks02_integration.runtime")
        .unwrap();

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        ["build", "ready", "finish", "cleanup"]
    );
}
