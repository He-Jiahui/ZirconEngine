use super::*;

pub(super) const MODULE_LAYOUT_GUARD_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "status-output row-data parent",
        STATUS_OUTPUT_ROW_DATA_PARENT_PATH,
        400,
    ),
    ("module-layout parent", MODULE_LAYOUT_PARENT_PATH, 80),
    ("module-layout root paths", ROOT_PATHS_PATH, 100),
    ("module-layout root statuses", ROOT_STATUSES_PATH, 80),
    ("module-layout root child rows", ROOT_CHILD_ROWS_PATH, 100),
    ("module-layout root owner paths", ROOT_OWNER_PATHS_PATH, 140),
    (
        "module-layout root inventory",
        ROOT_INVENTORY_GUARD_PATH,
        100,
    ),
    (
        "module-layout delegation child",
        MODULE_LAYOUT_CHILDREN[0].1,
        180,
    ),
    (
        "module-layout child-summary child",
        MODULE_LAYOUT_CHILDREN[1].1,
        160,
    ),
    (
        "module-layout status-mirror child",
        MODULE_LAYOUT_CHILDREN[3].1,
        180,
    ),
    (
        "module-layout budget child",
        MODULE_LAYOUT_CHILDREN[4].1,
        80,
    ),
    (
        "module-layout child-summary parent",
        MODULE_LAYOUT_CHILD_SUMMARIES_PATH,
        400,
    ),
    (
        "module-layout child-summary status-doc child",
        MODULE_LAYOUT_CHILD_SUMMARY_STATUS_DOCS_PATH,
        400,
    ),
    (
        "module-layout status-doc child",
        MODULE_LAYOUT_STATUS_DOCS_PATH,
        400,
    ),
    (
        "module-layout status row data",
        PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_ROWS_PATH,
        220,
    ),
];
