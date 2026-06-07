use std::sync::{Arc, Mutex};

use super::super::*;
use super::fixtures::RecordedPlugin;
use crate::core::lifecycle::{ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[test]
fn plugin_resolution_builds_plugin_context_instead_of_passing_only_core_handle() {
    let seen = Arc::new(Mutex::new(None::<PluginContext>));
    let seen_for_factory = Arc::clone(&seen);
    let runtime = CoreRuntime::new();
    let core = runtime.handle();

    core.register_module(
        ModuleDescriptor::new("PluginContextSpec", "plugin context test").with_plugin(
            PluginDescriptor::new(
                RegistryName::from_parts(
                    "PluginContextSpec",
                    ServiceKind::Plugin,
                    "RecordedPlugin",
                ),
                StartupMode::Immediate,
                Vec::new(),
                Arc::new(move |context: &PluginContext| {
                    *seen_for_factory.lock().unwrap() = Some(context.clone());
                    Ok(Arc::new(RecordedPlugin(context.clone())) as ServiceObject)
                }),
            ),
        ),
    )
    .unwrap();

    core.activate_module("PluginContextSpec").unwrap();
    let resolved = core
        .resolve_plugin::<RecordedPlugin>("PluginContextSpec.Plugin.RecordedPlugin")
        .unwrap();
    let context = seen.lock().unwrap().clone().unwrap();

    assert_eq!(resolved.0.plugin_name, context.plugin_name);
    assert_eq!(
        context.plugin_name,
        "PluginContextSpec.Plugin.RecordedPlugin"
    );
    assert!(context.core.upgrade().is_some());
    assert!(context.package_root.is_none());
    assert!(context.source_root.is_none());
    assert!(context.data_root.is_none());
}
