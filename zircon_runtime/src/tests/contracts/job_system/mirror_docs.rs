use std::collections::BTreeSet;
use std::path::Path;

use super::inventory::{
    BEHAVIOR_TEST_ANCHORS, EXPECTED_DIRECT_RAYON_PATHS, EXPECTED_JOB_SYSTEM_MODULES,
    FORBIDDEN_SCHEDULE_EXECUTOR_RAYON_ANCHORS, JOB_SYSTEM_MODULE_MAX_LINES, MIRROR_DOC_ANCHORS,
    SCHEDULE_EXECUTOR_ANCHORS,
};
use super::source_helpers::{collect_direct_rayon_paths, line_count};

#[path = "owner_contracts.rs"]
mod owner_contracts;

#[test]
fn runtime_11_job_system_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tasks_dir = runtime_root
        .join("src")
        .join("core")
        .join("runtime")
        .join("tasks");

    for module in EXPECTED_JOB_SYSTEM_MODULES {
        let path = tasks_dir.join(module);
        assert!(
            path.exists(),
            "Runtime 11 JobSystem owner module `{module}` is missing; update job_system_boundary before changing the task owner folder"
        );
        let line_count = line_count(&path);
        assert!(
            line_count <= JOB_SYSTEM_MODULE_MAX_LINES,
            "Runtime 11 JobSystem owner module `{module}` has {line_count} lines, exceeding the {JOB_SYSTEM_MODULE_MAX_LINES}-line owner budget"
        );
    }

    let (job_scheduler_tests, task_graph) = owner_contracts::assert_task_owner_contracts();

    let tasks_tests = include_str!("../../tasks.rs");
    let task_diagnostics_tests = include_str!("../../tasks/diagnostics.rs");
    let task_pool_inventory_tests = include_str!("../../tasks/pools.rs");
    let task_terminal_observer_tests = include_str!("../../tasks/terminal_observers.rs");
    let task_thread_budget_tests = include_str!("../../tasks/thread_budget.rs");
    let job_handle_tests = include_str!("../../../core/runtime/tasks/job_handle/tests/cases.rs");
    let diagnostics_tests = include_str!("../../../core/runtime/tasks/diagnostics/tests/cases.rs");
    let diagnostic_observation_tests =
        include_str!("../../../core/runtime/tasks/diagnostic_observation/tests/cases.rs");
    let task_graph_scope_tests =
        include_str!("../../../core/runtime/tasks/task_graph/scope/tests/cases.rs");
    let task_graph_scope_dependency_scale_tests =
        include_str!("../../../core/runtime/tasks/task_graph/scope/tests/dependency_scale.rs");
    let task_graph_scope_ownership_tests =
        include_str!("../../../core/runtime/tasks/task_graph/scope/tests/ownership.rs");
    let task_graph_engine_tests =
        include_str!("../../../core/runtime/tasks/task_graph/tests/cases.rs");
    let task_pool_tests = include_str!("../../../core/runtime/tasks/pool/tests/cases.rs");
    let bounded_stream_io_tests =
        include_str!("../../../core/runtime/tasks/bounded_stream_io/tests/cases.rs");
    let retained_byte_budget_tests =
        include_str!("../../../core/runtime/tasks/retained_byte_budget/tests/cases.rs");
    let dynamic_scene_spawn = include_str!("../../../scene/dynamic/spawn_task/loader.rs");
    assert!(dynamic_scene_spawn.contains("#[path = \"tests/loader.rs\"]"));
    assert!(dynamic_scene_spawn.contains("mod tests;"));
    let dynamic_scene_spawn_tests =
        include_str!("../../../scene/dynamic/spawn_task/tests/loader.rs");
    let level_manager_project_io =
        include_str!("../../../scene/module/level_manager_project_io.rs");
    assert!(level_manager_project_io
        .replace("\r\n", "\n")
        .contains("#[cfg(test)]\n#[path = \"tests/level_manager_project_io.rs\"]\nmod tests;"));
    let level_manager_project_io_tests =
        include_str!("../../../scene/module/tests/level_manager_project_io.rs");
    assert!(level_manager_project_io_tests.contains(
        "#[test]\nfn standalone_level_manager_rejects_artifact_io_without_an_implicit_process_owner()"
    ));
    assert!(level_manager_project_io_tests
        .contains("assert!(matches!(error, SceneProjectError::RuntimeUnavailable));"));
    assert_sources_contain_all(
        "Runtime 11 task behavior tests",
        &[
            bounded_stream_io_tests,
            retained_byte_budget_tests,
            tasks_tests,
            task_diagnostics_tests,
            task_pool_inventory_tests,
            task_terminal_observer_tests,
            task_thread_budget_tests,
            job_handle_tests,
            job_scheduler_tests,
            task_pool_tests,
            diagnostics_tests,
            diagnostic_observation_tests,
            task_graph,
            task_graph_engine_tests,
            task_graph_scope_tests,
            task_graph_scope_dependency_scale_tests,
            task_graph_scope_ownership_tests,
            dynamic_scene_spawn_tests,
            level_manager_project_io_tests,
        ],
        BEHAVIOR_TEST_ANCHORS,
    );

    let schedule_executor = include_str!("../../../scene/ecs/schedule_parallel_executor.rs");
    assert_contains_all(
        "ScheduleParallelExecutor",
        schedule_executor,
        SCHEDULE_EXECUTOR_ANCHORS,
    );
    for forbidden_rayon_anchor in FORBIDDEN_SCHEDULE_EXECUTOR_RAYON_ANCHORS {
        assert!(
            !schedule_executor.contains(forbidden_rayon_anchor),
            "ScheduleParallelExecutor should not reintroduce direct Rayon anchor `{forbidden_rayon_anchor}`"
        );
    }

    let direct_rayon_paths = collect_direct_rayon_paths(&runtime_root.join("src"));
    let expected_direct_rayon_paths = EXPECTED_DIRECT_RAYON_PATHS
        .iter()
        .copied()
        .map(String::from)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        direct_rayon_paths, expected_direct_rayon_paths,
        "Runtime 11 direct-Rayon whitelist should match job_system_boundary; only task primitives are allowed"
    );

    let mirror_docs = [
        (
            "JobSystem module doc",
            include_str!("../../../../../docs/crates/zircon_runtime/core/job_system.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../../docs/architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../../docs/architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        assert_contains_all(doc_name, doc_source, MIRROR_DOC_ANCHORS);
    }
    let python_inventory = include_str!("../../../../../.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py");
    let python_behavior = python_inventory
        .split_once("JOB_SYSTEM_BEHAVIOR_TEST_ANCHORS = (")
        .expect("current Python behavior inventory must retain its typed owner")
        .1
        .split_once("\n)")
        .expect("the Python behavior inventory must be delimited")
        .0;
    let python_anchors: Vec<_> = python_behavior
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix('"')
                .and_then(|line| line.strip_suffix("\","))
        })
        .collect();
    assert_eq!(python_anchors.len(), 77);
    assert_eq!(
        python_anchors
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        77
    );
    assert_eq!(BEHAVIOR_TEST_ANCHORS.len(), 74);
    for anchor in BEHAVIOR_TEST_ANCHORS {
        assert!(python_anchors.contains(anchor), "{anchor}");
    }
    let additional_python_anchors = [
        "canonical_wait_all_observes_every_terminal_before_propagating_failure",
        "direct_submit_after_propagates_dependency_failure_without_running_user_work",
        "direct_submit_after_reuses_graph_owner_without_scheduler_injection",
    ];
    for anchor in additional_python_anchors {
        assert!(python_anchors.contains(&anchor), "{anchor}");
        assert!(!BEHAVIOR_TEST_ANCHORS.contains(&anchor), "{anchor}");
    }
    assert_sources_contain_all(
        "additional Python inventory regressions in the mounted scope owners",
        &[
            task_graph_scope_tests,
            task_graph_scope_dependency_scale_tests,
        ],
        &additional_python_anchors,
    );
    let worklist = include_str!(
        "../../../../../docs/testing-and-validation/runtime-task-validation-worklist.md"
    );
    assert_contains_all(
        "task lifecycle worklist",
        worklist,
        &[
            "EngineTaskGraph owns budgeted Io/AsyncCompute/Compute pools",
            "TaskGraphScope owns admission, scope drain and close/join",
            "removed default/process_io constructors must not return",
            "worker-join and power acceptance remains pending",
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
            "../crates/zircon_runtime/core/job_system.md",
        ],
    );
    let index = include_str!(
        "../../../../../docs/crates/zircon_runtime/structure/runtime-absorption-evidence.md"
    );
    assert_contains_all(
        "runtime index navigation",
        index,
        &[
            "| 11 | Explicit task pool/lifecycle, scope, stream and result budgets |",
            "not a fresh whole-engine result",
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
            "../core/job_system.md",
        ],
    );
    let historical = crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source(
        "docs/plans/zircon_runtime/runtime/11/2026-07-09-job-system-task-model-output-records.md"
    );
    assert_contains_all(
        "fixed Runtime 11 history",
        historical,
        &[
            "job_system_boundary",
            "behavior_test_anchor_count = 12",
            "behavior_test_anchor_count = 13",
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
            "cargo_pending",
        ],
    );
}

fn assert_contains_all(label: &str, source: &str, required: &[&str]) {
    for anchor in required {
        assert!(
            source.contains(anchor),
            "{label} should contain Runtime 11 JobSystem audit anchor `{anchor}`"
        );
    }
}

fn assert_sources_contain_all(label: &str, sources: &[&str], required: &[&str]) {
    for anchor in required {
        assert!(
            sources.iter().any(|source| source.contains(anchor)),
            "{label} should contain Runtime 11 JobSystem audit anchor `{anchor}`"
        );
    }
}
