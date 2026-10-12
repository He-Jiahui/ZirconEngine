use super::*;

#[test]
fn material_management_query_filters_list_active_filters_with_remove_queries() {
    let sort_order = RenderMaterialManagementSortOrder::new(
        RenderMaterialManagementSortKey::MaterialName,
        RenderMaterialManagementSortDirection::Descending,
    );
    let page = RenderMaterialManagementPageRequest::new(20, Some(10));
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Fallback)
        .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic)
        .with_text_filter("  roughness  ")
        .with_sort_order(sort_order)
        .with_page(page);

    let filters = query.active_filters();

    assert_eq!(filters.len(), 3);
    assert_eq!(
        filters.iter().map(|filter| filter.kind).collect::<Vec<_>>(),
        vec![
            RenderMaterialManagementQueryFilterKind::Status,
            RenderMaterialManagementQueryFilterKind::IssueKind,
            RenderMaterialManagementQueryFilterKind::Text,
        ]
    );
    assert_eq!(
        filters[0].status,
        Some(RenderMaterialReadinessStatus::Fallback)
    );
    assert_eq!(filters[0].remove_query, query.without_status_filter());
    assert_eq!(filters[0].remove_query.status, None);
    assert_eq!(filters[0].remove_query.issue_kind, query.issue_kind);
    assert_eq!(filters[0].remove_query.text_filter, query.text_filter);
    assert_eq!(filters[0].remove_query.sort_order, sort_order);
    assert_eq!(
        filters[0].remove_query.page,
        RenderMaterialManagementPageRequest::new(0, Some(10))
    );

    assert_eq!(
        filters[1].issue_kind,
        Some(RenderMaterialManagementIssueKind::Diagnostic)
    );
    assert_eq!(filters[1].remove_query, query.without_issue_kind_filter());
    assert_eq!(filters[1].remove_query.status, query.status);
    assert_eq!(filters[1].remove_query.issue_kind, None);

    assert_eq!(filters[2].text.as_deref(), Some("roughness"));
    assert_eq!(filters[2].remove_query, query.without_text_filter());
    assert_eq!(filters[2].remove_query.text_filter, None);
}

#[test]
fn material_management_query_state_exposes_active_filter_rows() {
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Ready)
        .with_text_filter("  ready  ");
    let state = query.state();

    let filters = state.active_filters();

    assert_eq!(filters.len(), 2);
    assert_eq!(
        filters.iter().map(|filter| filter.kind).collect::<Vec<_>>(),
        vec![
            RenderMaterialManagementQueryFilterKind::Status,
            RenderMaterialManagementQueryFilterKind::Text,
        ]
    );
    assert_eq!(filters[0].remove_query, query.without_status_filter());
    assert_eq!(filters[1].remove_query, query.without_text_filter());

    let empty_query = RenderMaterialManagementQuery::new().with_text_filter("   ");
    assert!(empty_query.active_filters().is_empty());
    assert!(empty_query.state().active_filters().is_empty());
}
