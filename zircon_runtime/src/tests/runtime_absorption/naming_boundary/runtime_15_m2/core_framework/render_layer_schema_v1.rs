use std::{
    fs,
    path::{Path, PathBuf},
};

use super::*;

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
    let camera_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/core/framework/render/camera.md",
    );
    let render_product_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/graphics/render-product-submit.md",
    );
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let status_slice = read_runtime_15_naming_status_map(manifest_root);
    let date_slice = read_runtime_15_naming_date_map(manifest_root);

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
