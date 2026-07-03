use super::{assert_contains_all, repo_path, runtime_src_path};

const LOCK_UNWRAP_CALL: &str = concat!(".lock().", "unwrap()");

#[test]
fn runtime_15_native_live_host_bridge_methods_lock_poison_recovery_guard_covers_binding_registry() {
    let bridge_methods =
        read_runtime_src("plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs");
    let structure_parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let plugin_bridge_doc = read_repo("docs/zircon_runtime/plugin/bridge.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    assert_contains_all(
        "native live host bridge methods poison recovery",
        &bridge_methods,
        &[
            "use std::collections::BTreeMap;",
            "use std::sync::MutexGuard;",
            "fn lock_runtime_bridge_method_bindings(",
            "MutexGuard<'_, BTreeMap<String, Vec<NativeBridgeMethodBinding>>>",
            ".unwrap_or_else(|poisoned| poisoned.into_inner())",
            "self.lock_runtime_bridge_method_bindings()",
            "native_live_host_bridge_method_bindings_recover_poisoned_lock",
        ],
    );
    assert_contains_all(
        "native live host lock poison guard mount",
        &structure_parent,
        &[
            "#[path = \"structure_convention/native_live_host_lock_poison.rs\"]",
            "mod native_live_host_lock_poison;",
        ],
    );
    assert_no_direct_lock_unwrap_in_production("native live host bridge methods", &bridge_methods);
    assert!(
        !production_section(&bridge_methods).contains("lock poisoned"),
        "native live host bridge method production code should recover poisoned binding locks"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("plugin bridge doc", plugin_bridge_doc.as_str()),
        (
            "status-output M3 lock-poison row data",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 native live-host bridge methods lock poison recovery",
                "runtime_15_native_live_host_bridge_methods_lock_poison_recovery_static_passed_cargo_deferred",
                "plugin/native_plugin_loader/native_plugin_live_host/bridge_methods.rs",
                "native_live_host_bridge_method_bindings_recover_poisoned_lock",
                "runtime_15_native_live_host_bridge_methods_lock_poison_recovery_guard_covers_binding_registry",
            ],
        );
    }
}

fn assert_no_direct_lock_unwrap_in_production(label: &str, source: &str) {
    let production = production_section(source);
    assert!(
        !production.contains(LOCK_UNWRAP_CALL),
        "{label} production code should use poison-safe lock helpers instead of {LOCK_UNWRAP_CALL}"
    );
}

fn production_section(source: &str) -> &str {
    source.split("\n#[cfg(test)]").next().unwrap_or(source)
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
