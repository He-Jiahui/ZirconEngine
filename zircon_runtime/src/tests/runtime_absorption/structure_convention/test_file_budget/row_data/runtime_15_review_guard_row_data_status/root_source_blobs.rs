use super::*;

pub(super) fn status_doc_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in STATUS_DOC_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn status_doc_root_source_blob() -> String {
    [
        ROOT_PATHS_PATH,
        ROOT_STATUSES_PATH,
        ROOT_CHILD_ROWS_PATH,
        ROOT_SOURCE_BLOBS_PATH,
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n")
}

pub(super) fn status_doc_full_source_blob() -> String {
    [
        status_doc_root_source_blob(),
        status_doc_child_source_blob(),
    ]
    .join("\n")
}

pub(super) fn review_guard_status_support_source_blob() -> String {
    read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_CORE_ROWS_PATH)
}

pub(super) fn status_support_review_guard_source_blob() -> String {
    [
        STATUS_SUPPORT_REVIEW_GUARD_BASE_ROWS_PATH,
        STATUS_SUPPORT_REVIEW_GUARD_STATUS_DOC_ROWS_PATH,
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n")
}

pub(super) fn review_guard_status_map_source_blob() -> String {
    read_runtime_src(REVIEW_GUARD_TYPED_ERROR_ROW_DATA_STATUS_MAP_PATH)
}

pub(super) fn review_guard_date_map_source_blob() -> String {
    read_runtime_src(REVIEW_GUARD_TYPED_ERROR_ROW_DATA_DATE_MAP_PATH)
}

pub(super) fn status_support_status_map_source_blob() -> String {
    [
        STATUS_SUPPORT_ROW_DATA_BASE_STATUS_MAP_PATH,
        STATUS_SUPPORT_ROW_DATA_STATUS_DOC_STATUS_MAP_PATH,
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n")
}

pub(super) fn status_support_date_map_source_blob() -> String {
    [
        STATUS_SUPPORT_ROW_DATA_BASE_DATE_MAP_PATH,
        STATUS_SUPPORT_ROW_DATA_STATUS_DOC_DATE_MAP_PATH,
    ]
    .iter()
    .map(|path| read_runtime_src(path))
    .collect::<Vec<_>>()
    .join("\n")
}
