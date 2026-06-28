use std::{
    fs,
    path::{Path, PathBuf},
};

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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
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

#[test]
fn runtime_15_core_framework_render_fixtures_use_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let render_queue = read_text(
        &manifest_root.join("src/core/framework/render/core_pipeline/render_queue.rs"),
        "render queue fixture source should be readable",
    );
    let effect_stack = read_text(
        &manifest_root.join("src/core/framework/render/post_process/effect_stack_settings.rs"),
        "post-process effect stack fixture source should be readable",
    );
    let relevance = read_text(
        &manifest_root.join("src/core/framework/render/relevance.rs"),
        "render relevance fixture source should be readable",
    );
    let light_readiness = read_text(
        &manifest_root.join("src/core/framework/render/light/readiness.rs"),
        "render light readiness fixture source should be readable",
    );
    let scene_extract = read_text(
        &manifest_root.join("src/core/framework/render/scene_extract.rs"),
        "render scene extract fixture source should be readable",
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
    let common_render_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/core/framework/render/common_api.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary date slice should be readable",
    );

    assert_contains_all(
        "core framework render fixture current names",
        &render_queue,
        &["authored_queue_offsets_are_clamped_to_material_window"],
    );
    assert!(!render_queue.contains("authored_legacy_offsets"));
    assert_contains_all(
        "post-process effect stack fixture current names",
        &effect_stack,
        &["extended_effect_stack_settings_enable_product_node_without_retired_fields"],
    );
    assert!(!effect_stack.contains("without_legacy_fields"));
    assert_contains_all(
        "primitive relevance fixture current names",
        &relevance,
        &["primitive_relevance_preserves_layers_above_scene_schema_v1_mask_width"],
    );
    assert!(!relevance.contains("above_legacy_mask_width"));
    assert_contains_all(
        "typed scene-schema-v1 mask fixtures",
        &(light_readiness + "\n" + &scene_extract),
        &[
            "RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK)",
            "RenderLayerSet::from_scene_schema_v1_mask(u32::MAX)",
        ],
    );
    assert!(!scene_extract.contains("RenderLayerSet::from_legacy_mask(u32::MAX)"));

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("common render API doc", common_render_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 core framework render fixture naming hard cutover",
                "runtime_15_core_framework_render_fixture_naming_hard_cutover_static_passed_cargo_deferred",
                "core/framework/render/core_pipeline/render_queue.rs",
                "scene_schema_v1_mask",
                "runtime_15_core_framework_render_fixtures_use_current_names",
            ],
        );
    }
}

#[test]
fn runtime_15_render_layer_schema_v1_mask_api_uses_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let camera_source = read_text(
        &manifest_root.join("src/core/framework/render/camera.rs"),
        "render camera source should be readable",
    );
    let light_buffer = read_text(
        &manifest_root.join("src/graphics/scene/scene_renderer/lighting/light_buffer.rs"),
        "render light buffer source should be readable",
    );
    let build_runtime_frame = read_text(
        &manifest_root.join(
            "src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs",
        ),
        "runtime frame submit source should be readable",
    );
    let camera_history_key = read_text(
        &manifest_root
            .join("src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs"),
        "camera history key source should be readable",
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
    let camera_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/core/framework/render/camera.md",
    );
    let render_product_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/graphics/render-product-submit.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary date slice should be readable",
    );

    assert_contains_all(
        "RenderLayerSet schema-v1 mask API",
        &camera_source,
        &[
            "pub fn from_scene_schema_v1_mask(mask: u32) -> Self",
            "pub fn to_scene_schema_v1_mask_lossy(&self) -> u32",
            "pub fn intersects_scene_schema_v1_mask(&self, mask: u32) -> bool",
        ],
    );
    for retired_name in [
        "from_legacy_mask",
        "to_legacy_mask_lossy",
        "intersects_legacy_mask",
    ] {
        assert!(
            !camera_source.contains(retired_name),
            "RenderLayerSet should not keep retired `{retired_name}` helpers"
        );
    }

    for path in rust_files_except_naming_boundary(&manifest_root.join("src")) {
        let source = read_text(
            &path,
            "runtime Rust source should be readable for schema-v1 mask scan",
        );
        for retired_name in [
            "from_legacy_mask",
            "to_legacy_mask_lossy",
            "intersects_legacy_mask",
        ] {
            assert!(
                !source.contains(retired_name),
                "runtime source {:?} should not use retired `{retired_name}` render-layer helper",
                path
            );
        }
    }

    assert_contains_all(
        "render graphics callers use schema-v1 mask helpers",
        &(light_buffer + "\n" + &build_runtime_frame + "\n" + &camera_history_key),
        &[
            "to_scene_schema_v1_mask_lossy()",
            "RenderLayerSet::from_scene_schema_v1_mask",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("render camera doc", camera_doc),
        ("render product submit doc", render_product_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 render layer schema-v1 mask API naming hard cutover",
                "runtime_15_render_layer_schema_v1_mask_api_naming_hard_cutover_static_passed_cargo_deferred",
                "from_scene_schema_v1_mask",
                "to_scene_schema_v1_mask_lossy",
                "intersects_scene_schema_v1_mask",
                "runtime_15_render_layer_schema_v1_mask_api_uses_current_names",
            ],
        );
    }
}

fn rust_files_except_naming_boundary(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files_except_naming_boundary(root, &mut files);
    files.sort();
    files
}

fn collect_rust_files_except_naming_boundary(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("runtime source directory should be readable") {
        let entry = entry.expect("runtime source entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files_except_naming_boundary(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs")
            && !path_slash(&path).contains("/src/tests/runtime_absorption/naming_boundary/")
        {
            files.push(path);
        }
    }
}

fn path_slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
