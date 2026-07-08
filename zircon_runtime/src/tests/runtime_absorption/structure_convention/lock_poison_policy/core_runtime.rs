use super::support::*;

#[path = "core_runtime/config_devtools.rs"]
mod config_devtools;
#[path = "core_runtime/global_gate.rs"]
mod global_gate;
#[path = "core_runtime/handle_accessors.rs"]
mod handle_accessors;
#[path = "core_runtime/scene_eventbus.rs"]
mod scene_eventbus;
#[path = "core_runtime/task_profiling.rs"]
mod task_profiling;

const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");

#[test]
fn runtime_15_core_runtime_lock_poison_guard_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime.rs",
    );
    let config_devtools = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/config_devtools.rs",
    );
    let global_gate = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/global_gate.rs",
    );
    let handle_accessors = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
    );
    let scene_eventbus = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/scene_eventbus.rs",
    );
    let task_profiling = read_runtime_src(
        "tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/task_profiling.rs",
    );

    assert_contains_all(
        "core runtime lock-poison parent mounts child owners",
        &parent,
        &[
            "mod config_devtools;",
            "mod global_gate;",
            "mod handle_accessors;",
            "mod scene_eventbus;",
            "mod task_profiling;",
        ],
    );

    for moved_guard in [
        concat!(
            "fn ",
            "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus"
        ),
        concat!(
            "fn ",
            "runtime_15_production_sources_do_not_directly_unwrap_mutex_locks"
        ),
        concat!(
            "fn ",
            "runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store"
        ),
        concat!(
            "fn ",
            "runtime_15_core_runtime_devtools_lock_poison_recovery_guard_covers_devtools_snapshot"
        ),
        concat!(
            "fn ",
            "runtime_15_core_handle_diagnostics_lock_poison_recovery_guard_covers_diagnostic_store"
        ),
        concat!(
            "fn ",
            "runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles"
        ),
    ] {
        assert!(
            !parent.contains(moved_guard),
            "core_runtime.rs should mount child owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "scene/eventbus child owns F2 scene and event bus guard",
        &scene_eventbus,
        &[concat!(
            "fn ",
            "runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus"
        )],
    );
    assert_contains_all(
        "global gate child owns production lock unwrap scan",
        &global_gate,
        &[
            concat!(
                "fn ",
                "runtime_15_production_sources_do_not_directly_unwrap_mutex_locks"
            ),
            "fn collect_runtime_rust_sources(",
        ],
    );
    assert_contains_all(
        "config/devtools child owns runtime config and devtools guards",
        &config_devtools,
        &[
            concat!(
                "fn ",
                "runtime_15_config_store_lock_poison_recovery_guard_covers_runtime_config_store"
            ),
            concat!(
                "fn ",
                "runtime_15_core_runtime_devtools_lock_poison_recovery_guard_covers_devtools_snapshot"
            ),
        ],
    );
    assert_contains_all(
        "handle accessors child owns CoreHandle lock guards",
        &handle_accessors,
        &[
            concat!(
                "fn ",
                "runtime_15_core_handle_diagnostics_lock_poison_recovery_guard_covers_diagnostic_store"
            ),
            concat!(
                "fn ",
                "runtime_15_core_handle_time_lock_poison_recovery_guard_covers_runtime_clocks"
            ),
            concat!(
                "fn ",
                "runtime_15_core_handle_states_lock_poison_recovery_guard_covers_state_registry"
            ),
            concat!(
                "fn ",
                "runtime_15_core_handle_registry_lock_poison_recovery_guard_covers_registry_accessors"
            ),
        ],
    );
    assert_contains_all(
        "task/profiling child owns job and profiler lock guards",
        &task_profiling,
        &[
            concat!(
                "fn ",
                "runtime_15_core_runtime_task_lock_poison_recovery_guard_covers_job_handles"
            ),
            concat!(
                "fn ",
                "runtime_15_core_runtime_profiling_lock_poison_recovery_guard_covers_global_recorder"
            ),
        ],
    );

    let child_test_total = [
        parent.as_str(),
        config_devtools.as_str(),
        global_gate.as_str(),
        handle_accessors.as_str(),
        scene_eventbus.as_str(),
        task_profiling.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 11,
        "core runtime lock-poison children should preserve 10 existing guards plus the new split guard"
    );

    for (path, source) in [
        (
            "structure_convention/lock_poison_policy/core_runtime.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/config_devtools.rs",
            config_devtools.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/global_gate.rs",
            global_gate.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
            handle_accessors.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/scene_eventbus.rs",
            scene_eventbus.as_str(),
        ),
        (
            "structure_convention/lock_poison_policy/core_runtime/task_profiling.rs",
            task_profiling.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused child-owner budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output M3 foundation row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 core runtime lock poison guard child-owner split",
                "runtime_15_core_runtime_lock_poison_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/lock_poison_policy/core_runtime.rs",
                "structure_convention/lock_poison_policy/core_runtime/handle_accessors.rs",
                "runtime_15_core_runtime_lock_poison_guard_child_owner_split",
            ],
        );
    }
}
