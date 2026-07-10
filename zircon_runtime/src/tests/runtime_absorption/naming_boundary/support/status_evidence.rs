use std::path::Path;

use super::{read_repo_text, read_text};

pub(in super::super) fn read_runtime_15_naming_status_rows(manifest_root: &Path) -> String {
    read_runtime_test_children(
        manifest_root,
        "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m2",
        &[
            "core_scene_asset_dynamic.rs",
            "render_graphics.rs",
            "row_data_owner.rs",
            "ui_platform_editor.rs",
        ],
    )
}

pub(in super::super) fn read_runtime_15_naming_status_map(manifest_root: &Path) -> String {
    read_runtime_15_naming_map_evidence(
        manifest_root,
        "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary",
    )
}

pub(in super::super) fn read_runtime_15_naming_date_map(manifest_root: &Path) -> String {
    read_runtime_15_naming_map_evidence(
        manifest_root,
        "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary",
    )
}

fn read_runtime_15_naming_map_evidence(manifest_root: &Path, relative_directory: &str) -> String {
    [
        read_runtime_test_children(
            manifest_root,
            relative_directory,
            &[
                "core_bootstrap.rs",
                "plugin_ui_platform.rs",
                "render_graphics.rs",
                "scene_asset_runtime.rs",
            ],
        ),
        read_repo_text(
            manifest_root,
            "docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        ),
    ]
    .join("\n")
}

fn read_runtime_test_children(
    manifest_root: &Path,
    relative_directory: &str,
    child_files: &[&str],
) -> String {
    child_files
        .iter()
        .map(|child| {
            read_text(
                &manifest_root.join(relative_directory).join(child),
                "runtime naming-boundary status child should be readable",
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
