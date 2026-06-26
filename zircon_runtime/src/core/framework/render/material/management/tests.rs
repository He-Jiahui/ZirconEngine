use super::*;

fn record(
    label: &str,
    name: Option<&str>,
    status: RenderMaterialReadinessStatus,
) -> RenderMaterialManagementRecord {
    RenderMaterialManagementRecord {
        material_id: ResourceId::from_stable_label(label),
        material_name: name.map(str::to_string),
        snapshot: RenderMaterialManagementSnapshot {
            summary: RenderMaterialReadinessSummary {
                status,
                is_ready: status != RenderMaterialReadinessStatus::Invalid,
                ..RenderMaterialReadinessSummary::default()
            },
            ..RenderMaterialManagementSnapshot::default()
        },
    }
}

fn record_with_issue_counts(
    label: &str,
    name: Option<&str>,
    status: RenderMaterialReadinessStatus,
    validation_error_count: usize,
    fallback_usage_count: usize,
    diagnostic_row_count: usize,
) -> RenderMaterialManagementRecord {
    let mut record = record(label, name, status);
    record.snapshot.summary.validation_error_count = validation_error_count;
    record.snapshot.summary.fallback_usage_count = fallback_usage_count;
    record.snapshot.summary.diagnostic_count = diagnostic_row_count;
    record.snapshot.summary.uses_fallback = fallback_usage_count > 0;
    record.snapshot.summary.has_diagnostics = diagnostic_row_count > 0;
    record
}

mod page_navigation;
mod query_controls;
mod query_execution;
mod query_facets;
mod query_filters;
mod query_result_actions;
mod query_result_state;
mod query_state;
mod record_views;
