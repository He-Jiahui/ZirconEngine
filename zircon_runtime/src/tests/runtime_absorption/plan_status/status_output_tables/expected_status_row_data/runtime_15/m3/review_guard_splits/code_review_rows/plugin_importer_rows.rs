use super::Slice;

#[path = "plugin_importer_rows/review_guards.rs"]
mod review_guards;
#[path = "plugin_importer_rows/row_data_owner.rs"]
mod row_data_owner;
#[path = "plugin_importer_rows/source_inventory.rs"]
mod source_inventory;
#[path = "plugin_importer_rows/status_docs.rs"]
mod status_docs;
#[path = "plugin_importer_rows/structure_assertions.rs"]
mod structure_assertions;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 code review findings plugin-importer DX structure guard child-owner split",
        review_guards::PLUGIN_IMPORTER_DX_STRUCTURE_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX review guard child-owner split",
        review_guards::PLUGIN_IMPORTER_DX_REVIEW_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer D13 SDK review guard child-owner split",
        review_guards::PLUGIN_IMPORTER_D13_REVIEW_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX status-doc guard child-owner split",
        status_docs::PLUGIN_IMPORTER_DX_STATUS_DOCS_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX status-doc guard folder-backed split",
        status_docs::PLUGIN_IMPORTER_DX_STATUS_DOCS_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX status-doc root inventory child split",
        status_docs::PLUGIN_IMPORTER_DX_STATUS_DOCS_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX source inventory guard child-owner split",
        source_inventory::PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX source inventory guard folder-backed split",
        source_inventory::PLUGIN_IMPORTER_DX_SOURCE_INVENTORY_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX structure assertions guard child-owner split",
        structure_assertions::PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX structure assertions guard folder-backed split",
        structure_assertions::PLUGIN_IMPORTER_DX_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer DX review mounts guard folder-backed split",
        structure_assertions::PLUGIN_IMPORTER_DX_REVIEW_MOUNTS_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard folder-backed split",
        structure_assertions::PLUGIN_IMPORTER_D13_SDK_STRUCTURE_ASSERTIONS_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer D13 SDK parent-mount guard child split",
        structure_assertions::PLUGIN_IMPORTER_D13_SDK_PARENT_MOUNTS_GUARD_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer D13 SDK structure assertions guard child-owner split",
        structure_assertions::PLUGIN_IMPORTER_D13_SDK_STRUCTURE_ASSERTIONS_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 plugin-importer row-data owner child split",
        row_data_owner::PLUGIN_IMPORTER_ROWS_ROW_DATA_OWNER_CHILD_SPLIT,
    ),
];
