use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::super::super::super::*;
use super::super::super::fixtures::TestDriver;
use crate::core::framework::error::CoreResult;
use crate::core::runtime::ServiceObject;
use crate::core::{CoreError, LifecycleState, ServiceKind, StartupMode};

#[derive(Debug)]
struct RecordingLifecycle {
    calls: Arc<Mutex<Vec<String>>>,
    ready_after: usize,
    ready_calls: AtomicUsize,
    fail_finish: bool,
}

impl RecordingLifecycle {
    fn new(calls: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            calls,
            ready_after: 0,
            ready_calls: AtomicUsize::new(0),
            fail_finish: false,
        }
    }

    fn ready_after(mut self, ready_after: usize) -> Self {
        self.ready_after = ready_after;
        self
    }

    fn fail_finish(mut self) -> Self {
        self.fail_finish = true;
        self
    }

    fn record(&self, call: &'static str, context: &ModuleContext) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("{}:{}", context.module_name, call));
    }
}

impl ModuleLifecycle for RecordingLifecycle {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        self.record("build", context);
        Ok(())
    }

    fn ready(&self, context: &ModuleContext) -> CoreResult<bool> {
        self.record("ready", context);
        let call_index = self.ready_calls.fetch_add(1, Ordering::SeqCst);
        Ok(call_index >= self.ready_after)
    }

    fn finish(&self, context: &ModuleContext) -> CoreResult<()> {
        self.record("finish", context);
        if self.fail_finish {
            return Err(CoreError::MissingConfig("module.finish".to_owned()));
        }
        Ok(())
    }

    fn cleanup(&self, context: &ModuleContext) -> CoreResult<()> {
        self.record("cleanup", context);
        Ok(())
    }
}

fn recorded_calls(calls: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    calls.lock().unwrap().clone()
}

fn expected_calls(calls: &[&str]) -> Vec<String> {
    calls.iter().map(|call| (*call).to_owned()).collect()
}

#[test]
fn module_lifecycle_hooks_wrap_activation_and_deactivation() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(RecordingLifecycle::new(Arc::clone(&calls)));

    runtime
        .register_module(
            ModuleDescriptor::new("LifecycleModule", "lifecycle hooks").with_lifecycle(lifecycle),
        )
        .unwrap();

    runtime.activate_module("LifecycleModule").unwrap();
    runtime.deactivate_module("LifecycleModule").unwrap();

    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "LifecycleModule:build",
            "LifecycleModule:ready",
            "LifecycleModule:finish",
            "LifecycleModule:cleanup",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("LifecycleModule")
        .expect("module should stay registered after deactivation");
    assert_eq!(module.lifecycle, LifecycleState::Unloaded);
}

#[test]
fn module_ready_polling_allows_later_ready_result() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(RecordingLifecycle::new(Arc::clone(&calls)).ready_after(1));

    runtime
        .register_module(
            ModuleDescriptor::new("PollingReadyModule", "poll ready").with_lifecycle(lifecycle),
        )
        .unwrap();

    runtime
        .activate_module_with_ready_timeout("PollingReadyModule", Duration::from_millis(50))
        .unwrap();

    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "PollingReadyModule:build",
            "PollingReadyModule:ready",
            "PollingReadyModule:ready",
            "PollingReadyModule:finish",
        ])
    );
}

#[test]
fn module_ready_timeout_resets_module_and_started_services() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(RecordingLifecycle::new(Arc::clone(&calls)).ready_after(usize::MAX));
    let service_name =
        RegistryName::from_parts("TimeoutModule", ServiceKind::Driver, "ImmediateDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("TimeoutModule", "ready timeout")
                .with_lifecycle(lifecycle)
                .with_driver(DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                )),
        )
        .unwrap();

    let error = runtime
        .activate_module_with_ready_timeout("TimeoutModule", Duration::ZERO)
        .unwrap_err();
    assert!(matches!(
        error,
        CoreError::ModuleReadyTimeout { module, budget }
            if module == "TimeoutModule" && budget == Duration::ZERO
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&["TimeoutModule:build", "TimeoutModule:ready"])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("TimeoutModule")
        .expect("timed-out module should remain registered");
    assert_eq!(module.lifecycle, LifecycleState::Registered);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    let service = services
        .get(service_name.as_str())
        .expect("timed-out module should keep service entry");
    assert_eq!(service.lifecycle, LifecycleState::Registered);
    assert!(service.instance.is_none());
}

#[test]
fn module_finish_error_resets_module_and_started_services() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let lifecycle = Arc::new(RecordingLifecycle::new(Arc::clone(&calls)).fail_finish());
    let service_name =
        RegistryName::from_parts("FinishErrorModule", ServiceKind::Driver, "ImmediateDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("FinishErrorModule", "finish error")
                .with_lifecycle(lifecycle)
                .with_driver(DriverDescriptor::new(
                    service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                )),
        )
        .unwrap();

    let error = runtime.activate_module("FinishErrorModule").unwrap_err();
    assert!(matches!(
        error,
        CoreError::MissingConfig(key) if key == "module.finish"
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "FinishErrorModule:build",
            "FinishErrorModule:ready",
            "FinishErrorModule:finish",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    let module = modules
        .get("FinishErrorModule")
        .expect("failed module should remain registered");
    assert_eq!(module.lifecycle, LifecycleState::Registered);
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    let service = services
        .get(service_name.as_str())
        .expect("failed module should keep service entry");
    assert_eq!(service.lifecycle, LifecycleState::Registered);
    assert!(service.instance.is_none());
}

#[test]
fn activate_registered_modules_finishes_only_after_all_modules_are_ready() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));

    runtime
        .register_module(
            ModuleDescriptor::new("SceneModule", "scene")
                .with_init_level(InitLevel::Scene)
                .with_module_dependency(ModuleDependencySpec::named("ServersModule"))
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls)))),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("KernelModule", "kernel")
                .with_init_level(InitLevel::Kernel)
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls)))),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("ServersModule", "servers")
                .with_init_level(InitLevel::Servers)
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls)))),
        )
        .unwrap();

    runtime.activate_registered_modules().unwrap();

    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "KernelModule:build",
            "ServersModule:build",
            "SceneModule:build",
            "KernelModule:ready",
            "ServersModule:ready",
            "SceneModule:ready",
            "KernelModule:finish",
            "ServersModule:finish",
            "SceneModule:finish",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    for module_name in ["KernelModule", "ServersModule", "SceneModule"] {
        let module = modules
            .get(module_name)
            .expect("batch activation should keep every module registered");
        assert_eq!(module.lifecycle, LifecycleState::Running);
    }
}

#[test]
fn activate_registered_modules_rolls_back_all_started_modules_on_finish_error() {
    let runtime = CoreRuntime::new();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let first_service_name =
        RegistryName::from_parts("FirstBatchModule", ServiceKind::Driver, "ImmediateDriver");
    let second_service_name =
        RegistryName::from_parts("SecondBatchModule", ServiceKind::Driver, "ImmediateDriver");

    runtime
        .register_module(
            ModuleDescriptor::new("FirstBatchModule", "first")
                .with_init_level(InitLevel::Kernel)
                .with_lifecycle(Arc::new(RecordingLifecycle::new(Arc::clone(&calls))))
                .with_driver(DriverDescriptor::new(
                    first_service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 0 }) as ServiceObject)),
                )),
        )
        .unwrap();
    runtime
        .register_module(
            ModuleDescriptor::new("SecondBatchModule", "second")
                .with_init_level(InitLevel::Servers)
                .with_module_dependency(ModuleDependencySpec::named("FirstBatchModule"))
                .with_lifecycle(Arc::new(
                    RecordingLifecycle::new(Arc::clone(&calls)).fail_finish(),
                ))
                .with_driver(DriverDescriptor::new(
                    second_service_name.clone(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(|_| Ok(Arc::new(TestDriver { order: 1 }) as ServiceObject)),
                )),
        )
        .unwrap();

    let error = runtime.activate_registered_modules().unwrap_err();
    assert!(matches!(
        error,
        CoreError::MissingConfig(key) if key == "module.finish"
    ));
    assert_eq!(
        recorded_calls(&calls),
        expected_calls(&[
            "FirstBatchModule:build",
            "SecondBatchModule:build",
            "FirstBatchModule:ready",
            "SecondBatchModule:ready",
            "FirstBatchModule:finish",
            "SecondBatchModule:finish",
        ])
    );

    let handle = runtime.handle();
    let modules = handle.inner.modules.lock().unwrap();
    for module_name in ["FirstBatchModule", "SecondBatchModule"] {
        let module = modules
            .get(module_name)
            .expect("failed batch activation should keep every module registered");
        assert_eq!(module.lifecycle, LifecycleState::Registered);
    }
    drop(modules);

    let services = handle.inner.services.lock().unwrap();
    for service_name in [&first_service_name, &second_service_name] {
        let service = services
            .get(service_name.as_str())
            .expect("failed batch activation should keep service entries");
        assert_eq!(service.lifecycle, LifecycleState::Registered);
        assert!(service.instance.is_none());
    }
}
