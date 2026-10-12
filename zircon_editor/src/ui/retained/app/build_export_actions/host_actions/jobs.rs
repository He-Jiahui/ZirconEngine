use std::collections::BTreeMap;

use super::super::DesktopExportExecutionSummary;

#[path = "../../host_actions/jobs/cancellation.rs"]
mod cancellation;
#[path = "../../host_actions/jobs/enqueue.rs"]
mod enqueue;
#[path = "../../host_actions/jobs/polling.rs"]
mod polling;

fn insert_desktop_export_report(
    reports: &mut BTreeMap<String, DesktopExportExecutionSummary>,
    summary: DesktopExportExecutionSummary,
) {
    if let Some(current) = reports.get_mut(summary.profile_name.as_str()) {
        *current = summary;
    } else {
        reports.insert(summary.profile_name.clone(), summary);
    }
}

#[cfg(test)]
#[path = "../../host_actions/tests/jobs.rs"]
mod tests;
