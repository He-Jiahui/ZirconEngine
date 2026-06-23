use std::sync::Arc;

use super::super::super::super::super::super::*;
use super::super::super::super::super::fixtures::{TestDriver, TestManager};
use crate::core::runtime::ServiceObject;
use crate::core::CoreError;
use crate::core::{ServiceKind, StartupMode};

#[test]
fn deactivate_reports_first_blocked_service_in_shutdown_order() {
    let runtime = CoreRuntime::new();
    let driver_name = RegistryName::from_parts("OrderedModule", ServiceKind::Driver, "Driver");
    let manager_name = RegistryName::from_parts("OrderedModule", ServiceKind::Manager, "Manager");
    let plugin_name = RegistryName::from_parts("OrderedModule", ServiceKind::Plugin, "Plugin");
    let driver_dependent_name =
        RegistryName::from_parts("DependentModule", ServiceKind::Manager, "DriverDependent");
    let plugin_dependent_name =
        RegistryName::from_parts("DependentModule", ServiceKind::Manager, "PluginDependent");

    runtime
        .register_module(
            ModuleDescriptor::new("OrderedModule", "ordered blocked unload")
                .with_driver(DriverDescriptor::new(
                    driver_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ))
                .with_manager(ManagerDescriptor::new(
                    manager_name,
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                ))
                .with_plugin(PluginDescriptor::new(
                    plugin_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestManager) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("DependentModule", "dependents")
                .with_manager(ManagerDescriptor::new(
                    driver_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(driver_name)],
                    Arc::new(|core| {
                        let _ = core.resolve_driver::<TestDriver>("OrderedModule.Driver.Driver")?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ))
                .with_manager(ManagerDescriptor::new(
                    plugin_dependent_name.clone(),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(plugin_name.clone())],
                    Arc::new(move |core| {
                        let _ = core.resolve_plugin::<TestManager>(plugin_name.as_str())?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                )),
        )
        .unwrap();

    runtime.activate_module("OrderedModule").unwrap();
    runtime.activate_module("DependentModule").unwrap();

    let error = runtime.deactivate_module("OrderedModule").unwrap_err();
    let CoreError::UnloadBlocked(blocked_service, dependents) = error else {
        panic!("expected unload block when multiple external dependents are alive");
    };
    assert_eq!(blocked_service, "OrderedModule.Plugin.Plugin");
    assert_eq!(dependents, vec![plugin_dependent_name.to_string()]);
}
