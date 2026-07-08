use super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 typed-error native plugin loader structure guard child-owner split",
        super::structure_assertions::NATIVE_PLUGIN_LOADER_GUARD_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error native plugin loader structure guard folder-backed split",
        super::structure_assertions::NATIVE_PLUGIN_LOADER_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error native plugin loader routes child split",
        super::structure_assertions::NATIVE_PLUGIN_LOADER_ROUTES_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error native plugin loader routes source helper child split",
        super::structure_assertions::NATIVE_PLUGIN_LOADER_ROUTES_SOURCE_HELPER_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error native plugin loader source helper child split",
        super::structure_assertions::NATIVE_PLUGIN_LOADER_SOURCE_HELPER_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error structure moved-guard absence child-owner split",
        super::structure_assertions::MOVED_GUARD_ABSENCE_CHILD_OWNER_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error moved-guard absence guard folder-backed split",
        super::structure_assertions::MOVED_GUARD_ABSENCE_GUARD_FOLDER_BACKED_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error moved-guard absence parent-backflow child split",
        super::structure_assertions::MOVED_GUARD_ABSENCE_PARENT_BACKFLOW_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error moved-guard absence root inventory child split",
        super::structure_assertions::MOVED_GUARD_ABSENCE_ROOT_INVENTORY_CHILD_SPLIT,
    ),
    (
        "Runtime 15 M3 typed-error moved-guard absence child-owner route split",
        super::structure_assertions::MOVED_GUARD_ABSENCE_CHILD_OWNER_ROUTE_SPLIT,
    ),
    super::map_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    super::map_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
];
