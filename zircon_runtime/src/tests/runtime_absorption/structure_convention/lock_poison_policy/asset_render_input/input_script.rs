use super::*;

#[test]
fn runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state() {
    let input_manager = read_runtime_src("input/runtime/default_input_manager.rs");
    let input_action_manager = read_runtime_src("input/runtime/default_input_action_manager.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let input_doc = read_repo("docs/zircon_runtime/input/input_state.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "DefaultInputManager state poison recovery",
        &input_manager,
        &[
            "use std::sync::{Mutex, MutexGuard};",
            "fn lock_state(&self) -> MutexGuard<'_, InputState>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let mut state = self.lock_state();",
            "let state = self.lock_state();",
            "input_manager_accessors_recover_poisoned_state_lock",
        ],
    );
    assert_contains_all(
        "DefaultInputActionManager evaluator poison recovery",
        &input_action_manager,
        &[
            "use std::sync::{Mutex, MutexGuard};",
            "fn lock_evaluator(&self) -> MutexGuard<'_, InputActionEvaluator>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_evaluator().action_map().clone()",
            "self.lock_evaluator().set_action_map(action_map);",
            "input_action_manager_accessors_recover_poisoned_evaluator_lock",
        ],
    );

    for (label, source) in [
        ("default input manager", input_manager.as_str()),
        (
            "default input action manager",
            input_action_manager.as_str(),
        ),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("input runtime doc", input_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 input runtime manager lock poison recovery",
                "runtime_15_input_runtime_manager_lock_poison_recovery_static_passed_cargo_deferred",
                "input/runtime/default_input_manager.rs",
                "input_manager_accessors_recover_poisoned_state_lock",
                "runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state",
            ],
        );
    }
}

#[test]
fn runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries() {
    let backend_registry = read_runtime_src("script/vm/backend/backend_registry.rs");
    let host_registry = read_runtime_src("script/vm/host/host_registry.rs");
    let host_export_registry = read_runtime_src("script/vm/host/host_export_registry.rs");
    let hot_reload_coordinator = read_runtime_src("script/vm/runtime/hot_reload_coordinator.rs");
    let hot_reload_coordinator_tests =
        read_runtime_src("script/vm/runtime/hot_reload_coordinator/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let script_vm_doc = read_repo("docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "VM backend registry poison recovery",
        &backend_registry,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_families(&self) -> MutexGuard<'_, BTreeMap<String, Arc<dyn VmBackendFamily>>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_families().insert(name.clone(), family);",
            "let families = self.lock_families()",
            "vm_backend_registry_accessors_recover_poisoned_family_lock",
        ],
    );
    assert_contains_all(
        "VM host registry poison recovery",
        &host_registry,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_state(&self) -> MutexGuard<'_, HostRegistryState>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let mut state = self.lock_state();",
            "let state = self.lock_state();",
            "host_registry_accessors_recover_poisoned_handle_lock",
        ],
    );
    assert_contains_all(
        "VM host export registry poison recovery",
        &host_export_registry,
        &[
            "use std::sync::{Arc, Mutex, MutexGuard};",
            "fn lock_modules(&self) -> MutexGuard<'_, HashMap<String, HostExportModuleEntry>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "let mut modules = self.lock_modules();",
            "let modules = self.lock_modules();",
            "host_export_registry_accessors_recover_poisoned_module_lock",
        ],
    );
    assert_contains_all(
        "VM hot reload coordinator poison recovery",
        &hot_reload_coordinator,
        &[
            "use std::sync::{Mutex, MutexGuard};",
            "fn lock_slots(&self) -> MutexGuard<'_, HashMap<PluginSlotId, PluginSlot>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            ".field(\"slot_count\", &self.lock_slots().len())",
            "let mut slots = self.lock_slots();",
        ],
    );
    assert_contains_all(
        "VM hot reload coordinator poison recovery test",
        &hot_reload_coordinator_tests,
        &[
            "hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
            "let _guard = coordinator.slots.lock().unwrap();",
            "assert!(coordinator.list_slots().is_empty());",
            ".load_package(\"policy-recording\", &backend, test_package(\"0.1.0\"), &host)",
        ],
    );

    for (label, source) in [
        ("VM backend registry", backend_registry.as_str()),
        ("VM host registry", host_registry.as_str()),
        ("VM host export registry", host_export_registry.as_str()),
        ("VM hot reload coordinator", hot_reload_coordinator.as_str()),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
        assert!(
            !production_section(source).contains("lock poisoned"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("script VM docs", script_vm_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 script VM registry lock poison recovery",
                "runtime_15_script_vm_registry_lock_poison_recovery_static_passed_cargo_deferred",
                "script/vm/backend/backend_registry.rs",
                "script/vm/host/host_export_registry.rs",
                "hot_reload_coordinator_accessors_recover_poisoned_slot_table_lock",
                "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries",
            ],
        );
    }
}

#[test]
fn runtime_15_zr_vm_real_backend_runtime_lock_poison_recovery_guard_covers_global_runtime_lock() {
    let runtime_lock =
        read_runtime_src("script/vm/backend/zr_vm_project_backend/real_backend/lock.rs");
    let package_loader =
        read_runtime_src("script/vm/backend/zr_vm_project_backend/real_backend/package.rs");
    let plugin_instance =
        read_runtime_src("script/vm/backend/zr_vm_project_backend/real_backend/instance.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let script_vm_doc = read_repo("docs/zircon_runtime/script/vm/zr_vm_project_backend.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "ZrVM real backend runtime lock poison recovery",
        &runtime_lock,
        &[
            "pub(super) fn acquire_zr_vm_lock() -> MutexGuard<'static, ()>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "zr_vm_real_backend_runtime_lock_recovers_after_poison",
        ],
    );
    assert_contains_all(
        "ZrVM package loader uses shared runtime lock",
        &package_loader,
        &[
            "use super::lock::acquire_zr_vm_lock;",
            "let _guard = acquire_zr_vm_lock();",
        ],
    );
    assert_contains_all(
        "ZrVM plugin instance lifecycle uses shared runtime lock",
        &plugin_instance,
        &[
            "use super::lock::acquire_zr_vm_lock;",
            "fn activate(&mut self, _host: &VmPluginHostContext) -> Result<(), VmError>",
            "fn call_export(",
            "let _guard = acquire_zr_vm_lock();",
        ],
    );

    for (label, source) in [
        ("ZrVM real backend runtime lock", runtime_lock.as_str()),
        ("ZrVM package loader", package_loader.as_str()),
        ("ZrVM plugin instance", plugin_instance.as_str()),
    ] {
        assert_no_direct_lock_unwrap_in_production(label, source);
        assert!(
            !production_section(source).contains("lock should not be poisoned")
                && !production_section(source).contains(".expect(\"zr_vm runtime lock"),
            "{label} production code should recover poisoned locks instead of panicking"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("ZrVM project backend doc", script_vm_doc.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 ZrVM real backend runtime lock poison recovery",
                "runtime_15_zr_vm_real_backend_runtime_lock_poison_recovery_static_passed_cargo_timeout_no_result",
                "script/vm/backend/zr_vm_project_backend/real_backend/lock.rs",
                "script/vm/backend/zr_vm_project_backend/real_backend/package.rs",
                "zr_vm_real_backend_runtime_lock_recovers_after_poison",
                "runtime_15_zr_vm_real_backend_runtime_lock_poison_recovery_guard_covers_global_runtime_lock",
            ],
        );
    }
}
