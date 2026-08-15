use super::*;

#[test]
fn runtime_15_core_handle_diagnostics_lock_poison_recovery_guard_covers_diagnostic_store() {
    let diagnostics_handle = read_runtime_src("core/runtime/handle/diagnostics.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let diagnostics_doc = read_repo("docs/zircon_runtime/core/diagnostics.md");

    assert_contains_all(
        "CoreHandle diagnostics poison recovery",
        &diagnostics_handle,
        &[
            "use std::sync::MutexGuard;",
            "fn lock_diagnostics(&self) -> MutexGuard<'_, DiagnosticStore>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_diagnostics().clone()",
            "self.lock_diagnostics().snapshot()",
            "self.lock_diagnostics()",
            "core_handle_diagnostic_accessors_recover_poisoned_store_lock",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("core handle diagnostics", &diagnostics_handle);
}

#[test]
fn runtime_15_core_handle_time_lock_poison_recovery_guard_covers_runtime_clocks() {
    let time_handle = read_runtime_src("core/runtime/handle/time.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let diagnostics_doc = read_repo("docs/zircon_runtime/core/diagnostics.md");

    assert_contains_all(
        "CoreHandle time poison recovery",
        &time_handle,
        &[
            "use std::sync::MutexGuard;",
            "fn lock_time(&self) -> MutexGuard<'_, RuntimeTimeClocks>",
            "fn lock_frame_clock(&self) -> MutexGuard<'_, FrameClock>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "*self.lock_time()",
            "let mut time = self.lock_time();",
            "self.lock_frame_clock().tick()",
            "self.lock_time().pause_virtual_time()",
            "record_time_diagnostics(self, clocks, advance)",
            "handle.record_diagnostic(",
            "core_handle_time_accessors_recover_poisoned_runtime_clocks",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("core handle time", &time_handle);
}

#[test]
fn runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry() {
    let states_handle = read_runtime_src("core/runtime/handle/states.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let state_doc = read_repo("docs/zircon_runtime/core/state.md");

    assert_contains_all(
        "CoreHandle states poison recovery",
        &states_handle,
        &[
            "use std::sync::MutexGuard;",
            "fn lock_states(&self) -> MutexGuard<'_, StateRegistry>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_states().init_state::<T>(T::default())",
            "self.lock_states().insert_state(state)",
            "self.lock_states().state::<T>()",
            "self.lock_states().next_state::<T>()",
            "self.lock_states().set_next_state(state)",
            "self.lock_states().apply_state_transition::<T>()?",
            "self.lock_states().transition_events::<T>()",
            "core_handle_state_accessors_recover_poisoned_state_registry_lock",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("core handle states", &states_handle);
}

#[test]
fn runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors() {
    let core_handle = read_runtime_src("core/runtime/handle/core_handle.rs");
    let activation = read_runtime_src("core/runtime/handle/activation.rs");
    let register_module = read_runtime_src("core/runtime/handle/registration/register_module.rs");
    let resolution = read_runtime_src("core/runtime/handle/resolution.rs");
    let runtime_extensions = read_runtime_src("core/runtime/handle/runtime_extensions.rs");
    let registration_structure =
        read_runtime_src("core/runtime/tests/registration/structure/service_count_paths.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let lifecycle_doc = read_repo("docs/zircon_runtime/core/runtime/lifecycle.md");

    assert_contains_all(
        "CoreHandle registry poison recovery helpers",
        &core_handle,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_poison_recovered<T>(lock: &Mutex<T>) -> MutexGuard<'_, T>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "pub(crate) fn lock_modules(&self)",
            "pub(crate) fn lock_services(&self)",
            "pub fn replace_devtools_plugin_catalog_entries(",
            "pub(crate) fn lock_runtime_module_lifecycle_observer(",
            "core_handle_registry_accessors_recover_poisoned_runtime_locks",
        ],
    );
    assert_contains_all(
        "CoreHandle activation uses registry helpers",
        &activation,
        &["self.lock_modules()", "self.lock_services()"],
    );
    assert_contains_all(
        "CoreHandle registration uses registry helpers",
        &register_module,
        &[
            "let modules = self.lock_modules();",
            "let mut modules = self.lock_modules();",
            "let mut services = self.lock_services();",
        ],
    );
    assert_contains_all(
        "CoreHandle resolution uses registry helpers",
        &resolution,
        &[
            "let services = self.lock_services();",
            "let mut services = self.lock_services();",
            "let modules = self.lock_modules();",
        ],
    );
    assert_contains_all(
        "CoreHandle runtime extensions use registry helpers",
        &runtime_extensions,
        &[
            "*self.lock_runtime_module_lifecycle_observer() = Some(observer);",
            "self.lock_runtime_module_lifecycle_observer().take()",
            "self.lock_runtime_module_lifecycle_observer().clone()",
        ],
    );
    assert!(!runtime_extensions.contains("SceneRuntimeHook"));
    assert!(!runtime_extensions.contains("lock_scene_hooks"));
    assert_contains_all(
        "registration structure test tracks helper commit boundary",
        &registration_structure,
        &[
            ".rfind(\"let mut modules = self.lock_modules()\")",
            ".find(\"let modules = self.lock_modules();\")",
        ],
    );

    for (label, source) in [
        ("core handle root", core_handle.as_str()),
        ("core handle activation", activation.as_str()),
        ("core handle registration", register_module.as_str()),
        ("core handle resolution", resolution.as_str()),
        (
            "core handle runtime extensions",
            runtime_extensions.as_str(),
        ),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
        assert!(
            !production_section(source).contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }
}
