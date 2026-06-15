use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const JOB_SYSTEM_MODULE_MAX_LINES: usize = 500;
const EXPECTED_JOB_SYSTEM_MODULES: &[&str] = &[
    "diagnostics",
    "job_handle",
    "job_scheduler",
    "mod",
    "parallel_for",
    "pool",
    "pools",
    "report",
    "thread_assignment",
];
const EXPECTED_DIRECT_RAYON_PATHS: &[&str] = &[
    "src/core/runtime/tasks/parallel_for.rs",
    "src/core/runtime/tasks/pool.rs",
    "src/graphics/visibility/culling/parallel_frustum.rs",
];

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

    let tasks_mod = include_str!("../../core/runtime/tasks/mod.rs");
    for declaration in [
        "mod diagnostics;",
        "mod job_handle;",
        "mod job_scheduler;",
        "mod parallel_for;",
        "mod pool;",
        "mod pools;",
        "mod report;",
        "mod thread_assignment;",
    ] {
        assert!(
            tasks_mod.contains(declaration),
            "core/runtime/tasks/mod.rs should retain JobSystem owner declaration `{declaration}`"
        );
    }
    for public_anchor in [
        "pub use diagnostics::{",
        "pub use job_handle::JobHandle;",
        "pub use job_scheduler::JobScheduler;",
        "pub use parallel_for::parallel_for;",
        "pub use report::{JobSchedulerReport, TaskPoolReport, TaskPoolReportEntry};",
        "TASKS_SCHEDULED_DIAGNOSTIC",
        "TASKS_COMPLETED_DIAGNOSTIC",
        "TASKS_DEPENDENCY_WAIT_MS_DIAGNOSTIC",
        "TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC",
    ] {
        assert!(
            tasks_mod.contains(public_anchor),
            "core/runtime/tasks/mod.rs should retain public JobSystem surface `{public_anchor}`"
        );
    }

    let job_scheduler = include_str!("../../core/runtime/tasks/job_scheduler.rs");
    for scheduler_anchor in [
        "pub fn schedule(&self",
        "pub fn schedule_after(",
        "PendingScheduledJob",
        "record_dependency_wait",
    ] {
        assert!(
            job_scheduler.contains(scheduler_anchor),
            "JobScheduler should retain dependency scheduling anchor `{scheduler_anchor}`"
        );
    }

    let job_handle = include_str!("../../core/runtime/tasks/job_handle.rs");
    for handle_anchor in [
        "pub fn combine(handles: &[JobHandle])",
        "pub fn completed() -> Self",
        "pub fn wait(&self)",
        "Condvar",
    ] {
        assert!(
            job_handle.contains(handle_anchor),
            "JobHandle should retain completion/synchronization anchor `{handle_anchor}`"
        );
    }

    let parallel_for = include_str!("../../core/runtime/tasks/parallel_for.rs");
    for parallel_anchor in ["pub fn parallel_for", "pool.install", "par_chunks_mut"] {
        assert!(
            parallel_for.contains(parallel_anchor),
            "parallel_for primitive should retain Rayon-owned task-pool anchor `{parallel_anchor}`"
        );
    }

    let diagnostics = include_str!("../../core/runtime/tasks/diagnostics.rs");
    for diagnostic_anchor in [
        "\"tasks.scheduled\"",
        "\"tasks.completed\"",
        "\"tasks.dependency_wait_ms\"",
        "\"tasks.main_thread_wait_ms\"",
    ] {
        assert!(
            diagnostics.contains(diagnostic_anchor),
            "JobSystem diagnostics should retain counter anchor `{diagnostic_anchor}`"
        );
    }

    let report = include_str!("../../core/runtime/tasks/report.rs");
    for report_anchor in ["pub struct JobSchedulerReport", "record_diagnostics"] {
        assert!(
            report.contains(report_anchor),
            "JobSystem report should retain anchor `{report_anchor}`"
        );
    }

    let tasks_tests = include_str!("../tasks.rs");
    for behavior_test_anchor in [
        "job_handle_wait_blocks_until_task_completes",
        "schedule_after_runs_task_only_after_all_dependencies",
        "combined_handle_completes_when_all_children_complete",
        "schedule_after_does_not_consume_worker_while_waiting_on_dependencies",
        "job_diagnostics_track_schedule_complete_and_wait_times",
        "deep_dependency_chain_completes_in_order",
        "wide_fanout_combine_waits_for_all",
        "parallel_for_visits_every_item_exactly_once",
        "parallel_for_chunk_size_bounds_task_granularity",
    ] {
        assert!(
            tasks_tests.contains(behavior_test_anchor),
            "zircon_runtime/src/tests/tasks.rs should retain Runtime 11 behavior test anchor `{behavior_test_anchor}`"
        );
    }

    let schedule_executor = include_str!("../../scene/ecs/schedule_parallel_executor.rs");
    for executor_anchor in [
        "JobHandle::completed()",
        ".schedule_after(",
        "run_parallel_tasks(",
        "scheduler.join(",
    ] {
        assert!(
            schedule_executor.contains(executor_anchor),
            "ScheduleParallelExecutor should retain JobSystem consumption anchor `{executor_anchor}`"
        );
    }
    for forbidden_rayon_anchor in [
        "use rayon",
        "rayon::",
        ".par_iter(",
        ".par_chunks",
        ".into_par_iter(",
    ] {
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
        "Runtime 11 direct-Rayon whitelist should match job_system_boundary; only task primitives plus the render-owned parallel_frustum exception are allowed"
    );

    let mirror_docs = [
        (
            "JobSystem module doc",
            include_str!("../../../../docs/zircon_runtime/core/job_system.md"),
        ),
        (
            "Runtime 11 plan",
            include_str!(
                "../../../../docs/plans/zircon_runtime/runtime/11-job-system-task-model.md"
            ),
        ),
        (
            "runtime index",
            include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "M0 review",
            include_str!("../../../../docs/engine-architecture/runtime-architecture-review-m0.md"),
        ),
        (
            "interface convergence",
            include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md"),
        ),
    ];

    for (doc_name, doc_source) in mirror_docs {
        for required_anchor in [
            "job_system_boundary",
            "expected_module_count = 9",
            "direct_rayon_paths = 3",
            "schedule_parallel_executor_direct_rayon = []",
            "diagnostic_anchor_count = 4",
            "behavior_test_anchor_count = 9",
            "missing_behavior_test_anchors = []",
            "oversized_modules = []",
            "mirror_docs_guard_present = true",
            "risks = []",
            "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
        ] {
            assert!(
                doc_source.contains(required_anchor),
                "{doc_name} should mirror Runtime 11 JobSystem audit anchor `{required_anchor}`"
            );
        }
    }
}

fn line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        .lines()
        .count()
}

fn collect_direct_rayon_paths(source_root: &Path) -> BTreeSet<String> {
    let runtime_root = source_root
        .parent()
        .expect("runtime source root should have manifest parent");
    rust_source_files(source_root)
        .into_iter()
        .filter(|path| !is_test_path(&relative_path(runtime_root, path)))
        .filter(|path| {
            fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .lines()
                .any(line_mentions_rayon)
        })
        .map(|path| relative_path(runtime_root, &path))
        .collect()
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("runtime source directory should be readable") {
        let entry = entry.expect("runtime source entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn line_mentions_rayon(line: &str) -> bool {
    line.contains("use rayon")
        || line.contains("rayon::")
        || line.contains(".par_iter(")
        || line.contains(".par_chunks")
        || line.contains(".into_par_iter(")
}

fn is_test_path(relative_path: &str) -> bool {
    let file_name = relative_path.rsplit('/').next().unwrap_or(relative_path);
    relative_path.split('/').any(|part| part == "tests")
        || file_name == "tests.rs"
        || file_name.ends_with("_tests.rs")
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
