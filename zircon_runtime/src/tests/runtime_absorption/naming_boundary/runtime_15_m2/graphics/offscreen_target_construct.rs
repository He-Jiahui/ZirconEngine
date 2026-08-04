use super::*;

#[test]
fn runtime_15_offscreen_target_construct_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let render_backend_dir = manifest_root.join("src/graphics/backend/render_backend");
    let retired_offscreen_target_new = render_backend_dir.join("offscreen_target_new");
    let offscreen_target_construct_dir = render_backend_dir.join("offscreen_target_construct");
    let render_backend_mod = read_text(
        &render_backend_dir.join("mod.rs"),
        "render backend module entry should be readable",
    );
    let construct_mod = read_text(
        &offscreen_target_construct_dir.join("mod.rs"),
        "offscreen target construct module entry should be readable",
    );
    let construct_owner = read_text(
        &offscreen_target_construct_dir.join("construct.rs"),
        "offscreen target construct owner should be readable",
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
    let graphics_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/graphics/render-product-submit.md",
    );

    assert!(
        !retired_offscreen_target_new.exists(),
        "render backend should not keep *_new construction owner directory {:?}",
        retired_offscreen_target_new
    );
    assert!(
        offscreen_target_construct_dir.is_dir(),
        "render backend should keep offscreen target construction in a construct-named directory"
    );
    assert_contains_all(
        "render backend module entry",
        &render_backend_mod,
        &["mod offscreen_target_construct;"],
    );
    assert!(
        !render_backend_mod.contains("mod offscreen_target_new;"),
        "render_backend/mod.rs should not preserve the retired offscreen_target_new module name"
    );
    assert_contains_all(
        "offscreen target construct module entry",
        &construct_mod,
        &[
            "mod construct;",
            "mod create_cluster_buffer;",
            "mod create_texture_bundle;",
            "mod texture_bundle;",
        ],
    );
    assert_contains_all(
        "offscreen target construct owner",
        &construct_owner,
        &[
            "impl OffscreenTarget",
            "pub(crate) fn new(",
            "zircon-offscreen-final-color",
            "create_cluster_buffer(device, cluster_buffer_bytes)",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("graphics render-product doc", graphics_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 offscreen target construct directory naming hard cutover",
                "runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result",
                "runtime_15_offscreen_target_construct_uses_owner_name",
            ],
        );
    }
    for (label, source) in [
        ("runtime index", runtime_index.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("graphics render-product doc", graphics_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &["graphics/backend/render_backend/offscreen_target_construct/construct.rs"],
        );
    }
}
