use std::collections::BTreeSet;
use std::path::Path;

use super::inventory::{
    BEHAVIOR_TEST_ANCHORS, DIAGNOSTIC_ANCHORS, EXPECTED_DIRECT_RAYON_PATHS,
    EXPECTED_JOB_SYSTEM_MODULES, FORBIDDEN_LEGACY_DIAGNOSTIC_ANCHORS,
    FORBIDDEN_SCHEDULE_EXECUTOR_RAYON_ANCHORS, JOB_HANDLE_ANCHORS, JOB_SCHEDULER_ANCHORS,
    JOB_SYSTEM_MODULE_MAX_LINES, MIRROR_DOC_ANCHORS, PARALLEL_FOR_ANCHORS, REPORT_ANCHORS,
    SCHEDULE_EXECUTOR_ANCHORS, TASKS_MOD_DECLARATIONS, TASKS_MOD_PUBLIC_ANCHORS, TIMER_ANCHORS,
};
use super::source_helpers::{collect_direct_rayon_paths, line_count};

#[test]
fn runtime_11_job_system_mirror_docs_match_structure_audit_counts() {
    let runtime_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tasks_dir = runtime_root
        .join("src")
        .join("core")
        .join("runtime")
        .join("tasks");

    for module in EXPECTED_JOB_SYSTEM_MODULES {
        let path = tasks_dir.join(format!("{module}.rs"));
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

    let tasks_mod = include_str!("../../../core/runtime/tasks/mod.rs");
    assert_contains_all(
        "core/runtime/tasks/mod.rs declarations",
        tasks_mod,
        TASKS_MOD_DECLARATIONS,
    );
    assert_contains_all(
        "core/runtime/tasks/mod.rs public surface",
        tasks_mod,
        TASKS_MOD_PUBLIC_ANCHORS,
    );

    let job_scheduler = include_str!("../../../core/runtime/tasks/job_scheduler.rs");
    assert_contains_all("JobScheduler", job_scheduler, JOB_SCHEDULER_ANCHORS);

    let job_handle = include_str!("../../../core/runtime/tasks/job_handle.rs");
    assert_contains_all("JobHandle", job_handle, JOB_HANDLE_ANCHORS);

    let parallel_for = include_str!("../../../core/runtime/tasks/parallel_for.rs");
    assert_contains_all("parallel_for primitive", parallel_for, PARALLEL_FOR_ANCHORS);

    let diagnostics = include_str!("../../../core/runtime/tasks/diagnostics.rs");
    assert_contains_all("JobSystem diagnostics", diagnostics, DIAGNOSTIC_ANCHORS);

    let report = include_str!("../../../core/runtime/tasks/report.rs");
    assert_contains_all("JobSystem report", report, REPORT_ANCHORS);

    let timer = include_str!("../../../core/runtime/tasks/timer.rs");
    assert_contains_all("TaskTimer", timer, TIMER_ANCHORS);

    for legacy_anchor in FORBIDDEN_LEGACY_DIAGNOSTIC_ANCHORS {
        for (owner_name, owner_source) in [
            ("tasks mod", tasks_mod),
            ("JobScheduler", job_scheduler),
            ("JobHandle", job_handle),
            ("JobSystem diagnostics", diagnostics),
            ("JobSystem report", report),
        ] {
            assert!(
                !owner_source.contains(legacy_anchor),
                "{owner_name} must not retain retired task diagnostic anchor `{legacy_anchor}`"
            );
        }
    }

    let tasks_tests = include_str!("../../tasks.rs");
    assert_sources_contain_all(
        "Runtime 11 task behavior tests",
        &[tasks_tests, job_handle],
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
            include_str!("../../../../../docs/zircon_runtime/core/job_system.md"),
        ),
        (
            "Runtime 11 plan",
            include_str!(
                "../../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
            ),
        ),
        (
            "interface convergence",
            include_str!(
                "../../../../../docs/engine-architecture/runtime-interface-convergence.md"
            ),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        assert_contains_all(doc_name, doc_source, MIRROR_DOC_ANCHORS);
    }
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
