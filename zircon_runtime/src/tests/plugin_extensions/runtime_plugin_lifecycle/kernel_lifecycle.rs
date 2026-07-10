use std::sync::{Arc, Mutex};

use crate::core::{CoreRuntime, ModuleLifecycle};
use crate::plugin::{RuntimePlugin, RuntimePluginRegistrationReport};

#[path = "lifecycle_fixtures.rs"]
mod lifecycle_fixtures;

use lifecycle_fixtures::{KernelLifecyclePlugin, RecordingModuleLifecycle};

#[test]
fn runtime_plugin_embedded_descriptor_uses_kernel_module_lifecycle() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle: Arc<dyn ModuleLifecycle> =
        Arc::new(RecordingModuleLifecycle::new(Arc::clone(&calls)));
    let plugin = KernelLifecyclePlugin::new(lifecycle);
    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);
    assert_eq!(registration.extensions.modules().len(), 1);
    let descriptor = registration
        .extensions
        .modules()
        .first()
        .expect("runtime plugin module descriptor")
        .clone();
    let runtime = CoreRuntime::new();

    assert_eq!(plugin.module_descriptor().name, descriptor.name);
    let _shared_lifecycle: &dyn ModuleLifecycle = plugin.lifecycle();
    runtime.register_module(descriptor).unwrap();
    runtime.activate_registered_modules().unwrap();
    runtime.deactivate_module("weather.runtime").unwrap();

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        ["build", "ready", "finish", "cleanup"]
    );
}
