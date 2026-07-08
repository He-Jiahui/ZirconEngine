use super::*;

pub(super) fn status_support_plan_doc_child_paths() -> Vec<&'static str> {
    let mut paths = STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILDREN.to_vec();
    paths.extend(
        STATUS_SUPPORT_RUNTIME_INDEX_ANCHOR_ROUTE_CHILDREN
            .iter()
            .copied(),
    );
    paths.extend(
        STATUS_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN
            .iter()
            .copied(),
    );
    paths
}

pub(super) fn date_support_plan_doc_child_paths() -> Vec<&'static str> {
    let mut paths = DATE_SUPPORT_PLAN_DOC_ROUTE_CHILDREN.to_vec();
    paths.extend(
        DATE_SUPPORT_RUNTIME_INDEX_ANCHOR_ROUTE_CHILDREN
            .iter()
            .copied(),
    );
    paths.extend(
        DATE_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN
            .iter()
            .copied(),
    );
    paths
}
