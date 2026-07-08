use std::fs;
use std::path::{Path, PathBuf};

use super::super::super::support::{assert_contains_all, read_repo_text, read_text};

#[test]
fn runtime_15_hybrid_gi_extract_scene_source_uses_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let hybrid_gi_root = manifest_root
        .parent()
        .expect("runtime crate should live under repository root")
        .join("zircon_plugins/hybrid_gi/runtime/src/hybrid_gi");
    let hybrid_gi_files = rust_files(&hybrid_gi_root);
    let collect_inputs = read_text(
        &hybrid_gi_root.join("renderer/gpu_resources/execute_prepare/execute/collect_inputs.rs"),
        "Hybrid GI collect-inputs source should be readable",
    );
    let probe_quantization = read_text(
        &hybrid_gi_root.join("renderer/gpu_resources/execute_prepare/probe_quantization.rs"),
        "Hybrid GI probe quantization source should be readable",
    );
    let trace_region_inputs = read_text(
        &hybrid_gi_root.join("renderer/gpu_resources/execute_prepare/trace_region_inputs.rs"),
        "Hybrid GI trace-region inputs source should be readable",
    );
    let runtime_parent_chain = read_text(
        &hybrid_gi_root
            .join("renderer/post_process_sources/encode_hybrid_gi_probes/runtime_parent_chain.rs"),
        "Hybrid GI runtime parent-chain source should be readable",
    );
    let hierarchy_weight = read_text(
        &hybrid_gi_root.join(
            "renderer/post_process_sources/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs",
        ),
        "Hybrid GI hierarchy resolve-weight source should be readable",
    );
    let audit_script = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/hard_cutover_migration_smells.py",
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

    assert!(
        !hybrid_gi_files.is_empty(),
        "Hybrid GI runtime source should contain Rust files"
    );
    for path in hybrid_gi_files {
        let source = read_text(&path, "Hybrid GI runtime Rust source should be readable");
        assert!(
            !source.contains("legacy"),
            "Hybrid GI runtime source {:?} should not keep hard-cutover legacy wording",
            path
        );
    }
    assert_contains_all(
        "Hybrid GI extract scene-source names",
        &(collect_inputs
            + "\n"
            + &probe_quantization
            + "\n"
            + &trace_region_inputs
            + "\n"
            + &runtime_parent_chain
            + "\n"
            + &hierarchy_weight),
        &[
            "extract_trace_region_ids",
            "extract-backed",
            "extract-sourced RenderHybridGiProbe",
            "extract_input",
        ],
    );
    assert!(
        !audit_script.contains("legacy-hybrid-gi-render-debt"),
        "hard-cutover audit should not retain the retired Hybrid GI legacy debt bucket"
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("render product submit doc", render_product_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 Hybrid GI extract scene-source naming hard cutover",
                "runtime_15_hybrid_gi_extract_scene_source_naming_hard_cutover_static_passed_cargo_deferred",
                "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi",
                "extract_trace_region_ids",
                "extract-backed",
                "extract-sourced RenderHybridGiProbe",
                "runtime_15_hybrid_gi_extract_scene_source_uses_current_names",
            ],
        );
    }
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("Hybrid GI runtime directory should be readable") {
        let entry = entry.expect("Hybrid GI runtime entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
