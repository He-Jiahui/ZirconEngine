use super::*;

pub(super) const FOUNDATION_ROW_DATA_GUARD_OWNER_PATHS: &[(&str, &str, usize)] = &[
    (
        "runtime_15 row-data guard",
        RUNTIME_15_ROW_DATA_GUARD_PATH,
        800,
    ),
    (
        "foundation row-data guard",
        FOUNDATION_ROW_DATA_GUARD_PATH,
        90,
    ),
    ("foundation root paths", ROOT_PATHS_PATH, 120),
    ("foundation root statuses", ROOT_STATUSES_PATH, 80),
    ("foundation root child rows", ROOT_CHILD_ROWS_PATH, 120),
    ("foundation root owner paths", ROOT_OWNER_PATHS_PATH, 180),
    ("foundation root inventory", ROOT_INVENTORY_GUARD_PATH, 100),
    (
        "top-level expected status row data",
        TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH,
        800,
    ),
    (
        "Runtime 15 expected status row data",
        RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH,
        800,
    ),
    (
        "Runtime 15 foundation expected status row data",
        RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH,
        800,
    ),
    ("foundation core rows", FOUNDATION_CORE_ROWS_PATH, 800),
    (
        "foundation typed-error runtime rows",
        FOUNDATION_TYPED_ERROR_RUNTIME_ROWS_PATH,
        800,
    ),
    (
        "foundation typed-error plugin rows",
        FOUNDATION_TYPED_ERROR_PLUGIN_ROWS_PATH,
        800,
    ),
    (
        "foundation typed-error scene/asset rows",
        FOUNDATION_TYPED_ERROR_SCENE_ASSET_ROWS_PATH,
        800,
    ),
    (
        "foundation production guard support rows",
        PRODUCTION_GUARD_SUPPORT_FOUNDATION_ROWS_PATH,
        180,
    ),
];
