pub(crate) const EXPECTED_DIRECT_RAYON_PATHS: &[&str] = &[
    "src/core/runtime/tasks/parallel_for.rs",
    "src/core/runtime/tasks/pool.rs",
];

pub(crate) const FORBIDDEN_LEGACY_DIAGNOSTIC_ANCHORS: &[&str] = &[
    "TASKS_MAIN_THREAD_WAIT_MS_DIAGNOSTIC",
    "tasks.main_thread_wait_ms",
    "main_thread_wait_ms",
    "record_main_thread_wait",
];

pub(crate) const SCHEDULE_EXECUTOR_ANCHORS: &[&str] = &[
    "JobHandle::completed()",
    ".schedule_after(",
    "run_parallel_tasks(",
    "scheduler.join(",
];

pub(crate) const FORBIDDEN_SCHEDULE_EXECUTOR_RAYON_ANCHORS: &[&str] = &[
    "use rayon",
    "rayon::",
    ".par_iter(",
    ".par_chunks",
    ".into_par_iter(",
];

pub(crate) const MIRROR_DOC_ANCHORS: &[&str] = &[
    "job_system_boundary",
    "expected_module_count = 22",
    "behavior_test_anchor_count = 73",
    "tasks/task_graph/",
    "tasks/bounded_stream_io/",
    "tasks/retained_byte_budget.rs",
    "TaskGraphWorkerInventory",
    "runtime_11_job_system_mirror_docs_match_structure_audit_counts",
];
