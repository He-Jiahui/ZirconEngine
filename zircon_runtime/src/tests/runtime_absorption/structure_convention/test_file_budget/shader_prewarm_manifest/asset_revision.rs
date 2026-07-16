use super::*;

#[test]
fn runtime_15_shader_prewarm_asset_revision_export_is_wired() {
    let manifest = read_runtime_src("bin/zircon_shader_prewarm/manifest.rs");
    let revision = read_runtime_src("bin/zircon_shader_prewarm/manifest/revision.rs");
    let tests = read_runtime_src("bin/zircon_shader_prewarm/manifest/tests.rs");
    let raw_revision_tests =
        read_runtime_src("bin/zircon_shader_prewarm/manifest/tests/raw_revision.rs");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let template_doc = read_repo("docs/zircon_runtime/graphics/shader/template.md");
    let product_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "asset-root revision manifest wiring",
        &manifest,
        &[
            "mod revision;",
            "asset_scan_revision_from_source_digest",
            "asset_scan_revision_from_content_hashes",
            "shader_source_from_zmeta",
            "shader_prewarm_source",
            "meta.source_digest",
        ],
    );
    assert_contains_all(
        "asset-root revision owner",
        &revision,
        &[
            "ASSET_SCAN_INITIAL_RESOURCE_REVISION",
            "asset_scan_revision_from_source_digest",
            "asset_scan_revision_from_content_hashes",
            "blake3::Hasher",
            "non_zero_revision_from_hash",
        ],
    );
    assert_contains_all(
        "asset-root revision tests",
        &(tests.clone() + &raw_revision_tests),
        &[
            "shader_prewarm_asset_root_manifest_uses_zmeta_source_digest_revision",
            "shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision",
            "source-hash-a",
            "source-hash-b",
            "simple_a",
            "simple_b",
            "material_revision",
            "assert_ne!",
        ],
    );

    for (path, source) in [
        ("bin/zircon_shader_prewarm/manifest.rs", manifest.as_str()),
        (
            "bin/zircon_shader_prewarm/manifest/revision.rs",
            revision.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests.rs",
            tests.as_str(),
        ),
        (
            "bin/zircon_shader_prewarm/manifest/tests/raw_revision.rs",
            raw_revision_tests.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("template doc", template_doc.as_str()),
        ("render product submit doc", product_submit_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Asset-root shader edit revision export",
                "render_plan08_asset_root_shader_edit_revision_export_passed_cargo_renderdoc_deferred",
                "bin/zircon_shader_prewarm/manifest/revision.rs",
                "shader_prewarm_asset_root_manifest_uses_zmeta_source_digest_revision",
                "shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision",
                "runtime_15_shader_prewarm_asset_revision_export_is_wired",
                "live project registry exact revision overlay",
            ],
        );
    }
}
