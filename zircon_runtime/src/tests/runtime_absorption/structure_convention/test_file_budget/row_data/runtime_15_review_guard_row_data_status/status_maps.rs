use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_doc_maps_are_current() {
    let review_expected_status_map = review_guard_status_map_source_blob();
    let review_expected_date_map = review_guard_date_map_source_blob();
    let status_support_expected_status_map = status_support_status_map_source_blob();
    let status_support_expected_date_map = status_support_date_map_source_blob();

    assert_contains_all(
        "Runtime 15 review expected status map records row-data split",
        &review_expected_status_map,
        &[
            "Runtime 15 M3 review guard status row-data child-owner split",
            "runtime_15_review_guard_status_row_data_child_owner_split_static_passed_cargo_deferred",
            TOPIC_CHILD_OWNER_STATUS_NAME,
            TOPIC_CHILD_OWNER_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected status map records review-guard status-doc split",
        &status_support_expected_status_map,
        &[
            REVIEW_GUARD_CHILD_OWNER_STATUS_NAME,
            REVIEW_GUARD_CHILD_OWNER_STATUS_ID,
            STATUS_DOC_CHILD_OWNER_STATUS_NAME,
            STATUS_DOC_CHILD_OWNER_STATUS_ID,
            FOLDER_BACKED_STATUS_NAME,
            FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 review expected date map records row-data split",
        &review_expected_date_map,
        &[
            "Runtime 15 M3 review guard status row-data child-owner split",
            "2026-06-30",
            TOPIC_CHILD_OWNER_STATUS_NAME,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records review-guard status-doc split",
        &status_support_expected_date_map,
        &[
            REVIEW_GUARD_CHILD_OWNER_STATUS_NAME,
            STATUS_DOC_CHILD_OWNER_STATUS_NAME,
            FOLDER_BACKED_STATUS_NAME,
            "2026-07-03",
        ],
    );
}
