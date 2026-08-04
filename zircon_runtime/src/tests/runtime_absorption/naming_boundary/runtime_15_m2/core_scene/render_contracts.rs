use super::*;

#[test]
fn runtime_15_render_shader_definition_uses_bare_flag_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shader_definition_source = read_text(
        &manifest_root.join("src/core/framework/render/shader/definition_value.rs"),
        "render shader definition value source should be readable",
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
    let render_assets_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/asset/render-assets.md");
    let zmeta_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/asset/zmeta-shader-material.md",
    );

    assert_contains_all(
        "render shader definition bare flag serde branch",
        &shader_definition_source,
        &[
            "BareFlag(String)",
            "DefinitionValueRepr::BareFlag(name) => Self::from(name)",
        ],
    );
    assert!(
        !shader_definition_source.contains("LegacyFlag"),
        "render shader definition value serde branch should not use legacy naming"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("zmeta shader material doc", zmeta_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 render shader definition bare-flag naming hard cutover",
                "runtime_15_render_shader_definition_bare_flag_naming_hard_cutover_static_passed_cargo_deferred",
                "BareFlag",
                "runtime_15_render_shader_definition_uses_bare_flag_names",
            ],
        );
    }
}

#[test]
fn runtime_15_frame_extract_snapshot_adapter_uses_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let frame_extract_source = read_text(
        &manifest_root.join("src/core/framework/render/frame_extract.rs"),
        "render frame extract source should be readable",
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
    let scene_render_extract_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/scene/render_extract.md");

    assert_contains_all(
        "frame extract snapshot adapter source names",
        &frame_extract_source,
        &[
            "Builds a frame DTO from the scene viewport snapshot packet for preview,",
            "pub fn from_snapshot(world: RenderWorldSnapshotHandle, snapshot: RenderSceneSnapshot)",
            "from a `SceneViewportRenderPacket`",
        ],
    );
    assert!(
        !frame_extract_source.contains("legacy viewport packet"),
        "RenderFrameExtract::from_snapshot should describe the snapshot adapter without legacy packet wording"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "scene render extract doc",
            scene_render_extract_doc.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 frame extract snapshot adapter naming hard cutover",
                "runtime_15_frame_extract_snapshot_adapter_naming_hard_cutover_static_passed_cargo_deferred",
                "core/framework/render/frame_extract.rs",
                "scene viewport snapshot packet",
                "runtime_15_frame_extract_snapshot_adapter_uses_current_names",
            ],
        );
    }
}
