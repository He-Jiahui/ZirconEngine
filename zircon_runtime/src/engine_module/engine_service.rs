use std::fmt;
use std::sync::Arc;

use crate::core::{
    DependencySpec, DriverDescriptor, ManagerDescriptor, PluginDescriptor, RegistryName,
    ServiceKind, StartupMode,
};

pub trait EngineService: Send + Sync + fmt::Debug {
    fn owner_module(&self) -> &str;
    fn registry_name(&self) -> &RegistryName;
    fn service_kind(&self) -> ServiceKind;
    fn startup_mode(&self) -> StartupMode;
    fn dependencies(&self) -> &[DependencySpec];
}

pub trait EngineDriver: EngineService {}
pub trait EngineManager: EngineService {}
pub trait EnginePlugin: EngineService {}

#[derive(Clone, Debug)]
struct ServiceContract {
    owner_module: String,
    registry_name: RegistryName,
    service_kind: ServiceKind,
    startup_mode: StartupMode,
    dependencies: Arc<[DependencySpec]>,
}

impl ServiceContract {
    fn new(
        owner_module: impl Into<String>,
        registry_name: RegistryName,
        service_kind: ServiceKind,
        startup_mode: StartupMode,
        dependencies: Arc<[DependencySpec]>,
    ) -> Self {
        Self {
            owner_module: owner_module.into(),
            registry_name,
            service_kind,
            startup_mode,
            dependencies,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DriverContract(ServiceContract);

#[derive(Clone, Debug)]
pub struct ManagerContract(ServiceContract);

#[derive(Clone, Debug)]
pub struct PluginContract(ServiceContract);

impl EngineService for DriverContract {
    fn owner_module(&self) -> &str {
        &self.0.owner_module
    }

    fn registry_name(&self) -> &RegistryName {
        &self.0.registry_name
    }

    fn service_kind(&self) -> ServiceKind {
        self.0.service_kind
    }

    fn startup_mode(&self) -> StartupMode {
        self.0.startup_mode
    }

    fn dependencies(&self) -> &[DependencySpec] {
        &self.0.dependencies
    }
}

impl EngineService for ManagerContract {
    fn owner_module(&self) -> &str {
        &self.0.owner_module
    }

    fn registry_name(&self) -> &RegistryName {
        &self.0.registry_name
    }

    fn service_kind(&self) -> ServiceKind {
        self.0.service_kind
    }

    fn startup_mode(&self) -> StartupMode {
        self.0.startup_mode
    }

    fn dependencies(&self) -> &[DependencySpec] {
        &self.0.dependencies
    }
}

impl EngineService for PluginContract {
    fn owner_module(&self) -> &str {
        &self.0.owner_module
    }

    fn registry_name(&self) -> &RegistryName {
        &self.0.registry_name
    }

    fn service_kind(&self) -> ServiceKind {
        self.0.service_kind
    }

    fn startup_mode(&self) -> StartupMode {
        self.0.startup_mode
    }

    fn dependencies(&self) -> &[DependencySpec] {
        &self.0.dependencies
    }
}

impl EngineDriver for DriverContract {}
impl EngineManager for ManagerContract {}
impl EnginePlugin for PluginContract {}

pub fn driver_contract(
    owner_module: impl Into<String>,
    descriptor: &DriverDescriptor,
) -> DriverContract {
    DriverContract(ServiceContract::new(
        owner_module,
        descriptor.name.clone(),
        ServiceKind::Driver,
        descriptor.startup_mode,
        Arc::clone(&descriptor.dependencies),
    ))
}

pub fn manager_contract(
    owner_module: impl Into<String>,
    descriptor: &ManagerDescriptor,
) -> ManagerContract {
    ManagerContract(ServiceContract::new(
        owner_module,
        descriptor.name.clone(),
        ServiceKind::Manager,
        descriptor.startup_mode,
        Arc::clone(&descriptor.dependencies),
    ))
}

pub fn plugin_contract(
    owner_module: impl Into<String>,
    descriptor: &PluginDescriptor,
) -> PluginContract {
    PluginContract(ServiceContract::new(
        owner_module,
        descriptor.name.clone(),
        ServiceKind::Plugin,
        descriptor.startup_mode,
        Arc::clone(&descriptor.dependencies),
    ))
}

#[cfg(test)]
mod shared_dependency_tests {
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
}
