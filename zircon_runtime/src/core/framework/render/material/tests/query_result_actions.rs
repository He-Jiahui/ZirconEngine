use super::*;

#[test]
fn material_management_query_result_actions_offer_filter_and_page_resets() {
    let records = vec![record(
        "material:alpha",
        Some("Alpha Ready"),
        RenderMaterialReadinessStatus::Ready,
    )];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Fallback)
        .with_text_filter("missing")
        .with_page(RenderMaterialManagementPageRequest::new(5, Some(5)));
    let result = record_set.query(query.clone());
    let actions = result.actions(&query);

    assert_eq!(
        actions.state.kind,
        RenderMaterialManagementQueryResultStateKind::EmptyFilteredSet
    );
    assert!(actions.can_clear_filters);
    assert!(actions.can_reset_page);
    assert_eq!(
        actions.clear_filters_query,
        Some(query.clone().clear_filters())
    );
    assert_eq!(actions.first_page_query, Some(query.first_page_query()));
    assert!(actions.previous_page_query.is_none());
    assert!(actions.next_page_query.is_none());
}

#[test]
fn material_management_query_result_actions_offer_adjacent_pages() {
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
            "material:gamma",
            Some("Gamma Ready"),
            RenderMaterialReadinessStatus::Ready,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let query = RenderMaterialManagementQuery::new()
        .with_page(RenderMaterialManagementPageRequest::new(1, Some(1)));
    let result = record_set.query(query.clone());
    let actions = result.actions(&query);

    assert_eq!(
        actions.state.kind,
        RenderMaterialManagementQueryResultStateKind::PopulatedPage
    );
    assert!(!actions.can_clear_filters);
    assert!(actions.can_reset_page);
    assert!(actions.can_go_to_previous_page);
    assert!(actions.can_go_to_next_page);
    assert_eq!(actions.clear_filters_query, None);
    assert_eq!(actions.first_page_query, Some(query.first_page_query()));
    assert_eq!(
        actions.previous_page_query,
        Some(
            query
                .clone()
                .with_page(RenderMaterialManagementPageRequest::new(0, Some(1)))
        )
    );
    assert_eq!(
        actions.next_page_query,
        Some(query.with_page(RenderMaterialManagementPageRequest::new(2, Some(1))))
    );
}

#[test]
fn material_management_query_result_actions_query_selection_exposes_actions() {
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
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Ready)
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(1)));
    let query_selection = record_set.query_selection(query.clone());
    let actions = query_selection.result_actions();

    assert_eq!(
        actions.state.kind,
        RenderMaterialManagementQueryResultStateKind::PopulatedPage
    );
    assert!(actions.can_clear_filters);
    assert!(!actions.can_reset_page);
    assert!(!actions.can_go_to_previous_page);
    assert!(actions.can_go_to_next_page);
    assert_eq!(
        actions.clear_filters_query,
        Some(query.clone().clear_filters())
    );
    assert_eq!(
        actions.next_page_query,
        Some(query.with_page(RenderMaterialManagementPageRequest::new(1, Some(1))))
    );
}
