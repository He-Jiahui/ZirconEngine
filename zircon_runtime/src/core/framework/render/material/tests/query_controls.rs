use super::*;

#[test]
fn material_management_query_controls_aggregate_filter_actions_and_page_chrome() {
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
            "material:fallback",
            Some("Fallback"),
            RenderMaterialReadinessStatus::Fallback,
        ),
    ];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Ready)
        .with_text_filter("ready")
        .with_sort_order(RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Ascending,
        ))
        .with_page(RenderMaterialManagementPageRequest::new(1, Some(1)));

    let result = record_set.query(query.clone());
    let controls = result.controls(&query);

    assert_eq!(controls.query_state, query.state());
    assert_eq!(
        controls.result_state.kind,
        RenderMaterialManagementQueryResultStateKind::PopulatedPage
    );
    assert_eq!(controls.result_state.total_count, 2);
    assert_eq!(controls.result_state.returned_count, 1);
    assert_eq!(controls.actions.state, controls.result_state);
    assert!(controls.actions.can_clear_filters);
    assert!(controls.actions.can_reset_page);
    assert!(controls.actions.can_go_to_previous_page);
    assert!(!controls.actions.can_go_to_next_page);
    assert_eq!(
        controls.actions.previous_page_query,
        Some(
            query
                .clone()
                .with_page(RenderMaterialManagementPageRequest::new(0, Some(1)))
        )
    );
    assert_eq!(
        controls
            .active_filters
            .iter()
            .map(|filter| filter.kind)
            .collect::<Vec<_>>(),
        vec![
            RenderMaterialManagementQueryFilterKind::Status,
            RenderMaterialManagementQueryFilterKind::Text,
        ]
    );
    assert_eq!(
        controls.active_filters[0].remove_query.page,
        RenderMaterialManagementPageRequest::new(0, Some(1))
    );
    assert_eq!(
        controls.page_window,
        RenderMaterialManagementPageWindow {
            start_index: 1,
            end_index_exclusive: 2,
        }
    );
    assert_eq!(controls.display_start_index, Some(2));
    assert_eq!(controls.display_end_index, Some(2));
    assert_eq!(controls.current_page_number, Some(2));
    assert_eq!(controls.total_page_count, Some(2));
}

#[test]
fn material_management_query_selection_exposes_query_controls() {
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
    let query_selection = record_set.query_selection(query.clone());

    let controls = query_selection.result_controls();

    assert_eq!(controls, query_selection.query_result.controls(&query));
    assert_eq!(
        controls.result_state.kind,
        RenderMaterialManagementQueryResultStateKind::EmptyFilteredSet
    );
    assert_eq!(controls.active_filters.len(), 2);
    assert!(controls.actions.can_clear_filters);
    assert!(controls.actions.can_reset_page);
    assert_eq!(
        controls.actions.clear_filters_query,
        Some(query.clone().clear_filters())
    );
    assert_eq!(
        controls.actions.first_page_query,
        Some(query.first_page_query())
    );
    assert!(controls.page_window.is_empty());
    assert_eq!(controls.display_start_index, None);
    assert_eq!(controls.display_end_index, None);
    assert_eq!(controls.current_page_number, None);
    assert_eq!(controls.total_page_count, Some(0));
}
