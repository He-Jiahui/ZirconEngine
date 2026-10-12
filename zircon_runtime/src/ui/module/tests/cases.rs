use std::sync::{Arc, Mutex};

use crate::core::framework::ui::UI_MODULE_NAME;
use crate::core::runtime::ServiceCallGuard;
use crate::core::{
    CoreError, CoreResult, CoreRuntime, InitLevel, ModuleCleanupContext, ModuleDescriptor,
    ModuleLifecycle,
};

use super::{module_descriptor, UiConfig, UiRuntimeDriver, UI_CONFIG_KEY, UI_RUNTIME_DRIVER_NAME};

type CleanupObservations = Arc<Mutex<Vec<(usize, bool)>>>;

#[derive(Debug)]
struct ObservedUiCleanup {
    observations: CleanupObservations,
}

impl ModuleLifecycle for ObservedUiCleanup {
    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
        super::lifecycle::UiModuleLifecycle.cleanup(context)?;
        context.with_existing_driver::<UiRuntimeDriver>(UI_RUNTIME_DRIVER_NAME, |registered| {
            let rejects_admission = matches!(
                registered.admit_project(),
                Err(CoreError::ServiceUnavailable(name)) if name == UI_RUNTIME_DRIVER_NAME
            );
            self.observations.lock().unwrap().push((
                registered as *const UiRuntimeDriver as usize,
                rejects_admission,
            ));
            Ok(())
        })?;
        Ok(())
    }
}

fn registered_runtime() -> (CoreRuntime, CleanupObservations) {
    let runtime = CoreRuntime::new();
    let observations: CleanupObservations = Arc::new(Mutex::new(Vec::new()));
    let descriptor = module_descriptor().with_lifecycle(Arc::new(ObservedUiCleanup {
        observations: Arc::clone(&observations),
    }));
    for dependency in &descriptor.module_dependencies {
        runtime
            .register_module(
                ModuleDescriptor::new(&dependency.module_name, "UI test dependency")
                    .with_init_level(InitLevel::Kernel),
            )
            .unwrap();
    }
    runtime.register_module(descriptor).unwrap();
    (runtime, observations)
}

fn driver_identity(runtime: &CoreRuntime) -> (u32, u32) {
    let core = runtime.handle();
    let services = core.inner.services.lock().unwrap();
    let registered = services.get(UI_RUNTIME_DRIVER_NAME).unwrap();
    (registered.index, registered.generation)
}

fn driver(runtime: &CoreRuntime) -> ServiceCallGuard<UiRuntimeDriver> {
    runtime.resolve_driver(UI_RUNTIME_DRIVER_NAME).unwrap()
}

#[test]
fn ui_driver_default_admission_is_stable_per_core_and_isolated_between_cores() {
    let (first, first_cleanup) = registered_runtime();
    let (second, second_cleanup) = registered_runtime();
    first.activate_module(UI_MODULE_NAME).unwrap();
    second.activate_module(UI_MODULE_NAME).unwrap();
    let token = first
        .resolve_driver_handle::<UiRuntimeDriver>(UI_RUNTIME_DRIVER_NAME)
        .unwrap();
    let retained = token.enter().unwrap();
    let retained_pointer = &*retained as *const UiRuntimeDriver as usize;
    assert!(retained.admit_project().unwrap());
    assert!(std::ptr::eq(&*retained, &*driver(&first)));
    assert!(!std::ptr::eq(&*retained, &*driver(&second)));

    drop(retained);
    first.deactivate_module(UI_MODULE_NAME).unwrap();
    assert_eq!(
        first_cleanup.lock().unwrap().as_slice(),
        &[(retained_pointer, true)]
    );
    assert!(token.enter().is_err());
    assert!(second_cleanup.lock().unwrap().is_empty());
    assert!(driver(&second).admit_project().unwrap());
}

#[test]
fn ui_driver_snapshots_config_and_reactivation_replaces_closed_admission() {
    let (runtime, cleanup) = registered_runtime();
    runtime
        .handle()
        .store_config(UI_CONFIG_KEY, &UiConfig { enabled: false })
        .unwrap();
    runtime.activate_module(UI_MODULE_NAME).unwrap();
    let token = runtime
        .resolve_driver_handle::<UiRuntimeDriver>(UI_RUNTIME_DRIVER_NAME)
        .unwrap();
    let retained_identity = driver_identity(&runtime);
    let retained = token.enter().unwrap();
    let retained_pointer = &*retained as *const UiRuntimeDriver as usize;
    assert!(!retained.admit_project().unwrap());
    runtime
        .handle()
        .store_config(UI_CONFIG_KEY, &UiConfig { enabled: true })
        .unwrap();
    assert!(!retained.config().enabled);
    assert!(!retained.admit_project().unwrap());

    drop(retained);
    runtime.deactivate_module(UI_MODULE_NAME).unwrap();
    assert_eq!(
        cleanup.lock().unwrap().as_slice(),
        &[(retained_pointer, true)]
    );
    assert!(token.enter().is_err());
    runtime.activate_module(UI_MODULE_NAME).unwrap();
    let replacement_token = runtime
        .resolve_driver_handle::<UiRuntimeDriver>(UI_RUNTIME_DRIVER_NAME)
        .unwrap();
    let replacement_identity = driver_identity(&runtime);
    assert_eq!(retained_identity.0, replacement_identity.0);
    assert_ne!(retained_identity.1, replacement_identity.1);
    let replacement = replacement_token.enter().unwrap();
    assert!(replacement.config().enabled);
    assert!(replacement.admit_project().unwrap());
    assert!(token.enter().is_err());
}

#[test]
fn ui_driver_invalid_config_fails_activation_without_defaulting_to_enabled() {
    let (runtime, _) = registered_runtime();
    runtime
        .handle()
        .store_config_value(UI_CONFIG_KEY, serde_json::json!({ "enabled": "false" }));
    let failure = runtime.activate_module(UI_MODULE_NAME).unwrap_err();
    let activation = match failure {
        CoreError::ModuleActivationRollback { activation, .. } => *activation,
        other => other,
    };
    assert!(matches!(activation, CoreError::ConfigParse(key, _) if key == UI_CONFIG_KEY));
    assert!(runtime
        .resolve_driver::<UiRuntimeDriver>(UI_RUNTIME_DRIVER_NAME)
        .is_err());
}
