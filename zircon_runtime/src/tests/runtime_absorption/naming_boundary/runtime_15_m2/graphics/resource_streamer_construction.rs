use super::*;

#[test]
fn runtime_15_resource_streamer_construction_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let resource_streamer_dir =
        manifest_root.join("src/graphics/scene/resources/resource_streamer");
    let retired_resource_streamer_new = resource_streamer_dir.join("resource_streamer_new.rs");
    let resource_streamer_mod = read_text(
        &resource_streamer_dir.join("mod.rs"),
        "resource streamer module entry should be readable",
    );
    let resource_streamer_construction = read_text(
        &resource_streamer_dir.join("resource_streamer_construction.rs"),
        "resource streamer construction owner should be readable",
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
        !retired_resource_streamer_new.exists(),
        "resource streamer should not keep *_new construction owner file {:?}",
        retired_resource_streamer_new
    );
    assert_contains_all(
        "resource streamer module entry",
        &resource_streamer_mod,
        &["mod resource_streamer_construction;"],
    );
    assert!(
        !resource_streamer_mod.contains("mod resource_streamer_new;"),
        "resource_streamer/mod.rs should not preserve the retired resource_streamer_new module name"
    );
    assert_contains_all(
        "resource streamer construction owner",
        &resource_streamer_construction,
        &[
            "impl ResourceStreamer",
            "pub(crate) fn new(",
            "fallback_texture: Arc::new",
            "OutputTargetWritebackConverter::new(device)",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("graphics render-product doc", graphics_doc),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 resource streamer construction module naming hard cutover",
                "runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/scene/resources/resource_streamer/resource_streamer_construction.rs",
                "runtime_15_resource_streamer_construction_uses_owner_name",
            ],
        );
    }
}
