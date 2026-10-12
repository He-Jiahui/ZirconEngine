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

#[path = "../../tests/page_navigation.rs"]
mod page_navigation;
#[path = "../../tests/query_controls.rs"]
mod query_controls;
#[path = "../../tests/query_execution.rs"]
mod query_execution;
#[path = "../../tests/query_facets.rs"]
mod query_facets;
#[path = "../../tests/query_filters.rs"]
mod query_filters;
#[path = "../../tests/query_result_actions.rs"]
mod query_result_actions;
#[path = "../../tests/query_result_state.rs"]
mod query_result_state;
#[path = "../../tests/query_state.rs"]
mod query_state;
#[path = "../../tests/record_views.rs"]
mod record_views;
