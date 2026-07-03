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
    let graphics_doc = read_repo_text(
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
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("graphics render-product doc", graphics_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 offscreen target construct directory naming hard cutover",
                "runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result",
                "graphics/backend/render_backend/offscreen_target_construct/construct.rs",
                "runtime_15_offscreen_target_construct_uses_owner_name",
            ],
        );
    }
}
