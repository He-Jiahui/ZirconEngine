use super::*;
use crate::core::runtime::ServiceObject;

fn dependency() -> DependencySpec {
    DependencySpec::named(RegistryName::from_parts(
        "Runtime46",
        ServiceKind::Manager,
        "Dependency",
    ))
}

#[test]
fn service_contracts_share_descriptor_dependency_slices() {
    let driver = DriverDescriptor::new(
        RegistryName::from_parts("Runtime46", ServiceKind::Driver, "Driver"),
        StartupMode::Immediate,
        vec![dependency()],
        Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
    );
    let manager = ManagerDescriptor::new(
        RegistryName::from_parts("Runtime46", ServiceKind::Manager, "Manager"),
        StartupMode::Lazy,
        vec![dependency()],
        Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
    );
    let plugin = PluginDescriptor::new(
        RegistryName::from_parts("Runtime46", ServiceKind::Plugin, "Plugin"),
        StartupMode::Immediate,
        vec![dependency()],
        Arc::new(|_| Ok(Arc::new(()) as ServiceObject)),
    );

    let driver_contract = driver_contract("Runtime46", &driver);
    let manager_contract = manager_contract("Runtime46", &manager);
    let plugin_contract = plugin_contract("Runtime46", &plugin);

    assert!(Arc::ptr_eq(
        &driver.dependencies,
        &driver_contract.0.dependencies
    ));
    assert!(Arc::ptr_eq(
        &manager.dependencies,
        &manager_contract.0.dependencies
    ));
    assert!(Arc::ptr_eq(
        &plugin.dependencies,
        &plugin_contract.0.dependencies
    ));
}
