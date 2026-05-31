use super::*;

#[test]
fn material_management_query_result_state_classifies_empty_filtered_and_populated_pages() {
    let empty_records = RenderMaterialManagementRecordSet::from_records(Vec::new());
    let default_query = RenderMaterialManagementQuery::new();
    let empty_result = empty_records.query(default_query.clone());
    let empty_state = empty_result.state(&default_query);

    assert_eq!(
        empty_state.kind,
        RenderMaterialManagementQueryResultStateKind::EmptyRecordSet
    );
    assert!(empty_state.is_record_set_empty);
    assert!(!empty_state.is_filter_empty);
    assert!(!empty_state.has_filtered_records);
    assert!(!empty_state.has_page_records);

    let records = vec![record(
        "material:alpha",
        Some("Alpha Ready"),
        RenderMaterialReadinessStatus::Ready,
    )];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let filtered_query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Fallback)
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(10)));
    let filtered_result = record_set.query(filtered_query.clone());
    let filtered_state = filtered_result.state(&filtered_query);

    assert_eq!(
        filtered_state.kind,
        RenderMaterialManagementQueryResultStateKind::EmptyFilteredSet
    );
    assert!(filtered_state.is_filter_empty);
    assert!(filtered_state.query_state.has_active_filters);
    assert_eq!(filtered_state.total_count, 0);
    assert_eq!(filtered_state.returned_count, 0);

    let populated_query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Ready)
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(10)));
    let populated_result = record_set.query(populated_query.clone());
    let populated_state = populated_result.state(&populated_query);

    assert_eq!(
        populated_state.kind,
        RenderMaterialManagementQueryResultStateKind::PopulatedPage
    );
    assert!(populated_state.has_filtered_records);
    assert!(populated_state.has_page_records);
    assert_eq!(populated_state.total_count, 1);
    assert_eq!(populated_state.returned_count, 1);
}

#[test]
fn material_management_query_result_state_classifies_page_edge_cases() {
    let records = vec![record(
        "material:alpha",
        Some("Alpha Ready"),
        RenderMaterialReadinessStatus::Ready,
    )];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);

    let zero_limit_query = RenderMaterialManagementQuery::new()
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(0)));
    let zero_limit_result = record_set.query(zero_limit_query.clone());
    let zero_limit_state = zero_limit_result.state(&zero_limit_query);

    assert_eq!(
        zero_limit_state.kind,
        RenderMaterialManagementQueryResultStateKind::EmptyPage
    );
    assert!(zero_limit_state.is_empty_page);
    assert!(zero_limit_state.has_filtered_records);
    assert!(!zero_limit_state.has_page_records);
    assert_eq!(zero_limit_state.total_count, 1);
    assert_eq!(zero_limit_state.returned_count, 0);

    let out_of_range_query = RenderMaterialManagementQuery::new()
        .with_page(RenderMaterialManagementPageRequest::new(10, Some(5)));
    let out_of_range_result = record_set.query(out_of_range_query.clone());
    let out_of_range_state = out_of_range_result.state(&out_of_range_query);

    assert_eq!(
        out_of_range_state.kind,
        RenderMaterialManagementQueryResultStateKind::PageOutOfRange
    );
    assert!(out_of_range_state.is_page_out_of_range);
    assert!(out_of_range_state.has_filtered_records);
    assert!(!out_of_range_state.has_page_records);
    assert_eq!(out_of_range_state.total_count, 1);
    assert_eq!(out_of_range_state.returned_count, 0);
}

#[test]
fn material_management_query_selection_exposes_result_state() {
    let records = vec![record(
        "material:alpha",
        Some("Alpha Ready"),
        RenderMaterialReadinessStatus::Ready,
    )];
    let record_set = RenderMaterialManagementRecordSet::from_records(records);
    let query = RenderMaterialManagementQuery::new()
        .with_status(RenderMaterialReadinessStatus::Ready)
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(1)));

    let query_selection = record_set.query_selection(query);
    let state = query_selection.result_state();

    assert_eq!(
        state.kind,
        RenderMaterialManagementQueryResultStateKind::PopulatedPage
    );
    assert!(state.query_state.has_status_filter);
    assert_eq!(state.total_count, 1);
    assert_eq!(state.returned_count, 1);
}
