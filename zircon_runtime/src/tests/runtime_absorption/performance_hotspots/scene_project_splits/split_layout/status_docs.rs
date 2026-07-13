use super::sources::{assert_contains_all, SplitLayoutSources};

const SLICE: &str = "Runtime 15 M3 Runtime 07 scene/project split-layout guard folder-backed split";
const STATUS: &str =
    "runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_static_passed_cargo_deferred";
const GUARD: &str = "runtime_15_runtime_07_scene_project_split_layout_guard_folder_backed_split";
const LEGACY_SLICE: &str = "Runtime 15 M3 Runtime 07 scene/project guard child-owner split";
const LEGACY_STATUS: &str =
    "runtime_15_runtime_07_scene_project_guard_child_owner_split_static_passed_cargo_deferred";
const LEGACY_GUARD: &str = "runtime_15_runtime_07_scene_project_guard_child_owner_split";

pub(super) fn assert_scene_project_split_docs(sources: &SplitLayoutSources) {
    for (label, source) in [("Runtime 07 numbered archive", sources.runtime_07_archive)] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                "scene_project_splits/split_layout",
                GUARD,
                "expected_test_file_count = 65",
            ],
        );
    }

    for (label, source) in [("Runtime 07 numbered archive", sources.runtime_07_archive)] {
        assert_contains_all(
            label,
            source,
            &[
                LEGACY_SLICE,
                LEGACY_STATUS,
                "scene_project_splits/scene_asset.rs",
                LEGACY_GUARD,
            ],
        );
    }
}
