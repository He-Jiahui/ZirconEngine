use super::*;

#[test]
fn material_management_query_page_info_derives_navigation_requests() {
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
        .with_status(RenderMaterialReadinessStatus::Ready)
        .with_text_filter("ready")
        .with_sort_order(RenderMaterialManagementSortOrder::new(
            RenderMaterialManagementSortKey::MaterialName,
            RenderMaterialManagementSortDirection::Ascending,
        ))
        .with_page(RenderMaterialManagementPageRequest::new(0, Some(1)));

    let first_page = record_set.query(query.clone());
    assert_eq!(
        first_page.page.page_request(),
        RenderMaterialManagementPageRequest::new(0, Some(1))
    );
    assert_eq!(
        first_page.page.next_page_request(),
        Some(RenderMaterialManagementPageRequest::new(1, Some(1)))
    );
    assert_eq!(first_page.page.previous_page_request(), None);
    assert_eq!(
        first_page.records[0].material_name.as_deref(),
        Some("Alpha Ready")
    );

    let second_query = query
        .next_page_query(first_page.page)
        .expect("first page should have a next query");
    assert_eq!(
        second_query.page,
        RenderMaterialManagementPageRequest::new(1, Some(1))
    );
    assert_eq!(second_query.status, query.status);
    assert_eq!(second_query.text_filter, query.text_filter);
    assert_eq!(second_query.sort_order, query.sort_order);

    let second_page = record_set.query(second_query.clone());
    assert_eq!(
        second_page.page.previous_page_request(),
        Some(RenderMaterialManagementPageRequest::new(0, Some(1)))
    );
    assert_eq!(
        second_page.page.next_page_request(),
        Some(RenderMaterialManagementPageRequest::new(2, Some(1)))
    );
    assert_eq!(
        second_page.records[0].material_name.as_deref(),
        Some("Beta Ready")
    );

    let previous_query = second_query
        .previous_page_query(second_page.page)
        .expect("second page should have a previous query");
    assert_eq!(
        previous_query.page,
        RenderMaterialManagementPageRequest::new(0, Some(1))
    );
    assert_eq!(previous_query.status, query.status);
    assert_eq!(previous_query.text_filter, query.text_filter);

    let third_query = second_query
        .next_page_query(second_page.page)
        .expect("second page should have a next query");
    let third_page = record_set.query(third_query);
    assert_eq!(
        third_page.records[0].material_name.as_deref(),
        Some("Gamma Ready")
    );
    assert_eq!(third_page.page.next_page_request(), None);
    assert_eq!(
        third_page.page.previous_page_request(),
        Some(RenderMaterialManagementPageRequest::new(1, Some(1)))
    );

    let all_page = record_set.query(
        query
            .clone()
            .with_page(RenderMaterialManagementPageRequest::all()),
    );
    assert_eq!(all_page.page.next_page_request(), None);
    assert_eq!(all_page.page.previous_page_request(), None);
    assert!(query.next_page_query(all_page.page).is_none());
    assert!(query.previous_page_query(all_page.page).is_none());

    let zero_limit_page = record_set.query(
        query
            .clone()
            .with_page(RenderMaterialManagementPageRequest::new(1, Some(0))),
    );
    assert!(zero_limit_page.page.has_previous_page);
    assert!(zero_limit_page.page.has_next_page);
    assert_eq!(zero_limit_page.page.next_page_request(), None);
    assert_eq!(zero_limit_page.page.previous_page_request(), None);
    assert!(query.next_page_query(zero_limit_page.page).is_none());
    assert!(query.previous_page_query(zero_limit_page.page).is_none());
}

#[test]
fn material_management_page_info_reports_window_and_page_numbers() {
    let first_page = RenderMaterialManagementPageInfo::from_page_request(
        RenderMaterialManagementPageRequest::new(0, Some(2)),
        5,
        2,
    );
    assert_eq!(
        first_page.window(),
        RenderMaterialManagementPageWindow {
            start_index: 0,
            end_index_exclusive: 2,
        }
    );
    assert_eq!(first_page.window().len(), 2);
    assert_eq!(first_page.display_start_index(), Some(1));
    assert_eq!(first_page.display_end_index(), Some(2));
    assert_eq!(first_page.current_page_number(), Some(1));
    assert_eq!(first_page.total_page_count(), Some(3));

    let middle_page = RenderMaterialManagementPageInfo::from_page_request(
        RenderMaterialManagementPageRequest::new(2, Some(2)),
        5,
        2,
    );
    assert_eq!(
        middle_page.window(),
        RenderMaterialManagementPageWindow {
            start_index: 2,
            end_index_exclusive: 4,
        }
    );
    assert_eq!(middle_page.display_start_index(), Some(3));
    assert_eq!(middle_page.display_end_index(), Some(4));
    assert_eq!(middle_page.current_page_number(), Some(2));
    assert_eq!(middle_page.total_page_count(), Some(3));

    let final_page = RenderMaterialManagementPageInfo::from_page_request(
        RenderMaterialManagementPageRequest::new(4, Some(2)),
        5,
        1,
    );
    assert_eq!(
        final_page.window(),
        RenderMaterialManagementPageWindow {
            start_index: 4,
            end_index_exclusive: 5,
        }
    );
    assert_eq!(final_page.display_start_index(), Some(5));
    assert_eq!(final_page.display_end_index(), Some(5));
    assert_eq!(final_page.current_page_number(), Some(3));
    assert_eq!(final_page.total_page_count(), Some(3));

    let empty_page = RenderMaterialManagementPageInfo::from_page_request(
        RenderMaterialManagementPageRequest::new(0, Some(2)),
        0,
        0,
    );
    assert!(empty_page.window().is_empty());
    assert_eq!(empty_page.display_start_index(), None);
    assert_eq!(empty_page.display_end_index(), None);
    assert_eq!(empty_page.current_page_number(), None);
    assert_eq!(empty_page.total_page_count(), Some(0));

    let offset_past_end = RenderMaterialManagementPageInfo::from_page_request(
        RenderMaterialManagementPageRequest::new(8, Some(2)),
        5,
        0,
    );
    assert_eq!(
        offset_past_end.window(),
        RenderMaterialManagementPageWindow {
            start_index: 5,
            end_index_exclusive: 5,
        }
    );
    assert_eq!(offset_past_end.display_start_index(), None);
    assert_eq!(offset_past_end.display_end_index(), None);
    assert_eq!(offset_past_end.current_page_number(), None);
    assert_eq!(offset_past_end.total_page_count(), Some(3));

    let all_page = RenderMaterialManagementPageInfo::from_page_request(
        RenderMaterialManagementPageRequest::all(),
        5,
        5,
    );
    assert_eq!(all_page.display_start_index(), Some(1));
    assert_eq!(all_page.display_end_index(), Some(5));
    assert_eq!(all_page.current_page_number(), None);
    assert_eq!(all_page.total_page_count(), None);

    let zero_limit_page = RenderMaterialManagementPageInfo::from_page_request(
        RenderMaterialManagementPageRequest::new(0, Some(0)),
        5,
        0,
    );
    assert_eq!(zero_limit_page.display_start_index(), None);
    assert_eq!(zero_limit_page.display_end_index(), None);
    assert_eq!(zero_limit_page.current_page_number(), None);
    assert_eq!(zero_limit_page.total_page_count(), None);
}
