use super::*;

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
    assert!(
        query_selection
            .selection
            .issue_index
            .validation_errors
            .is_empty()
    );
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
