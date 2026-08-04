use super::*;

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
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
    );
    let runtime_index = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let camera_controller_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/core/framework/camera_controller.md",
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

    let docs = [];
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
