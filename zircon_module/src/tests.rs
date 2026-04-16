use std::sync::Arc;

use zircon_core::{CoreRuntime, RegistryName, ServiceKind, StartupMode};

use crate::{
    dependency_on, factory, module_context, plugin_context, qualified_name, stub_module_descriptor,
    stub_plugin_descriptor,
};

#[test]
fn qualified_name_and_dependency_helpers_share_registry_shape() {
    let name = qualified_name("UiModule", ServiceKind::Manager, "InputManager");

    assert_eq!(
        name,
        RegistryName::from_parts("UiModule", ServiceKind::Manager, "InputManager")
    );
    assert_eq!(
        dependency_on("UiModule", ServiceKind::Manager, "InputManager").name,
        name
    );
}

#[test]
fn stub_module_descriptor_wires_driver_and_manager_defaults() {
    let descriptor =
        stub_module_descriptor("UiModule", "UI test module", "InputDriver", "InputManager");

    assert_eq!(descriptor.name, "UiModule");
    assert_eq!(descriptor.description, "UI test module");
    assert_eq!(
        descriptor.drivers.first().map(|driver| driver.startup_mode),
        Some(StartupMode::Immediate),
    );
    assert_eq!(
        descriptor
            .managers
            .first()
            .map(|manager| manager.startup_mode),
        Some(StartupMode::Lazy),
    );
}

#[test]
fn contexts_and_factory_preserve_supplied_names() {
    let runtime = CoreRuntime::new();
    let weak = runtime.weak();

    assert_eq!(
        module_context("UiModule", weak.clone()).module_name,
        "UiModule"
    );
    assert_eq!(plugin_context("ToolPlugin", weak).plugin_name, "ToolPlugin");

    let factory = factory(|_| Ok(Arc::new("service".to_string()) as _));
    let service = factory(&runtime.handle());
    assert!(service.is_ok());
}

#[test]
fn stub_plugin_descriptor_uses_plugin_registry_kind() {
    let descriptor = stub_plugin_descriptor("UiModule", "ToolPlugin", StartupMode::Lazy);

    assert_eq!(
        descriptor.name,
        qualified_name("UiModule", ServiceKind::Plugin, "ToolPlugin")
    );
    assert_eq!(descriptor.startup_mode, StartupMode::Lazy);
}
