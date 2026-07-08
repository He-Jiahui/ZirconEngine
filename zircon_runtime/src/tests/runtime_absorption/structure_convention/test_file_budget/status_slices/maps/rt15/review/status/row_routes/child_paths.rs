use super::*;

pub(super) fn status_support_row_data_child_paths() -> Vec<&'static str> {
    STATUS_SUPPORT_ROW_DATA_ROUTE_CHILDREN
        .iter()
        .chain(STATUS_SUPPORT_CHILD_GROUP_ROW_DATA_ROUTE_CHILDREN.iter())
        .chain(STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILDREN.iter())
        .copied()
        .collect()
}

pub(super) fn date_support_row_data_child_paths() -> Vec<&'static str> {
    DATE_SUPPORT_ROW_DATA_ROUTE_CHILDREN
        .iter()
        .chain(DATE_SUPPORT_CHILD_GROUP_ROW_DATA_ROUTE_CHILDREN.iter())
        .chain(DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILDREN.iter())
        .copied()
        .collect()
}
