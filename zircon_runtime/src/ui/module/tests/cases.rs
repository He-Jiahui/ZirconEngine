use std::sync::Arc;

use crate::core::framework::ui::UI_MODULE_NAME;
use crate::core::{CoreError, CoreRuntime, InitLevel, ModuleDescriptor};

use super::{module_descriptor, UiConfig, UiRuntimeDriver, UI_CONFIG_KEY, UI_RUNTIME_DRIVER_NAME};

fn registered_runtime() -> CoreRuntime {
    let runtime = CoreRuntime::new();
    let descriptor = module_descriptor();
    for dependency in &descriptor.module_dependencies {
        runtime
            .register_module(
                ModuleDescriptor::new(&dependency.module_name, "UI test dependency")
                    .with_init_level(InitLevel::Kernel),
            )
            .unwrap();
    }
    runtime.register_module(descriptor).unwrap();
    runtime
}

fn driver(runtime: &CoreRuntime) -> Arc<UiRuntimeDriver> {
    runtime.resolve_driver(UI_RUNTIME_DRIVER_NAME).unwrap()
}

#[test]
fn ui_driver_default_admission_is_stable_per_core_and_isolated_between_cores() {
    let first = registered_runtime();
    let second = registered_runtime();
    first.activate_module(UI_MODULE_NAME).unwrap();
    second.activate_module(UI_MODULE_NAME).unwrap();
    let retained = driver(&first);
    assert!(retained.admit_project().unwrap());
    assert!(Arc::ptr_eq(&retained, &driver(&first)));
    assert!(!Arc::ptr_eq(&retained, &driver(&second)));

    first.deactivate_module(UI_MODULE_NAME).unwrap();
    assert_eq!(
        retained.admit_project(),
        Err(CoreError::ServiceUnavailable(
            UI_RUNTIME_DRIVER_NAME.to_owned()
        ))
    );
    assert!(driver(&second).admit_project().unwrap());
}

#[test]
fn ui_driver_snapshots_config_and_reactivation_replaces_closed_admission() {
    let runtime = registered_runtime();
    runtime
        .handle()
        .store_config(UI_CONFIG_KEY, &UiConfig { enabled: false })
        .unwrap();
    runtime.activate_module(UI_MODULE_NAME).unwrap();
    let retained = driver(&runtime);
    assert!(!retained.admit_project().unwrap());
    runtime
        .handle()
        .store_config(UI_CONFIG_KEY, &UiConfig { enabled: true })
        .unwrap();
    assert!(!retained.config().enabled);
    assert!(!retained.admit_project().unwrap());

    runtime.deactivate_module(UI_MODULE_NAME).unwrap();
    assert!(matches!(
        retained.admit_project(),
        Err(CoreError::ServiceUnavailable(_))
    ));
    runtime.activate_module(UI_MODULE_NAME).unwrap();
    let replacement = driver(&runtime);
    assert!(!Arc::ptr_eq(&retained, &replacement));
    assert!(replacement.admit_project().unwrap());
    assert!(matches!(
        retained.admit_project(),
        Err(CoreError::ServiceUnavailable(_))
    ));
}

#[test]
fn ui_driver_invalid_config_fails_activation_without_defaulting_to_enabled() {
    let runtime = registered_runtime();
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
