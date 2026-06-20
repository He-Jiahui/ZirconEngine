use crate::ui::retained_host::app::RetainedEditorHost;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn poll_desktop_export_jobs(&mut self) {
        let (summaries, mut changed) = self.desktop_export_jobs.poll_updates();
        for summary in summaries {
            let message = summary.status_message();
            self.desktop_export_reports
                .insert(summary.profile_name.clone(), summary);
            self.set_status_line(message);
        }
        if let Some(started) = self
            .desktop_export_jobs
            .start_next(self.editor_manager.clone())
        {
            self.set_status_line(format!(
                "Desktop export {} started -> {}",
                started.profile_name,
                started.output_root.display()
            ));
            changed = true;
        }
        self.sync_desktop_export_status_task();
        if changed {
            self.mark_layout_dirty();
        }
    }
}
