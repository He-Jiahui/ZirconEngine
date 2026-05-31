use super::*;

#[test]
fn material_management_query_state_reports_filters_sort_and_page() {
    let sort_order = RenderMaterialManagementSortOrder::new(
        RenderMaterialManagementSortKey::Status,
        RenderMaterialManagementSortDirection::Descending,
    );
    let page = RenderMaterialManagementPageRequest::new(20, Some(10));
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Fallback)
        .with_issue_kind(RenderMaterialManagementIssueKind::Diagnostic)
        .with_text_filter("  roughness  ")
        .with_sort_order(sort_order)
        .with_page(page);

    let state = query.state();

    assert_eq!(state.status, Some(RenderMaterialReadinessStatus::Fallback));
    assert_eq!(
        state.issue_kind,
        Some(RenderMaterialManagementIssueKind::Diagnostic)
    );
    assert_eq!(state.text_filter.as_deref(), Some("roughness"));
    assert_eq!(state.sort_order, sort_order);
    assert_eq!(state.page, page);
    assert!(state.has_status_filter);
    assert!(state.has_issue_filter);
    assert!(state.has_text_filter);
    assert!(state.has_active_filters);
    assert!(state.is_paged);
    assert!(query.has_active_filters());
    assert!(query.is_paged());

    let first_page_query = query.first_page_query();
    assert_eq!(
        first_page_query.page,
        RenderMaterialManagementPageRequest::new(0, Some(10))
    );
    assert_eq!(first_page_query.status, query.status);
    assert_eq!(first_page_query.issue_kind, query.issue_kind);
    assert_eq!(first_page_query.text_filter, query.text_filter);
    assert_eq!(first_page_query.sort_order, query.sort_order);

    let cleared = query.clear_filters();
    assert_eq!(cleared.status, None);
    assert_eq!(cleared.issue_kind, None);
    assert_eq!(cleared.text_filter, None);
    assert_eq!(cleared.sort_order, sort_order);
    assert_eq!(cleared.page, page);
    assert!(!cleared.has_active_filters());
    assert!(cleared.is_paged());
}

#[test]
fn material_management_query_state_treats_empty_text_and_unpaged_query_as_inactive() {
    let query = RenderMaterialManagementQuery::new().with_text_filter("   ");
    let state = query.state();

    assert_eq!(state.text_filter, None);
    assert!(!state.has_text_filter);
    assert!(!state.has_active_filters);
    assert!(!state.is_paged);
    assert!(!query.has_active_filters());
    assert!(!query.is_paged());

    let offset_only_query = RenderMaterialManagementQuery::new()
        .with_page(RenderMaterialManagementPageRequest::new(3, None));
    assert!(offset_only_query.state().is_paged);
    assert!(offset_only_query.is_paged());

    let zero_limit_query = RenderMaterialManagementQuery::new()
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(0)));
    assert!(zero_limit_query.state().is_paged);
    assert!(zero_limit_query.is_paged());
    assert!(!zero_limit_query.has_active_filters());
}
