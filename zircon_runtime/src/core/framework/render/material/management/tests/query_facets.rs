use super::*;

#[test]
fn material_management_query_facets_build_status_and_issue_select_queries() {
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
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let query = RenderMaterialManagementQuery::new()
        .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic)
        .with_text_filter(" issue ")
        .with_sort_order(RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Ascending,
        ))
        .with_page(RenderMaterialManagementPageRequest::new(2, Some(1)));

    let result = record_set.query(query.clone());
    let facets = result.facets(&query);

    assert_eq!(
        facets
            .status_facets
            .iter()
            .map(|facet| (facet.status, facet.material_count, facet.is_active))
            .collect::<Vec<_>>(),
        vec![
            (Some(RenderMaterialReadinessStatus::Ready), 0, false),
            (Some(RenderMaterialReadinessStatus::Diagnostic), 1, false),
            (Some(RenderMaterialReadinessStatus::Fallback), 1, false),
            (Some(RenderMaterialReadinessStatus::Invalid), 1, false),
        ]
    );
    assert_eq!(
        facets
            .issue_facets
            .iter()
            .map(|facet| (facet.issue_kind, facet.material_count, facet.is_active))
            .collect::<Vec<_>>(),
        vec![
            (
                Some(RenderMaterialManagementIssueKind::ValidationError),
                1,
                false
            ),
            (
                Some(RenderMaterialManagementIssueKind::FallbackUsage),
                1,
                false
            ),
            (Some(RenderMaterialManagementIssueKind::Diagnostic), 3, true),
        ]
    );

    let fallback_facet = &facets.status_facets[2];
    assert_eq!(
        fallback_facet.kind,
        RenderMaterialManagementQueryFacetKind::Status
    );
    assert_eq!(
        fallback_facet.select_query,
        RenderMaterialManagementQuery {
            status: Some(RenderMaterialReadinessStatus::Fallback),
            issue_kind: Some(RenderMaterialManagementIssueKind::Diagnostic),
            text_filter: Some("issue".to_string()),
            sort_order: query.sort_order,
            page: RenderMaterialManagementPageRequest::new(0, Some(1)),
        }
    );

    let validation_facet = &facets.issue_facets[0];
    assert_eq!(
        validation_facet.kind,
        RenderMaterialManagementQueryFacetKind::IssueKind
    );
    assert_eq!(
        validation_facet.select_query,
        RenderMaterialManagementQuery {
            status: None,
            issue_kind: Some(RenderMaterialManagementIssueKind::ValidationError),
            text_filter: Some("issue".to_string()),
            sort_order: query.sort_order,
            page: RenderMaterialManagementPageRequest::new(0, Some(1)),
        }
    );
    assert_eq!(result.controls(&query).facets, facets);
}

#[test]
fn material_management_query_facets_selection_exposes_result_facets() {
    let records = vec![
        record_with_issue_counts(
            "material:invalid",
            Some("Invalid Issue"),
            RenderMaterialReadinessStatus::Invalid,
            2,
            0,
            1,
        ),
        record_with_issue_counts(
            "material:fallback",
            Some("Fallback Issue"),
            RenderMaterialReadinessStatus::Fallback,
            0,
            3,
            1,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Invalid)
        .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic)
        .with_page(RenderMaterialManagementPageRequest::new(1, Some(1)));

    let query_selection = record_set.query_selection(query.clone());
    let facets = query_selection.result_facets();

    assert_eq!(facets, query_selection.query_result.facets(&query));
    assert_eq!(facets, query_selection.result_controls().facets);
    assert_eq!(
        facets
            .status_facets
            .iter()
            .map(|facet| (facet.status, facet.material_count, facet.is_active))
            .collect::<Vec<_>>(),
        vec![
            (Some(RenderMaterialReadinessStatus::Ready), 0, false),
            (Some(RenderMaterialReadinessStatus::Diagnostic), 0, false),
            (Some(RenderMaterialReadinessStatus::Fallback), 0, false),
            (Some(RenderMaterialReadinessStatus::Invalid), 1, true),
        ]
    );
    assert_eq!(
        facets
            .issue_facets
            .iter()
            .map(|facet| (facet.issue_kind, facet.material_count, facet.is_active))
            .collect::<Vec<_>>(),
        vec![
            (
                Some(RenderMaterialManagementIssueKind::ValidationError),
                1,
                false
            ),
            (
                Some(RenderMaterialManagementIssueKind::FallbackUsage),
                0,
                false
            ),
            (Some(RenderMaterialManagementIssueKind::Diagnostic), 1, true),
        ]
    );
    assert!(query_selection.query_result.records.is_empty());
    assert_eq!(query_selection.query_result.page.total_count, 1);
}
