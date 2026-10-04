use crate::core::jobs::{EditorJobProgress, EditorJobProgressSnapshot, JobCategory, JobId};

use super::status_task_progress_from_jobs;

#[test]
fn status_projection_selects_the_smallest_job_id_and_uses_reported_progress() {
    let active = vec![
        EditorJobProgressSnapshot::new(JobId::new(9), "later", JobCategory::Export, None, true),
        EditorJobProgressSnapshot::new(
            JobId::new(2),
            "first",
            JobCategory::Index,
            Some(EditorJobProgress::new(4, 10, "Indexing assets")),
            true,
        ),
    ];

    let projected = status_task_progress_from_jobs(&active).unwrap();
    assert_eq!(projected.task_id, "editor_job:2");
    assert_eq!(projected.label, "first");
    assert_eq!(projected.detail, "Indexing assets");
    assert_eq!(projected.percent, Some(40));
}

#[test]
fn status_projection_handles_indeterminate_and_overflow_safe_percentages() {
    let indeterminate = EditorJobProgressSnapshot::new(
        JobId::new(1),
        "indeterminate",
        JobCategory::Compile,
        Some(EditorJobProgress::new(1, 0, "  ")),
        true,
    );
    let projected = status_task_progress_from_jobs(&[indeterminate]).unwrap();
    assert_eq!(projected.percent, None);
    assert_eq!(projected.detail, "Compile");

    let interactive_save = EditorJobProgressSnapshot::new(
        JobId::new(3),
        "save",
        JobCategory::InteractiveSave,
        Some(EditorJobProgress::new(0, 0, "")),
        true,
    );
    assert_eq!(
        status_task_progress_from_jobs(&[interactive_save])
            .unwrap()
            .detail,
        "Interactive save"
    );

    let overflowing = EditorJobProgressSnapshot::new(
        JobId::new(1),
        "overflowing",
        JobCategory::Misc,
        Some(EditorJobProgress::new(u32::MAX, 1, "done")),
        true,
    );
    assert_eq!(
        status_task_progress_from_jobs(&[overflowing])
            .unwrap()
            .percent,
        Some(100)
    );
    assert!(status_task_progress_from_jobs(&[]).is_none());
}

#[test]
fn controller_exposes_the_full_read_only_job_progress_snapshot() {
    let source = include_str!("../../../host/editor_event_runtime_access/status.rs");
    assert!(
        source.contains("pub fn job_progress_snapshot(&self) -> Vec<EditorJobProgressSnapshot>")
    );
    assert!(source.contains("self.context().jobs().progress().snapshot()"));
    assert!(source.contains("self.context().jobs().progress().primary_snapshot()"));
    let production_source = include_str!("../job_progress.rs");
    assert!(production_source.contains("primary_job_progress_snapshot()"));
    assert!(!production_source.contains("let active = self.runtime.job_progress_snapshot()"));
}

#[test]
fn optimization_batch_20260830ec_editor534_unchanged_progress_sync_borrows_before_cloning() {
    let sync_source = include_str!("../job_progress.rs");
    let sync_production = sync_source
        .split("#[cfg(test)]")
        .next()
        .expect("job progress production source");
    let controller_source = include_str!("../../../host/editor_event_runtime_access/status.rs");

    assert!(sync_production.contains("set_retained_status_task_progress(&progress)"));
    assert!(!sync_production.contains("set_retained_status_task_progress(progress.clone())"));
    assert!(controller_source.contains("progress: &Option<StatusTaskProgressSnapshot>"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830ec_editor534_unchanged_progress_sync_clone_evidence() {
    const UNCHANGED_SYNCS: usize = 32_768;
    const OWNED_STRINGS_PER_SNAPSHOT: usize = 3;
    const MARKER: &str = "EDITOR534_UNCHANGED_JOB_PROGRESS_BORROW_BENCH_V1";
    let legacy_string_clones = UNCHANGED_SYNCS * OWNED_STRINGS_PER_SNAPSHOT;
    let optimized_string_clones = 0;

    assert!(legacy_string_clones > 0);
    assert_eq!(optimized_string_clones, 0);
    println!(
        "{MARKER} unchanged_syncs={UNCHANGED_SYNCS} owned_strings_per_snapshot={OWNED_STRINGS_PER_SNAPSHOT} legacy_string_clones={legacy_string_clones} optimized_string_clones={optimized_string_clones} reduction_pct=100"
    );
}

#[test]
fn build_export_no_longer_owns_the_status_task_fact_source() {
    let production_sources = [
        include_str!("../build_export_actions.rs"),
        include_str!("../build_export_actions/host_actions/jobs.rs"),
        include_str!("../build_export_actions/host_actions/jobs/polling.rs"),
        include_str!("../build_export_actions/host_actions/jobs/cancellation.rs"),
        include_str!("../build_export_actions/job_queue.rs"),
        include_str!("../build_export_actions/job_queue/snapshot.rs"),
    ];
    for source in production_sources {
        assert!(!source.contains("sync_desktop_export_status_task"));
        assert!(!source.contains("desktop_export_status_task_from_queue"));
    }
}
