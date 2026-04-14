//! Core runtime, service registry, and descriptors.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, Weak};

use crate::config_store::ConfigStore;
use crate::error::CoreError;
use crate::event_bus::{EngineEvent, EventBus};
use crate::job_scheduler::JobScheduler;
use crate::lifecycle::{LifecycleState, ServiceKind, StartupMode};
use crate::types::{ChannelReceiver, ServiceObject};

pub type ServiceFactory =
    Arc<dyn Fn(&CoreHandle) -> Result<ServiceObject, CoreError> + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegistryName(String);

impl RegistryName {
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let mut parts = value.split('.');
        let first = parts.next();
        let second = parts.next();
        let third = parts.next();
        if first.is_none()
            || second.is_none()
            || third.is_none()
            || first.is_some_and(str::is_empty)
            || second.is_some_and(str::is_empty)
            || third.is_some_and(str::is_empty)
        {
            return Err(CoreError::InvalidRegistryName(value));
        }
        Ok(Self(value))
    }

    pub fn from_parts(module: &str, kind: ServiceKind, service: &str) -> Self {
        Self(format!("{module}.{}.{}", kind.as_str(), service))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RegistryName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencySpec {
    pub name: RegistryName,
}

impl DependencySpec {
    pub fn named(name: RegistryName) -> Self {
        Self { name }
    }
}

#[derive(Clone)]
pub struct DriverDescriptor {
    pub name: RegistryName,
    pub startup_mode: StartupMode,
    pub dependencies: Vec<DependencySpec>,
    pub factory: ServiceFactory,
}

impl DriverDescriptor {
    pub fn new(
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: ServiceFactory,
    ) -> Self {
        Self {
            name,
            startup_mode,
            dependencies,
            factory,
        }
    }
}

impl fmt::Debug for DriverDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DriverDescriptor")
            .field("name", &self.name)
            .field("startup_mode", &self.startup_mode)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}

#[derive(Clone)]
pub struct ManagerDescriptor {
    pub name: RegistryName,
    pub startup_mode: StartupMode,
    pub dependencies: Vec<DependencySpec>,
    pub factory: ServiceFactory,
}

impl ManagerDescriptor {
    pub fn new(
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: ServiceFactory,
    ) -> Self {
        Self {
            name,
            startup_mode,
            dependencies,
            factory,
        }
    }
}

impl fmt::Debug for ManagerDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManagerDescriptor")
            .field("name", &self.name)
            .field("startup_mode", &self.startup_mode)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}

#[derive(Clone)]
pub struct PluginDescriptor {
    pub name: RegistryName,
    pub startup_mode: StartupMode,
    pub dependencies: Vec<DependencySpec>,
    pub factory: ServiceFactory,
}

impl PluginDescriptor {
    pub fn new(
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: ServiceFactory,
    ) -> Self {
        Self {
            name,
            startup_mode,
            dependencies,
            factory,
        }
    }
}

impl fmt::Debug for PluginDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginDescriptor")
            .field("name", &self.name)
            .field("startup_mode", &self.startup_mode)
            .field("dependencies", &self.dependencies)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct ModuleDescriptor {
    pub name: String,
    pub description: String,
    pub drivers: Vec<DriverDescriptor>,
    pub managers: Vec<ManagerDescriptor>,
    pub plugins: Vec<PluginDescriptor>,
}

impl ModuleDescriptor {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            drivers: Vec::new(),
            managers: Vec::new(),
            plugins: Vec::new(),
        }
    }

    pub fn with_driver(mut self, descriptor: DriverDescriptor) -> Self {
        self.drivers.push(descriptor);
        self
    }

    pub fn with_manager(mut self, descriptor: ManagerDescriptor) -> Self {
        self.managers.push(descriptor);
        self
    }

    pub fn with_plugin(mut self, descriptor: PluginDescriptor) -> Self {
        self.plugins.push(descriptor);
        self
    }
}

#[derive(Clone, Debug)]
pub struct ModuleContext {
    pub module_name: String,
    pub core: CoreWeak,
}

#[derive(Clone, Debug)]
pub struct PluginContext {
    pub plugin_name: String,
    pub core: CoreWeak,
}

#[derive(Clone)]
pub struct CoreRuntime {
    inner: Arc<CoreRuntimeInner>,
}

#[derive(Clone)]
pub struct CoreHandle {
    inner: Arc<CoreRuntimeInner>,
}

#[derive(Clone, Debug)]
pub struct CoreWeak {
    inner: Weak<CoreRuntimeInner>,
}

struct CoreRuntimeInner {
    modules: Mutex<HashMap<String, ModuleEntry>>,
    services: Mutex<HashMap<String, ServiceEntry>>,
    event_bus: EventBus,
    config_store: ConfigStore,
    scheduler: JobScheduler,
}

struct ModuleEntry {
    #[allow(dead_code)]
    descriptor: ModuleDescriptor,
    lifecycle: LifecycleState,
}

struct ServiceEntry {
    name: RegistryName,
    owner_module: String,
    kind: ServiceKind,
    startup_mode: StartupMode,
    dependencies: Vec<DependencySpec>,
    factory: ServiceFactory,
    lifecycle: LifecycleState,
    instance: Option<ServiceObject>,
}

impl CoreRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CoreRuntimeInner {
                modules: Mutex::new(HashMap::new()),
                services: Mutex::new(HashMap::new()),
                event_bus: EventBus::default(),
                config_store: ConfigStore::default(),
                scheduler: JobScheduler::default(),
            }),
        }
    }

    pub fn handle(&self) -> CoreHandle {
        CoreHandle {
            inner: self.inner.clone(),
        }
    }

    pub fn weak(&self) -> CoreWeak {
        self.handle().downgrade()
    }

    pub fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), CoreError> {
        self.handle().register_module(descriptor)
    }

    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.handle().activate_module(module_name)
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        self.handle().deactivate_module(module_name)
    }

    pub fn resolve_driver<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.handle().resolve_driver(name)
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        self.handle().resolve_manager(name)
    }

    pub fn publish_event(&self, topic: impl Into<String>, payload: Value) {
        self.handle().publish_event(topic, payload)
    }

    pub fn subscribe_events(&self, topic: impl Into<String>) -> ChannelReceiver<EngineEvent> {
        self.handle().subscribe_events(topic)
    }

    pub fn store_config_value(&self, key: impl Into<String>, value: Value) {
        self.handle().store_config_value(key, value)
    }

    pub fn load_config_value(&self, key: &str) -> Option<Value> {
        self.handle().load_config_value(key)
    }

    pub fn snapshot_config_values(&self) -> HashMap<String, Value> {
        self.handle().snapshot_config_values()
    }

    pub fn load_config<T: DeserializeOwned>(&self, key: &str) -> Result<T, CoreError> {
        self.handle().load_config(key)
    }
}

impl Default for CoreRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CoreRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreRuntime").finish()
    }
}

impl CoreHandle {
    pub fn downgrade(&self) -> CoreWeak {
        CoreWeak {
            inner: Arc::downgrade(&self.inner),
        }
    }

    pub fn register_module(&self, descriptor: ModuleDescriptor) -> Result<(), CoreError> {
        let module_name = descriptor.name.clone();
        let mut modules = self.inner.modules.lock().unwrap();
        if modules.contains_key(&module_name) {
            return Err(CoreError::DuplicateModule(module_name));
        }

        {
            let mut services = self.inner.services.lock().unwrap();
            for driver in &descriptor.drivers {
                self.insert_service(
                    &mut services,
                    module_name.clone(),
                    ServiceKind::Driver,
                    driver.name.clone(),
                    driver.startup_mode,
                    driver.dependencies.clone(),
                    driver.factory.clone(),
                )?;
            }
            for manager in &descriptor.managers {
                self.insert_service(
                    &mut services,
                    module_name.clone(),
                    ServiceKind::Manager,
                    manager.name.clone(),
                    manager.startup_mode,
                    manager.dependencies.clone(),
                    manager.factory.clone(),
                )?;
            }
            for plugin in &descriptor.plugins {
                self.insert_service(
                    &mut services,
                    module_name.clone(),
                    ServiceKind::Plugin,
                    plugin.name.clone(),
                    plugin.startup_mode,
                    plugin.dependencies.clone(),
                    plugin.factory.clone(),
                )?;
            }
        }

        modules.insert(
            module_name,
            ModuleEntry {
                descriptor,
                lifecycle: LifecycleState::Registered,
            },
        );
        Ok(())
    }

    fn insert_service(
        &self,
        services: &mut HashMap<String, ServiceEntry>,
        owner_module: String,
        kind: ServiceKind,
        name: RegistryName,
        startup_mode: StartupMode,
        dependencies: Vec<DependencySpec>,
        factory: ServiceFactory,
    ) -> Result<(), CoreError> {
        let key = name.to_string();
        if services.contains_key(&key) {
            return Err(CoreError::DuplicateService(key));
        }
        services.insert(
            key,
            ServiceEntry {
                name,
                owner_module,
                kind,
                startup_mode,
                dependencies,
                factory,
                lifecycle: LifecycleState::Registered,
                instance: None,
            },
        );
        Ok(())
    }

    pub fn activate_module(&self, module_name: &str) -> Result<(), CoreError> {
        {
            let mut modules = self.inner.modules.lock().unwrap();
            let entry = modules
                .get_mut(module_name)
                .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
            if entry.lifecycle == LifecycleState::Running {
                return Ok(());
            }
            entry.lifecycle = LifecycleState::Initializing;
        }

        let immediate_services = {
            let services = self.inner.services.lock().unwrap();
            let mut names: Vec<_> = services
                .values()
                .filter(|entry| {
                    entry.owner_module == module_name
                        && entry.startup_mode == StartupMode::Immediate
                })
                .map(|entry| (entry.kind, entry.name.to_string()))
                .collect();
            names.sort_by_key(|(kind, _)| match kind {
                ServiceKind::Driver => 0,
                ServiceKind::Manager => 1,
                ServiceKind::Plugin => 2,
            });
            names.into_iter().map(|(_, name)| name).collect::<Vec<_>>()
        };

        for service in immediate_services {
            self.resolve_named_service(&service, None)?;
        }

        let mut modules = self.inner.modules.lock().unwrap();
        let entry = modules
            .get_mut(module_name)
            .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
        entry.lifecycle = LifecycleState::Running;
        Ok(())
    }

    pub fn deactivate_module(&self, module_name: &str) -> Result<(), CoreError> {
        {
            let mut modules = self.inner.modules.lock().unwrap();
            let entry = modules
                .get_mut(module_name)
                .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
            entry.lifecycle = LifecycleState::Stopping;
        }

        let unload_order = {
            let services = self.inner.services.lock().unwrap();
            let mut names: Vec<_> = services
                .values()
                .filter(|entry| entry.owner_module == module_name)
                .map(|entry| (entry.kind, entry.name.to_string()))
                .collect();
            names.sort_by_key(|(kind, _)| match kind {
                ServiceKind::Plugin => 0,
                ServiceKind::Manager => 1,
                ServiceKind::Driver => 2,
            });
            names.into_iter().map(|(_, name)| name).collect::<Vec<_>>()
        };
        let unloading: HashSet<_> = unload_order.iter().cloned().collect();

        for service_name in unload_order {
            let dependents = self.running_dependents(&service_name, &unloading);
            if !dependents.is_empty() {
                return Err(CoreError::UnloadBlocked(service_name, dependents));
            }
            if let Some(entry) = self.inner.services.lock().unwrap().get_mut(&service_name) {
                entry.instance = None;
                entry.lifecycle = LifecycleState::Unloaded;
            }
        }

        let mut modules = self.inner.modules.lock().unwrap();
        let entry = modules
            .get_mut(module_name)
            .ok_or_else(|| CoreError::MissingModule(module_name.to_string()))?;
        entry.lifecycle = LifecycleState::Unloaded;
        Ok(())
    }

    fn running_dependents(&self, service_name: &str, unloading: &HashSet<String>) -> Vec<String> {
        self.inner
            .services
            .lock()
            .unwrap()
            .values()
            .filter(|entry| {
                entry.name.as_str() != service_name
                    && !unloading.contains(entry.name.as_str())
                    && entry.instance.is_some()
                    && entry
                        .dependencies
                        .iter()
                        .any(|dependency| dependency.name.as_str() == service_name)
            })
            .map(|entry| entry.name.to_string())
            .collect()
    }

    pub fn resolve_driver<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Driver))?;
        Arc::downcast::<T>(service).map_err(|_| CoreError::ServiceDowncast(name.to_string()))
    }

    pub fn resolve_manager<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Manager))?;
        Arc::downcast::<T>(service).map_err(|_| CoreError::ServiceDowncast(name.to_string()))
    }

    pub fn resolve_plugin<T: Any + Send + Sync>(&self, name: &str) -> Result<Arc<T>, CoreError> {
        let service = self.resolve_named_service(name, Some(ServiceKind::Plugin))?;
        Arc::downcast::<T>(service).map_err(|_| CoreError::ServiceDowncast(name.to_string()))
    }

    fn resolve_named_service(
        &self,
        service_name: &str,
        expected_kind: Option<ServiceKind>,
    ) -> Result<ServiceObject, CoreError> {
        self.resolve_named_service_inner(service_name, expected_kind, &mut Vec::new())
    }

    fn resolve_named_service_inner(
        &self,
        service_name: &str,
        expected_kind: Option<ServiceKind>,
        stack: &mut Vec<String>,
    ) -> Result<ServiceObject, CoreError> {
        {
            let services = self.inner.services.lock().unwrap();
            let entry = services
                .get(service_name)
                .ok_or_else(|| CoreError::MissingService(service_name.to_string()))?;
            if let Some(expected_kind) = expected_kind {
                if entry.kind != expected_kind {
                    return Err(CoreError::ServiceKindMismatch {
                        name: service_name.to_string(),
                        expected: expected_kind,
                        actual: entry.kind,
                    });
                }
            }
            if let Some(instance) = entry.instance.clone() {
                return Ok(instance);
            }
        }

        if stack.iter().any(|existing| existing == service_name) {
            return Err(CoreError::DependencyCycle(service_name.to_string()));
        }
        stack.push(service_name.to_string());

        let (owner_module, dependencies, factory) = {
            let mut services = self.inner.services.lock().unwrap();
            let entry = services
                .get_mut(service_name)
                .ok_or_else(|| CoreError::MissingService(service_name.to_string()))?;
            entry.lifecycle = LifecycleState::Initializing;
            (
                entry.owner_module.clone(),
                entry.dependencies.clone(),
                entry.factory.clone(),
            )
        };

        let should_activate = {
            let modules = self.inner.modules.lock().unwrap();
            modules
                .get(&owner_module)
                .is_some_and(|module| module.lifecycle == LifecycleState::Registered)
        };
        if should_activate {
            self.activate_module(&owner_module)?;
        }

        for dependency in &dependencies {
            self.resolve_named_service_inner(dependency.name.as_str(), None, stack)?;
        }

        let instance = factory(self).map_err(|error| {
            CoreError::Initialization(service_name.to_string(), error.to_string())
        })?;

        {
            let mut services = self.inner.services.lock().unwrap();
            let entry = services
                .get_mut(service_name)
                .ok_or_else(|| CoreError::MissingService(service_name.to_string()))?;
            entry.instance = Some(instance.clone());
            entry.lifecycle = LifecycleState::Running;
        }

        stack.pop();
        Ok(instance)
    }

    pub fn publish_event(&self, topic: impl Into<String>, payload: Value) {
        self.inner.event_bus.publish(EngineEvent {
            topic: topic.into(),
            payload,
        });
    }

    pub fn subscribe_events(&self, topic: impl Into<String>) -> ChannelReceiver<EngineEvent> {
        self.inner.event_bus.subscribe(topic)
    }

    pub fn store_config_value(&self, key: impl Into<String>, value: Value) {
        self.inner.config_store.store_value(key, value);
    }

    pub fn load_config_value(&self, key: &str) -> Option<Value> {
        self.inner.config_store.load_value(key)
    }

    pub fn snapshot_config_values(&self) -> HashMap<String, Value> {
        self.inner.config_store.snapshot_values()
    }

    pub fn store_config<T: serde::Serialize>(
        &self,
        key: impl Into<String>,
        value: &T,
    ) -> Result<(), CoreError> {
        self.inner.config_store.store(key, value)
    }

    pub fn load_config<T: DeserializeOwned>(&self, key: &str) -> Result<T, CoreError> {
        self.inner.config_store.load(key)
    }

    pub fn scheduler(&self) -> &JobScheduler {
        &self.inner.scheduler
    }
}

impl fmt::Debug for CoreHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CoreHandle").finish()
    }
}

impl CoreWeak {
    pub fn upgrade(&self) -> Option<CoreHandle> {
        self.inner.upgrade().map(|inner| CoreHandle { inner })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel_util::recv_latest;
    use crossbeam_channel::unbounded;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestDriver {
        order: usize,
    }

    #[derive(Debug)]
    struct TestManager;

    #[test]
    fn recv_latest_keeps_last_message() {
        let (sender, receiver) = unbounded();
        sender.send(1).unwrap();
        sender.send(2).unwrap();

        assert_eq!(recv_latest(&receiver), Some(2));
        assert_eq!(recv_latest::<i32>(&receiver), None);
    }

    #[test]
    fn immediate_services_activate_in_dependency_order() {
        let runtime = CoreRuntime::new();
        let order = Arc::new(AtomicUsize::new(0));

        let driver_order = order.clone();
        let driver = DriverDescriptor::new(
            RegistryName::from_parts("TestModule", ServiceKind::Driver, "ClockDriver"),
            StartupMode::Immediate,
            Vec::new(),
            Arc::new(move |_| {
                let order = driver_order.fetch_add(1, Ordering::SeqCst);
                Ok(Arc::new(TestDriver { order }) as ServiceObject)
            }),
        );
        let manager = ManagerDescriptor::new(
            RegistryName::from_parts("TestModule", ServiceKind::Manager, "ClockManager"),
            StartupMode::Immediate,
            vec![DependencySpec::named(RegistryName::from_parts(
                "TestModule",
                ServiceKind::Driver,
                "ClockDriver",
            ))],
            Arc::new(move |core| {
                let _driver = core.resolve_driver::<TestDriver>("TestModule.Driver.ClockDriver")?;
                Ok(Arc::new(TestManager) as ServiceObject)
            }),
        );

        runtime
            .register_module(
                ModuleDescriptor::new("TestModule", "test")
                    .with_driver(driver)
                    .with_manager(manager),
            )
            .unwrap();
        runtime.activate_module("TestModule").unwrap();

        let driver = runtime
            .resolve_driver::<TestDriver>("TestModule.Driver.ClockDriver")
            .unwrap();
        assert_eq!(driver.order, 0);
    }

    #[test]
    fn lazy_manager_is_created_on_first_resolve() {
        let runtime = CoreRuntime::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let manager_calls = calls.clone();

        runtime
            .register_module(ModuleDescriptor::new("LazyModule", "lazy").with_manager(
                ManagerDescriptor::new(
                    RegistryName::from_parts("LazyModule", ServiceKind::Manager, "LazyManager"),
                    StartupMode::Lazy,
                    Vec::new(),
                    Arc::new(move |_| {
                        manager_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ))
            .unwrap();
        runtime.activate_module("LazyModule").unwrap();

        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let _ = runtime
            .resolve_manager::<TestManager>("LazyModule.Manager.LazyManager")
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn deactivate_blocks_when_external_dependents_are_alive() {
        let runtime = CoreRuntime::new();
        runtime
            .register_module(ModuleDescriptor::new("ModuleA", "a").with_driver(
                DriverDescriptor::new(
                    RegistryName::from_parts("ModuleA", ServiceKind::Driver, "ClockDriver"),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                ),
            ))
            .unwrap();
        runtime
            .register_module(ModuleDescriptor::new("ModuleB", "b").with_manager(
                ManagerDescriptor::new(
                    RegistryName::from_parts("ModuleB", ServiceKind::Manager, "ClockManager"),
                    StartupMode::Immediate,
                    vec![DependencySpec::named(RegistryName::from_parts(
                        "ModuleA",
                        ServiceKind::Driver,
                        "ClockDriver",
                    ))],
                    Arc::new(|core| {
                        let _ = core.resolve_driver::<TestDriver>("ModuleA.Driver.ClockDriver")?;
                        Ok(Arc::new(TestManager) as ServiceObject)
                    }),
                ),
            ))
            .unwrap();

        runtime.activate_module("ModuleA").unwrap();
        runtime.activate_module("ModuleB").unwrap();
        let error = runtime.deactivate_module("ModuleA").unwrap_err();
        assert!(matches!(error, CoreError::UnloadBlocked(_, _)));
    }

    #[test]
    fn event_bus_and_config_store_roundtrip() {
        let runtime = CoreRuntime::new();
        let events = runtime.handle().subscribe_events("editor.selection");
        runtime.publish_event("editor.selection", serde_json::json!({ "node": 7 }));
        let event = events.recv().unwrap();
        assert_eq!(event.payload["node"], 7);

        runtime
            .handle()
            .store_config("editor.theme", &serde_json::json!({ "name": "TokyoNight" }))
            .unwrap();
        let theme: Value = runtime.load_config("editor.theme").unwrap();
        assert_eq!(theme["name"], "TokyoNight");
    }
}
