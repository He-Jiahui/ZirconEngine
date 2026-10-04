use crate::core::jobs::{EditorJobProgressSnapshot, JobCategory};
use crate::ui::workbench::snapshot::{StatusTaskProgressSnapshot, StatusTaskProgressTone};

use super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(super) fn sync_editor_job_progress(&mut self) {
        let primary = self.runtime.primary_job_progress_snapshot();
        let progress = status_task_progress_from_jobs(primary.as_slice());
        if !self.runtime.set_retained_status_task_progress(&progress) {
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
#[path = "tests/job_progress.rs"]
mod tests;
