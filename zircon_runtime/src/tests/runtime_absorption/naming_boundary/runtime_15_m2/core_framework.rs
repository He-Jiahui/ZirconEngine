use std::path::Path;

use super::super::{assert_contains_all, read_repo_text, read_text};

#[test]
fn runtime_15_camera_controller_output_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let camera_controller_dir = manifest_root.join("src/core/framework/camera_controller");
    let retired_common = camera_controller_dir.join("common.rs");
    let camera_controller_mod = read_text(
        &camera_controller_dir.join("mod.rs"),
        "camera controller module entry should be readable",
    );
    let controller_output = read_text(
        &camera_controller_dir.join("controller_output.rs"),
        "camera controller output owner should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let camera_controller_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/core/framework/camera_controller.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
        ),
        "Runtime 15 expected status row data should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert!(
        !retired_common.exists(),
        "camera controller should not keep banned-name module file {:?}",
        retired_common
    );
    assert_contains_all(
        "camera controller module entry",
        &camera_controller_mod,
        &[
            "mod controller_output;",
            "pub use controller_output::{CameraControllerOutput, CursorGrabIntent, CursorGrabMode};",
        ],
    );
    assert!(
        !camera_controller_mod.contains("mod common;"),
        "camera_controller/mod.rs should not preserve the banned common module name"
    );
    assert!(
        !camera_controller_mod.contains("pub use common"),
        "camera_controller/mod.rs should not re-export through the retired common owner"
    );
    assert_contains_all(
        "camera controller output owner",
        &controller_output,
        &[
            "pub enum CursorGrabMode",
            "pub struct CursorGrabIntent",
            "pub struct CameraControllerOutput",
            "pub fn unchanged",
            "pub fn from_transform",
            "pub fn with_cursor_grab",
        ],
    );

    let docs = [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("camera controller doc", camera_controller_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ];
    for (label, source) in docs {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 camera controller output module naming hard cutover",
                "runtime_15_camera_controller_output_naming_hard_cutover_static_passed_cargo_deferred",
                "core/framework/camera_controller/controller_output.rs",
                "runtime_15_camera_controller_output_uses_owner_name",
            ],
        );
    }
}
