use super::{assert_contains_all, repo_path, runtime_src_path};

const LOCK_UNWRAP_CALL: &str = concat!(".lock()", ".unwrap()");
const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");

#[path = "lock_poison_policy/asset_render_input.rs"]
mod asset_render_input;
#[path = "lock_poison_policy/core_runtime.rs"]
mod core_runtime;
#[path = "lock_poison_policy/runtime_services.rs"]
mod runtime_services;

#[test]
fn runtime_15_lock_poison_policy_guard_is_folder_backed() {
    let parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/lock_poison_policy.rs");
    let core_runtime = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs",
    );
    let core_runtime_config_devtools = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/config_devtools.rs",
    );
    let core_runtime_global_gate = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/global_gate.rs",
    );
    let core_runtime_handle_accessors = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
    );
    let core_runtime_scene_eventbus = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/scene_eventbus.rs",
    );
    let core_runtime_task_profiling = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/task_profiling.rs",
    );
    let runtime_services = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/runtime_services.rs",
    );
    let asset_render_input = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/asset_render_input.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "lock poison policy parent mounts child owners",
        &parent,
        &[
            "mod asset_render_input;",
            "mod core_runtime;",
            "mod runtime_services;",
        ],
    );

    for moved_guard in [
        concat!(
            "fn ",
            "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus"
        ),
        concat!(
            "fn ",
            "runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors"
        ),
        concat!(
            "fn ",
            "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot"
        ),
        concat!(
            "fn ",
            "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager"
        ),
        concat!(
            "fn ",
            "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager"
        ),
        concat!(
            "fn ",
            "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "lock poison policy parent should mount child owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "core runtime lock poison child owns core guards",
        &core_runtime,
        &[
            "mod config_devtools;",
            "mod global_gate;",
            "mod handle_accessors;",
            "mod scene_eventbus;",
            "mod task_profiling;",
        ],
    );
    let core_runtime_children = format!(
        "{}\n{}\n{}\n{}\n{}",
        core_runtime_config_devtools,
        core_runtime_global_gate,
        core_runtime_handle_accessors,
        core_runtime_scene_eventbus,
        core_runtime_task_profiling
    );
    assert_contains_all(
        "core runtime lock poison children preserve core guards",
        &core_runtime_children,
        &[
            concat!(
                "fn ",
                "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus"
            ),
            concat!(
                "fn ",
                "runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors"
            ),
        ],
    );
    assert_contains_all(
        "runtime services lock poison child owns plugin scene resource guards",
        &runtime_services,
        &[
            "use super::*;",
            concat!(
                "fn ",
                "runtime_15_plugin_bridge_table_lock_poison_recovery_guard_covers_provider_slot"
            ),
            concat!(
                "fn ",
                "runtime_15_core_resource_manager_lock_poison_recovery_guard_covers_resource_manager"
            ),
        ],
    );
    assert_contains_all(
        "asset render input lock poison child owns asset graphics input guards",
        &asset_render_input,
        &[
            "use super::*;",
            concat!(
                "fn ",
                "runtime_15_asset_project_manager_lock_poison_recovery_guard_covers_project_asset_manager"
            ),
            concat!(
                "fn ",
                "runtime_15_script_vm_registry_lock_poison_recovery_guard_covers_vm_registries"
            ),
            concat!(
                "fn ",
                "runtime_15_zr_vm_real_backend_runtime_lock_poison_recovery_guard_covers_global_runtime_lock"
            ),
        ],
    );

    assert_eq!(
        parent.matches(TEST_ATTRIBUTE).count()
            + core_runtime.matches(TEST_ATTRIBUTE).count()
            + core_runtime_config_devtools
                .matches(TEST_ATTRIBUTE)
                .count()
            + core_runtime_global_gate.matches(TEST_ATTRIBUTE).count()
            + core_runtime_handle_accessors
                .matches(TEST_ATTRIBUTE)
                .count()
            + core_runtime_scene_eventbus.matches(TEST_ATTRIBUTE).count()
            + core_runtime_task_profiling
                .matches(TEST_ATTRIBUTE)
                .count()
            + runtime_services.matches(TEST_ATTRIBUTE).count()
            + asset_render_input.matches(TEST_ATTRIBUTE).count(),
        25,
        "lock poison policy parent plus split children should preserve 21 original guards plus the production global gate, the ZrVM runtime lock guard, and two layout guards"
    );

    for (path, source) in [
        (
            "structure_convention/lock_poison_policy.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime.rs",
            core_runtime.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/config_devtools.rs",
            core_runtime_config_devtools.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/global_gate.rs",
            core_runtime_global_gate.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
            core_runtime_handle_accessors.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/scene_eventbus.rs",
            core_runtime_scene_eventbus.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/task_profiling.rs",
            core_runtime_task_profiling.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/runtime_services.rs",
            runtime_services.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/asset_render_input.rs",
            asset_render_input.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 lock poison policy guard folder split",
                "runtime_15_lock_poison_policy_guard_folder_split_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy.rs",
                "structure_convention/lock_poison_policy/core_runtime.rs",
                "structure_convention/lock_poison_policy/runtime_services.rs",
                "structure_convention/lock_poison_policy/asset_render_input.rs",
                "runtime_15_lock_poison_policy_guard_is_folder_backed",
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
