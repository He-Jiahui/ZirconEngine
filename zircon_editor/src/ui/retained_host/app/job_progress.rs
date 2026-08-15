use crate::core::jobs::{EditorJobProgressSnapshot, JobCategory};
use crate::ui::workbench::snapshot::{StatusTaskProgressSnapshot, StatusTaskProgressTone};

use super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(super) fn sync_editor_job_progress(&mut self) {
        let primary = self.runtime.primary_job_progress_snapshot();
        let progress = status_task_progress_from_jobs(primary.as_slice());
        if !self
            .runtime
            .set_retained_status_task_progress(progress.clone())
        {
            return;
        }
        match self
            .workbench_window_bridge
            .prepare_status_task_progress(progress.as_ref())
        {
            Ok(()) => self.pending_activity_projection_refresh = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}

fn status_task_progress_from_jobs(
    active: &[EditorJobProgressSnapshot],
) -> Option<StatusTaskProgressSnapshot> {
    let job = active.iter().min_by_key(|job| job.id())?;
    let progress = job.progress();
    let detail = progress
        .map(|progress| progress.message().trim())
        .filter(|message| !message.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| category_label(job.category()).to_string());
    Some(
        StatusTaskProgressSnapshot::new(format!("editor_job:{}", job.id().value()), job.label())
            .with_detail(detail)
            .with_percent(progress.and_then(progress_percent))
            .with_tone(StatusTaskProgressTone::Info),
    )
}

fn progress_percent(progress: &crate::core::jobs::EditorJobProgress) -> Option<u8> {
    if progress.total() == 0 {
        return None;
    }
    let percent = (u64::from(progress.completed()) * 100) / u64::from(progress.total());
    Some(percent.min(100) as u8)
}

fn category_label(category: JobCategory) -> &'static str {
    match category {
        JobCategory::Import => "Import",
        JobCategory::Compile => "Compile",
        JobCategory::Thumbnail => "Thumbnail",
        JobCategory::Export => "Export",
        JobCategory::InteractiveSave => "Interactive save",
        JobCategory::Index => "Index",
        JobCategory::Play => "Play",
        JobCategory::Misc => "Miscellaneous",
    }
}

#[cfg(test)]
mod tests {
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
        let source = include_str!("../../host/editor_event_runtime_access/status.rs");
        assert!(source
            .contains("pub fn job_progress_snapshot(&self) -> Vec<EditorJobProgressSnapshot>"));
        assert!(source.contains("self.context().jobs().progress().snapshot()"));
        assert!(source.contains("self.context().jobs().progress().primary_snapshot()"));
        let production_source = include_str!("job_progress.rs");
        assert!(production_source.contains("primary_job_progress_snapshot()"));
        assert!(!production_source.contains("let active = self.runtime.job_progress_snapshot()"));
    }

    #[test]
    fn build_export_no_longer_owns_the_status_task_fact_source() {
        let production_sources = [
            include_str!("build_export_actions.rs"),
            include_str!("build_export_actions/host_actions/jobs.rs"),
            include_str!("build_export_actions/host_actions/jobs/polling.rs"),
            include_str!("build_export_actions/host_actions/jobs/cancellation.rs"),
            include_str!("build_export_actions/job_queue.rs"),
            include_str!("build_export_actions/job_queue/snapshot.rs"),
        ];
        for source in production_sources {
            assert!(!source.contains("sync_desktop_export_status_task"));
            assert!(!source.contains("desktop_export_status_task_from_queue"));
        }
    }
}
