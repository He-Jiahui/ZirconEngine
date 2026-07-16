use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use std::time::Duration;

use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

use super::support::{
    editor_runtime_with_disabled_subsystems_config_path, env_lock, unique_temp_path,
};

#[test]
fn registry_owned_editor_manager_does_not_retain_the_runtime_root() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_runtime_lifecycle");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let weak = runtime.weak();

    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    drop(runtime);

    assert!(
        weak.upgrade().is_none(),
        "a registry-owned editor service must not keep CoreRuntime alive"
    );
    assert_eq!(manager.runtime_diagnostics(), Default::default());
    drop(manager);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
}

#[test]
fn failed_project_open_does_not_make_the_editor_manager_retain_the_runtime_root() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_failed_project_open_lifecycle");
    let missing_project = unique_temp_path("zircon_editor_missing_project");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let weak = runtime.weak();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();

    assert!(manager.open_project(&missing_project).is_err());
    drop(runtime);

    assert!(
        weak.upgrade().is_none(),
        "a failed project open must not add a strong Runtime back-reference"
    );
    assert_eq!(manager.runtime_diagnostics(), Default::default());
    drop(manager);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
}

#[test]
fn project_open_panic_unwind_drops_temporary_editor_manager_owners() {
    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_project_open_unwind_lifecycle");
    let missing_project = unique_temp_path("zircon_editor_unwind_missing_project");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
    let weak = runtime.weak();
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let unwind_manager = Arc::clone(&manager);

    let panic_result = catch_unwind(AssertUnwindSafe(move || {
        unwind_manager
            .open_project(&missing_project)
            .expect("missing project deliberately drives the unwind path");
    }));
    assert!(panic_result.is_err());
    drop(runtime);

    assert!(
        weak.upgrade().is_none(),
        "panic unwind must release temporary editor service owners"
    );
    assert_eq!(manager.runtime_diagnostics(), Default::default());
    drop(manager);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
}

#[test]
fn repeated_editor_runtime_fixtures_release_every_runtime_root() {
    const FIXTURE_COUNT: usize = 128;
    const FIXTURE_TEARDOWN_OBSERVATION_BUDGET: Duration = Duration::from_millis(500);

    let _guard = env_lock().lock().unwrap();
    let config_path = unique_temp_path("zircon_editor_repeated_runtime_lifecycle");

    for fixture_index in 0..FIXTURE_COUNT {
        let runtime = editor_runtime_with_disabled_subsystems_config_path(&config_path);
        let weak = runtime.weak();
        let manager = runtime
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();

        drop(manager);
        drop(runtime);

        assert!(
            weak.upgrade().is_none(),
            "editor Runtime fixture {fixture_index} must release its root while the process task owner stays shared"
        );
    }

    // Keep the process alive briefly so the acceptance monitor can observe a stable process budget
    // after every fixture-local Runtime root has been released.
    std::thread::sleep(FIXTURE_TEARDOWN_OBSERVATION_BUDGET);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
}
