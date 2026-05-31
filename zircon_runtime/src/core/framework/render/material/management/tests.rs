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
mod query_facets;
mod query_filters;
mod query_result_actions;
mod query_result_state;
mod query_state;

#[test]
fn material_management_sort_orders_records_and_filtered_views() {
    let records = vec![
        record(
            "material:beta",
            Some("Beta"),
            RenderMaterialReadinessStatus::Ready,
        ),
        record(
            "material:alpha",
            Some("Alpha"),
            RenderMaterialReadinessStatus::Ready,
        ),
        record(
            "material:invalid",
            Some("Invalid"),
            RenderMaterialReadinessStatus::Invalid,
        ),
        record(
            "material:fallback",
            None,
            RenderMaterialReadinessStatus::Fallback,
        ),
    ];
    let name_sort = RenderMaterialManagementSortOrder::new(
        RenderMaterialManagementSortKey::MaterialName,
        RenderMaterialManagementSortDirection::Ascending,
    );
    let name_sorted =
        RenderMaterialManagementRecordSet::from_sorted_records(records.clone(), name_sort);

    assert_eq!(
        name_sorted
            .records
            .iter()
            .map(|record| record.material_name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("Alpha"), Some("Beta"), Some("Invalid"), None]
    );
    assert_eq!(name_sorted.summary.total_count, 4);
    assert_eq!(name_sorted.status_index.total_count(), 4);

    let name_desc = RenderMaterialManagementRecordSet::from_sorted_records(
        records.clone(),
        RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Descending,
        ),
    );
    assert_eq!(
        name_desc
            .records
            .iter()
            .map(|record| record.material_name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("Invalid"), Some("Beta"), Some("Alpha"), None]
    );

    let status_sort = RenderMaterialManagementSortOrder::new(
        RenderMaterialManagementSortKey::Status,
        RenderMaterialManagementSortDirection::Descending,
    );
    let status_sorted =
        RenderMaterialManagementRecordSet::from_sorted_records(records.clone(), status_sort);

    assert_eq!(
        status_sorted
            .records
            .iter()
            .map(RenderMaterialManagementRecord::status)
            .collect::<Vec<_>>(),
        vec![
            RenderMaterialReadinessStatus::Invalid,
            RenderMaterialReadinessStatus::Fallback,
            RenderMaterialReadinessStatus::Ready,
            RenderMaterialReadinessStatus::Ready,
        ]
    );
    assert_eq!(
        status_sorted.records[2].material_name.as_deref(),
        Some("Alpha")
    );
    assert_eq!(status_sorted.summary, name_sorted.summary);

    let ready_view = RenderMaterialManagementStatusView::from_records_sorted(
        &records,
        RenderMaterialReadinessStatus::Ready,
        RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Descending,
        ),
    );

    assert_eq!(
        ready_view
            .records
            .iter()
            .map(|record| record.material_name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("Beta"), Some("Alpha")]
    );
    assert_eq!(ready_view.material_ids.len(), 2);
    assert_eq!(ready_view.status, RenderMaterialReadinessStatus::Ready);

    let overview = RenderMaterialManagementOverview::from_records(&records);
    let overview_name_desc = overview.sorted(RenderMaterialManagementSortOrder::new(
        RenderMaterialManagementSortKey::MaterialName,
        RenderMaterialManagementSortDirection::Descending,
    ));
    assert_eq!(
        overview_name_desc
            .records
            .iter()
            .map(|record| record.material_name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("Invalid"), Some("Beta"), Some("Alpha"), None]
    );
    assert_eq!(
        overview_name_desc.status_index.total_count(),
        overview.status_index.total_count()
    );
    assert_eq!(overview_name_desc.summary, overview.summary);
}

#[test]
fn material_management_issue_summary_counts_filtered_and_selected_rows() {
    let records = vec![
        record_with_issue_counts(
            "material:ready",
            Some("Ready"),
            RenderMaterialReadinessStatus::Ready,
            0,
            0,
            0,
        ),
        record_with_issue_counts(
            "material:diagnostic",
            Some("Diagnostic"),
            RenderMaterialReadinessStatus::Diagnostic,
            0,
            0,
            2,
        ),
        record_with_issue_counts(
            "material:fallback",
            Some("Fallback"),
            RenderMaterialReadinessStatus::Fallback,
            0,
            3,
            0,
        ),
        record_with_issue_counts(
            "material:invalid",
            Some("Invalid"),
            RenderMaterialReadinessStatus::Invalid,
            5,
            0,
            1,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records.clone());

    assert_eq!(record_set.summary.total_count, 4);
    assert_eq!(record_set.summary.ready_count, 1);
    assert_eq!(record_set.summary.diagnostic_count, 1);
    assert_eq!(record_set.summary.fallback_count, 1);
    assert_eq!(record_set.summary.invalid_count, 1);
    assert_eq!(record_set.summary.degraded_count(), 3);
    assert_eq!(record_set.summary.validation_error_count, 5);
    assert_eq!(record_set.summary.fallback_usage_count, 3);
    assert_eq!(record_set.summary.diagnostic_row_count, 3);
    assert_eq!(record_set.summary.issue_row_count(), 11);
    assert!(record_set.summary.has_issue_rows());
    assert_eq!(
        record_set.summary.status,
        RenderMaterialReadinessStatus::Invalid
    );
    assert_eq!(
        record_set.issue_index.validation_errors,
        vec![records[3].material_id]
    );
    assert_eq!(
        record_set.issue_index.fallback_usages,
        vec![records[2].material_id]
    );
    assert_eq!(
        record_set.issue_index.diagnostics,
        vec![records[1].material_id, records[3].material_id]
    );
    assert_eq!(
        record_set
            .issue_index
            .ids_for_issue_kind(RenderMaterialManagementIssueKind::ValidationError),
        &[records[3].material_id]
    );
    assert_eq!(
        record_set
            .issue_index
            .ids_for_issue_kind(RenderMaterialManagementIssueKind::FallbackUsage),
        &[records[2].material_id]
    );
    assert_eq!(
        record_set
            .issue_index
            .ids_for_issue_kind(RenderMaterialManagementIssueKind::Diagnostic),
        &[records[1].material_id, records[3].material_id]
    );
    assert_eq!(record_set.issue_index.bucket_entry_count(), 4);
    assert!(!record_set.issue_index.is_empty());

    let paged_fallbacks = record_set.query(
        RenderMaterialManagementQuery::new()
            .with_status(RenderMaterialReadinessStatus::Fallback)
            .with_page(RenderMaterialManagementPageRequest::new(0, Some(0))),
    );
    assert!(paged_fallbacks.records.is_empty());
    assert_eq!(paged_fallbacks.page.total_count, 1);
    assert_eq!(paged_fallbacks.summary.total_count, 1);
    assert_eq!(paged_fallbacks.summary.fallback_count, 1);
    assert_eq!(paged_fallbacks.summary.validation_error_count, 0);
    assert_eq!(paged_fallbacks.summary.fallback_usage_count, 3);
    assert_eq!(paged_fallbacks.summary.diagnostic_row_count, 0);
    assert_eq!(paged_fallbacks.summary.issue_row_count(), 3);
    assert_eq!(
        paged_fallbacks.summary.status,
        RenderMaterialReadinessStatus::Fallback
    );
    assert_eq!(
        paged_fallbacks.issue_index.fallback_usages,
        vec![records[2].material_id]
    );
    assert!(paged_fallbacks.issue_index.validation_errors.is_empty());
    assert!(paged_fallbacks.issue_index.diagnostics.is_empty());

    let overview_query = record_set.overview().query(
        RenderMaterialManagementQuery::new()
            .with_text_filter("Diagnostic")
            .with_page(RenderMaterialManagementPageRequest::new(0, Some(1))),
    );
    assert_eq!(overview_query.records.len(), 1);
    assert_eq!(overview_query.summary.total_count, 1);
    assert_eq!(overview_query.summary.diagnostic_count, 1);
    assert_eq!(overview_query.summary.validation_error_count, 0);
    assert_eq!(overview_query.summary.fallback_usage_count, 0);
    assert_eq!(overview_query.summary.diagnostic_row_count, 2);
    assert_eq!(overview_query.summary.issue_row_count(), 2);
    assert_eq!(
        overview_query.issue_index.diagnostics,
        vec![records[1].material_id]
    );
    assert!(overview_query.issue_index.validation_errors.is_empty());
    assert!(overview_query.issue_index.fallback_usages.is_empty());

    let missing_id = ResourceId::from_stable_label("material:missing");
    let selection = record_set.select([
        records[1].material_id,
        missing_id,
        records[3].material_id,
        records[1].material_id,
    ]);
    assert_eq!(selection.requested_count, 3);
    assert_eq!(selection.summary.total_count, 2);
    assert_eq!(selection.summary.diagnostic_count, 1);
    assert_eq!(selection.summary.invalid_count, 1);
    assert_eq!(selection.summary.validation_error_count, 5);
    assert_eq!(selection.summary.fallback_usage_count, 0);
    assert_eq!(selection.summary.diagnostic_row_count, 3);
    assert_eq!(selection.summary.issue_row_count(), 8);
    assert_eq!(
        selection.issue_index.validation_errors,
        vec![records[3].material_id]
    );
    assert!(selection.issue_index.fallback_usages.is_empty());
    assert_eq!(
        selection.issue_index.diagnostics,
        vec![records[1].material_id, records[3].material_id]
    );
    assert_eq!(selection.missing_material_ids, vec![missing_id]);
}

#[test]
fn material_management_issue_index_tracks_filtered_and_selected_issue_types() {
    let records = vec![
        record_with_issue_counts(
            "material:invalid",
            Some("Invalid"),
            RenderMaterialReadinessStatus::Invalid,
            2,
            0,
            1,
        ),
        record_with_issue_counts(
            "material:fallback",
            Some("Fallback"),
            RenderMaterialReadinessStatus::Fallback,
            0,
            3,
            1,
        ),
        record_with_issue_counts(
            "material:diagnostic",
            Some("Diagnostic"),
            RenderMaterialReadinessStatus::Diagnostic,
            0,
            0,
            1,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records.clone());

    assert_eq!(
        record_set.status_index.invalid,
        vec![records[0].material_id]
    );
    assert_eq!(
        record_set.status_index.fallback,
        vec![records[1].material_id]
    );
    assert_eq!(
        record_set.status_index.diagnostic,
        vec![records[2].material_id]
    );
    assert_eq!(
        record_set.issue_index.validation_errors,
        vec![records[0].material_id]
    );
    assert_eq!(
        record_set.issue_index.fallback_usages,
        vec![records[1].material_id]
    );
    assert_eq!(
        record_set.issue_index.diagnostics,
        vec![
            records[0].material_id,
            records[1].material_id,
            records[2].material_id
        ]
    );

    let paged_diagnostics = record_set.query(
        RenderMaterialManagementQuery::new()
            .with_status(RenderMaterialReadinessStatus::Diagnostic)
            .with_page(RenderMaterialManagementPageRequest::new(0, Some(0))),
    );
    assert!(paged_diagnostics.records.is_empty());
    assert_eq!(paged_diagnostics.page.total_count, 1);
    assert_eq!(
        paged_diagnostics.issue_index.diagnostics,
        vec![records[2].material_id]
    );

    let selection = record_set.select([
        records[1].material_id,
        records[0].material_id,
        records[1].material_id,
    ]);
    assert_eq!(
        selection.issue_index.validation_errors,
        vec![records[0].material_id]
    );
    assert_eq!(
        selection.issue_index.fallback_usages,
        vec![records[1].material_id]
    );
    assert_eq!(
        selection.issue_index.diagnostics,
        vec![records[1].material_id, records[0].material_id]
    );
}

#[test]
fn material_management_issue_view_returns_rows_for_issue_kind() {
    let records = vec![
        record_with_issue_counts(
            "material:invalid",
            Some("Invalid"),
            RenderMaterialReadinessStatus::Invalid,
            2,
            0,
            1,
        ),
        record_with_issue_counts(
            "material:fallback",
            Some("Fallback"),
            RenderMaterialReadinessStatus::Fallback,
            0,
            3,
            1,
        ),
        record_with_issue_counts(
            "material:diagnostic",
            Some("Diagnostic"),
            RenderMaterialReadinessStatus::Diagnostic,
            0,
            0,
            1,
        ),
        record_with_issue_counts(
            "material:ready",
            Some("Ready"),
            RenderMaterialReadinessStatus::Ready,
            0,
            0,
            0,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records.clone());

    let diagnostic_view = record_set.issue_view(RenderMaterialManagementIssueKind::Diagnostic);
    assert_eq!(
        diagnostic_view.material_ids,
        record_set.issue_index.diagnostics
    );
    assert_eq!(
        diagnostic_view
            .records
            .iter()
            .map(|record| record.material_name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("Invalid"), Some("Fallback"), Some("Diagnostic")]
    );
    assert_eq!(
        diagnostic_view.issue_kind,
        RenderMaterialManagementIssueKind::Diagnostic
    );
    assert_eq!(diagnostic_view.len(), 3);
    assert!(!diagnostic_view.is_empty());

    let fallback_view = RenderMaterialManagementIssueView::from_overview(
        &record_set.overview(),
        RenderMaterialManagementIssueKind::FallbackUsage,
    );
    assert_eq!(
        fallback_view.material_ids,
        record_set.issue_index.fallback_usages
    );
    assert_eq!(fallback_view.records.len(), 1);
    assert_eq!(fallback_view.records[0].material_id, records[1].material_id);

    let sorted_diagnostic_view = record_set.issue_view_sorted(
        RenderMaterialManagementIssueKind::Diagnostic,
        RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Ascending,
        ),
    );
    assert_eq!(
        sorted_diagnostic_view
            .records
            .iter()
            .map(|record| record.material_name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("Diagnostic"), Some("Fallback"), Some("Invalid")]
    );
    assert_eq!(
        sorted_diagnostic_view.material_ids,
        sorted_diagnostic_view
            .records
            .iter()
            .map(|record| record.material_id)
            .collect::<Vec<_>>()
    );

    let validation_error_view =
        record_set.issue_view(RenderMaterialManagementIssueKind::ValidationError);
    assert_eq!(
        validation_error_view.material_ids,
        vec![records[0].material_id]
    );
    assert_eq!(validation_error_view.records.len(), 1);
}

#[test]
fn material_management_query_filters_issue_kind_before_sorting_and_paging() {
    let records = vec![
        record_with_issue_counts(
            "material:invalid",
            Some("Zeta Issue"),
            RenderMaterialReadinessStatus::Invalid,
            2,
            0,
            1,
        ),
        record_with_issue_counts(
            "material:fallback",
            Some("Beta Issue"),
            RenderMaterialReadinessStatus::Fallback,
            0,
            3,
            1,
        ),
        record_with_issue_counts(
            "material:diagnostic",
            Some("Alpha Issue"),
            RenderMaterialReadinessStatus::Diagnostic,
            0,
            0,
            1,
        ),
        record_with_issue_counts(
            "material:ready",
            Some("Ready"),
            RenderMaterialReadinessStatus::Ready,
            0,
            0,
            0,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records.clone());

    let first_diagnostic_page = record_set.query(
        RenderMaterialManagementQuery::new()
            .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic)
            .with_text_filter("issue")
            .with_sort_order(RenderMaterialManagementSortOrder::new(
                RenderMaterialManagementSortKey::MaterialName,
                RenderMaterialManagementSortDirection::Ascending,
            ))
            .with_page(RenderMaterialManagementPageRequest::new(0, Some(2))),
    );

    assert_eq!(first_diagnostic_page.summary.total_count, 3);
    assert_eq!(first_diagnostic_page.summary.diagnostic_count, 1);
    assert_eq!(first_diagnostic_page.summary.fallback_count, 1);
    assert_eq!(first_diagnostic_page.summary.invalid_count, 1);
    assert_eq!(first_diagnostic_page.summary.validation_error_count, 2);
    assert_eq!(first_diagnostic_page.summary.fallback_usage_count, 3);
    assert_eq!(first_diagnostic_page.summary.diagnostic_row_count, 3);
    assert_eq!(
        first_diagnostic_page.summary.status,
        RenderMaterialReadinessStatus::Invalid
    );
    assert_eq!(
        first_diagnostic_page.issue_index.diagnostics,
        vec![
            records[2].material_id,
            records[1].material_id,
            records[0].material_id
        ]
    );
    assert_eq!(
        first_diagnostic_page.issue_index.validation_errors,
        vec![records[0].material_id]
    );
    assert_eq!(
        first_diagnostic_page.issue_index.fallback_usages,
        vec![records[1].material_id]
    );
    assert_eq!(first_diagnostic_page.page.total_count, 3);
    assert_eq!(first_diagnostic_page.page.returned_count, 2);
    assert!(first_diagnostic_page.page.has_next_page);
    assert_eq!(
        first_diagnostic_page
            .records
            .iter()
            .map(|record| record.material_name.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("Alpha Issue"), Some("Beta Issue")]
    );

    let fallback_status_diagnostics = record_set.query(
        RenderMaterialManagementQuery::new()
            .with_status(RenderMaterialReadinessStatus::Fallback)
            .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic),
    );
    assert_eq!(fallback_status_diagnostics.records.len(), 1);
    assert_eq!(
        fallback_status_diagnostics.records[0].material_id,
        records[1].material_id
    );
    assert_eq!(
        fallback_status_diagnostics.summary.status,
        RenderMaterialReadinessStatus::Fallback
    );

    let impossible_status_issue_pair = record_set.overview().query(
        RenderMaterialManagementQuery::new()
            .with_status(RenderMaterialReadinessStatus::Ready)
            .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic),
    );
    assert!(impossible_status_issue_pair.records.is_empty());
    assert_eq!(impossible_status_issue_pair.page.total_count, 0);
    assert!(impossible_status_issue_pair.issue_index.is_empty());
}

#[test]
fn material_management_query_selection_returns_page_details_in_display_order() {
    let records = vec![
        record_with_issue_counts(
            "material:invalid",
            Some("Zeta Issue"),
            RenderMaterialReadinessStatus::Invalid,
            2,
            0,
            1,
        ),
        record_with_issue_counts(
            "material:fallback",
            Some("Beta Issue"),
            RenderMaterialReadinessStatus::Fallback,
            0,
            3,
            1,
        ),
        record_with_issue_counts(
            "material:diagnostic",
            Some("Alpha Issue"),
            RenderMaterialReadinessStatus::Diagnostic,
            0,
            0,
            1,
        ),
        record_with_issue_counts(
            "material:ready",
            Some("Ready"),
            RenderMaterialReadinessStatus::Ready,
            0,
            0,
            0,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records.clone());
    let query = RenderMaterialManagementQuery::new()
        .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic)
        .with_text_filter("issue")
        .with_sort_order(RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Ascending,
        ))
        .with_page(RenderMaterialManagementPageRequest::new(1, Some(1)));

    let query_selection = record_set.query_selection(query.clone());

    assert_eq!(query_selection.query, query);
    assert_eq!(query_selection.len(), 1);
    assert!(!query_selection.is_empty());
    assert!(query_selection.is_complete());
    assert_eq!(query_selection.query_result.summary.total_count, 3);
    assert_eq!(query_selection.query_result.summary.diagnostic_count, 1);
    assert_eq!(query_selection.query_result.summary.fallback_count, 1);
    assert_eq!(query_selection.query_result.summary.invalid_count, 1);
    assert_eq!(query_selection.query_result.page.total_count, 3);
    assert_eq!(query_selection.query_result.page.returned_count, 1);
    assert!(query_selection.query_result.page.has_previous_page);
    assert!(query_selection.query_result.page.has_next_page);
    assert_eq!(
        query_selection
            .query_result
            .records
            .iter()
            .map(|record| record.material_id)
            .collect::<Vec<_>>(),
        vec![records[1].material_id]
    );
    assert_eq!(query_selection.selection.requested_count, 1);
    assert_eq!(query_selection.selection.len(), 1);
    assert!(query_selection.selection.missing_material_ids.is_empty());
    assert_eq!(
        query_selection
            .selection
            .records
            .iter()
            .map(|record| record.material_id)
            .collect::<Vec<_>>(),
        vec![records[1].material_id]
    );
    assert_eq!(query_selection.selection.records[0], records[1]);
    assert_eq!(query_selection.selection.summary.total_count, 1);
    assert_eq!(query_selection.selection.summary.fallback_count, 1);
    assert_eq!(query_selection.selection.summary.fallback_usage_count, 3);
    assert_eq!(query_selection.selection.summary.diagnostic_row_count, 1);
    assert_eq!(
        query_selection.selection.issue_index.fallback_usages,
        vec![records[1].material_id]
    );
    assert_eq!(
        query_selection.selection.issue_index.diagnostics,
        vec![records[1].material_id]
    );
    assert!(query_selection
        .selection
        .issue_index
        .validation_errors
        .is_empty());
}

#[test]
fn material_management_query_filters_sorts_and_pages() {
    let records = vec![
        record(
            "material:alpha",
            Some("Alpha Ready"),
            RenderMaterialReadinessStatus::Ready,
        ),
        record(
            "material:beta",
            Some("Beta Ready"),
            RenderMaterialReadinessStatus::Ready,
        ),
        record(
            "material:diagnostic",
            Some("Gamma Diagnostic"),
            RenderMaterialReadinessStatus::Diagnostic,
        ),
        record(
            "material:fallback",
            None,
            RenderMaterialReadinessStatus::Fallback,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Ready)
        .with_text_filter("READY")
        .with_sort_order(RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Descending,
        ))
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(1)));

    let first_page = record_set.query(query.clone());
    assert_eq!(first_page.summary.total_count, 2);
    assert_eq!(first_page.summary.ready_count, 2);
    assert_eq!(first_page.summary.degraded_count(), 0);
    assert_eq!(
        first_page.summary.status,
        RenderMaterialReadinessStatus::Ready
    );
    assert_eq!(first_page.status_index.ready.len(), 2);
    assert_eq!(first_page.page.total_count, 2);
    assert_eq!(first_page.page.returned_count, 1);
    assert!(!first_page.page.has_previous_page);
    assert!(first_page.page.has_next_page);
    assert_eq!(
        first_page.records[0].material_name.as_deref(),
        Some("Beta Ready")
    );

    let second_page =
        record_set.query(query.with_page(RenderMaterialManagementPageRequest::new(1, Some(1))));
    assert_eq!(second_page.page.total_count, 2);
    assert_eq!(second_page.page.returned_count, 1);
    assert!(second_page.page.has_previous_page);
    assert!(!second_page.page.has_next_page);
    assert_eq!(
        second_page.records[0].material_name.as_deref(),
        Some("Alpha Ready")
    );

    let id_text_query = RenderMaterialManagementQuery::new()
        .with_text_filter(record_set.records[2].material_id.to_string());
    let id_text_result = record_set.overview().query(id_text_query);
    assert_eq!(id_text_result.records.len(), 1);
    assert_eq!(
        id_text_result.records[0].status(),
        RenderMaterialReadinessStatus::Diagnostic
    );
}

#[test]
fn material_management_selection_preserves_request_order_and_missing_ids() {
    let records = vec![
        record(
            "material:alpha",
            Some("Alpha Ready"),
            RenderMaterialReadinessStatus::Ready,
        ),
        record(
            "material:beta",
            Some("Beta Diagnostic"),
            RenderMaterialReadinessStatus::Diagnostic,
        ),
        record(
            "material:invalid",
            Some("Invalid"),
            RenderMaterialReadinessStatus::Invalid,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records.clone());
    let missing_id = ResourceId::from_stable_label("material:missing");

    let selection = record_set.select([
        records[1].material_id,
        missing_id,
        records[0].material_id,
        records[1].material_id,
    ]);

    assert_eq!(selection.requested_count, 3);
    assert_eq!(selection.len(), 2);
    assert!(!selection.is_empty());
    assert_eq!(selection.missing_count(), 1);
    assert!(!selection.is_complete());
    assert_eq!(selection.missing_material_ids, vec![missing_id]);
    assert_eq!(
        selection
            .records
            .iter()
            .map(|record| record.material_id)
            .collect::<Vec<_>>(),
        vec![records[1].material_id, records[0].material_id]
    );
    assert_eq!(selection.summary.total_count, 2);
    assert_eq!(selection.summary.ready_count, 1);
    assert_eq!(selection.summary.diagnostic_count, 1);
    assert_eq!(selection.summary.invalid_count, 0);
    assert_eq!(
        selection.summary.status,
        RenderMaterialReadinessStatus::Diagnostic
    );
    assert_eq!(selection.status_index.total_count(), 2);
    assert!(selection.issue_index.is_empty());
    assert_eq!(
        selection
            .status_index
            .ids_for_status(RenderMaterialReadinessStatus::Diagnostic),
        &[records[1].material_id]
    );
    assert_eq!(
        selection
            .status_index
            .ids_for_status(RenderMaterialReadinessStatus::Ready),
        &[records[0].material_id]
    );

    let empty_selection = RenderMaterialManagementSelection::from_records(&records, [missing_id]);
    assert_eq!(empty_selection.requested_count, 1);
    assert!(empty_selection.is_empty());
    assert_eq!(empty_selection.missing_count(), 1);
    assert_eq!(
        empty_selection.summary.status,
        RenderMaterialReadinessStatus::Ready
    );
    assert!(empty_selection.issue_index.is_empty());
}
