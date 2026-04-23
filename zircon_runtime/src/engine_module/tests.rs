use std::sync::Arc;

use crate::core::{
    CoreRuntime, DriverDescriptor, ManagerDescriptor, ModuleDescriptor, PluginDescriptor,
    RegistryName, ServiceKind, ServiceObject, StartupMode,
};

use super::{
    dependency_on, driver_contract, factory, module_context, plugin_context, plugin_factory,
    qualified_name, EngineModule, EngineService,
};

fn stub_driver_descriptor(
    module: &str,
    service: &str,
    startup_mode: StartupMode,
) -> DriverDescriptor {
    let name = qualified_name(module, ServiceKind::Driver, service);
    let service_name = name.to_string();
    DriverDescriptor::new(
        name,
        startup_mode,
        Vec::new(),
        Arc::new(move |_| Ok(Arc::new(service_name.clone()) as ServiceObject)),
    )
}

fn stub_manager_descriptor(
    module: &str,
    service: &str,
    startup_mode: StartupMode,
) -> ManagerDescriptor {
    let name = qualified_name(module, ServiceKind::Manager, service);
    let service_name = name.to_string();
    ManagerDescriptor::new(
        name,
        startup_mode,
        Vec::new(),
        Arc::new(move |_| Ok(Arc::new(service_name.clone()) as ServiceObject)),
    )
}

fn stub_plugin_descriptor(
    module: &str,
    service: &str,
    startup_mode: StartupMode,
) -> PluginDescriptor {
    let name = qualified_name(module, ServiceKind::Plugin, service);
    let service_name = name.to_string();
    PluginDescriptor::new(
        name,
        startup_mode,
        Vec::new(),
        plugin_factory(move |_| Ok(Arc::new(service_name.clone()) as ServiceObject)),
    )
}

fn stub_module_descriptor(
    module: &str,
    description: &str,
    driver_service: &str,
    manager_service: &str,
) -> ModuleDescriptor {
    ModuleDescriptor::new(module, description)
        .with_driver(stub_driver_descriptor(
            module,
            driver_service,
            StartupMode::Immediate,
        ))
        .with_manager(stub_manager_descriptor(
            module,
            manager_service,
            StartupMode::Lazy,
        ))
}

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
    let plugin_context = plugin_context("ToolPlugin", weak);
    assert_eq!(plugin_context.plugin_name, "ToolPlugin");
    assert!(plugin_context.package_root.is_none());
    assert!(plugin_context.source_root.is_none());
    assert!(plugin_context.data_root.is_none());

    let factory = factory(|_| Ok(Arc::new("service".to_string()) as _));
    let service = factory(&runtime.handle());
    assert!(service.is_ok());

    let plugin_factory =
        plugin_factory(|context| Ok(Arc::new(context.plugin_name.clone()) as ServiceObject));
    let plugin = plugin_factory(&plugin_context);
    assert!(plugin.is_ok());
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

#[test]
fn engine_module_contract_exposes_identity_and_descriptor() {
    #[derive(Debug, Default)]
    struct UiModule;

    impl EngineModule for UiModule {
        fn module_name(&self) -> &'static str {
            "UiModule"
        }

        fn module_description(&self) -> &'static str {
            "UI test module"
        }

        fn descriptor(&self) -> ModuleDescriptor {
            stub_module_descriptor(
                self.module_name(),
                self.module_description(),
                "UiDriver",
                "UiManager",
            )
        }
    }

    let module = UiModule;
    let descriptor = module.descriptor();

    assert_eq!(module.module_name(), "UiModule");
    assert_eq!(module.module_description(), "UI test module");
    assert_eq!(descriptor.name, module.module_name());
    assert_eq!(descriptor.description, module.module_description());
}

#[test]
fn driver_contract_preserves_descriptor_metadata() {
    let descriptor = stub_driver_descriptor("UiModule", "UiDriver", StartupMode::Lazy);
    let contract = driver_contract("UiModule", &descriptor);

    assert_eq!(contract.owner_module(), "UiModule");
    assert_eq!(contract.registry_name(), &descriptor.name);
    assert_eq!(contract.service_kind(), ServiceKind::Driver);
    assert_eq!(contract.startup_mode(), StartupMode::Lazy);
    assert!(contract.dependencies().is_empty());
}
