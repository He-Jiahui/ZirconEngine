use super::*;

#[test]
fn runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let texture_readiness_dir =
        manifest_root.join("src/asset/tests/assets/texture_upload_readiness");
    let retired_common = texture_readiness_dir.join("common.rs");
    let texture_readiness_parent = read_text(
        &manifest_root.join("src/asset/tests/assets/texture_upload_readiness.rs"),
        "texture upload readiness parent should be readable",
    );
    let container_fixtures = read_text(
        &texture_readiness_dir.join("container_fixtures.rs"),
        "texture upload readiness container fixtures owner should be readable",
    );
    let boundaries = read_text(
        &texture_readiness_dir.join("boundaries.rs"),
        "texture upload readiness boundaries tests should be readable",
    );
    let containers = read_text(
        &texture_readiness_dir.join("containers.rs"),
        "texture upload readiness container tests should be readable",
    );
    let dds = read_text(
        &texture_readiness_dir.join("dds.rs"),
        "texture upload readiness DDS tests should be readable",
    );
    let ktx = read_text(
        &texture_readiness_dir.join("ktx.rs"),
        "texture upload readiness KTX tests should be readable",
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
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let expected_status = read_runtime_15_naming_status_map(manifest_root);
    let expected_date = read_runtime_15_naming_date_map(manifest_root);

    assert!(
        !retired_common.exists(),
        "texture upload readiness tests should not keep banned-name module file {:?}",
        retired_common
    );
    assert_contains_all(
        "texture upload readiness parent",
        &texture_readiness_parent,
        &["mod container_fixtures;"],
    );
    assert!(
        !texture_readiness_parent.contains("mod common;"),
        "texture_upload_readiness.rs should not preserve the banned common module name"
    );
    assert_contains_all(
        "texture upload readiness container fixtures owner",
        &container_fixtures,
        &[
            "fn dds_classic_fourcc_bytes",
            "fn ktx1_compressed_level_bytes",
            "fn astc_container_bytes",
            "const KTX2_TEST_LEVEL_DATA_OFFSET",
        ],
    );

    for (label, source) in [
        ("texture readiness boundaries tests", boundaries.as_str()),
        ("texture readiness container tests", containers.as_str()),
        ("texture readiness DDS tests", dds.as_str()),
        ("texture readiness KTX tests", ktx.as_str()),
    ] {
        assert_contains_all(label, source, &["super::container_fixtures::*"]);
        assert!(
            !source.contains("super::common::*"),
            "{label} should not import the retired common owner"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 asset texture upload readiness container fixtures module naming hard cutover",
                "runtime_15_asset_texture_upload_readiness_container_fixtures_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/tests/assets/texture_upload_readiness/container_fixtures.rs",
                "runtime_15_asset_texture_upload_readiness_container_fixtures_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_dds_upload_policy_uses_classic_container_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dds_upload_support = read_text(
        &manifest_root.join("src/asset/assets/texture/upload_support/dds.rs"),
        "DDS upload support owner should be readable",
    );
    let texture_readiness_dir =
        manifest_root.join("src/asset/tests/assets/texture_upload_readiness");
    let container_fixtures = read_text(
        &texture_readiness_dir.join("container_fixtures.rs"),
        "texture upload readiness container fixtures should be readable",
    );
    let readiness_boundaries = read_text(
        &texture_readiness_dir.join("boundaries.rs"),
        "texture upload readiness boundaries tests should be readable",
    );
    let readiness_containers = read_text(
        &texture_readiness_dir.join("containers.rs"),
        "texture upload readiness container tests should be readable",
    );
    let readiness_dds = read_text(
        &texture_readiness_dir.join("dds.rs"),
        "texture upload readiness DDS tests should be readable",
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
    let status_rows = read_runtime_15_naming_status_rows(manifest_root);
    let expected_status = read_runtime_15_naming_status_map(manifest_root);
    let expected_date = read_runtime_15_naming_date_map(manifest_root);

    assert_contains_all(
        "DDS upload support owner",
        &dds_upload_support,
        &[
            "dds_classic_fourcc_upload_layout",
            "classic_faces",
            "classic_header_cubemap",
        ],
    );
    assert_contains_all(
        "texture upload readiness container fixtures",
        &container_fixtures,
        &[
            "dds_classic_fourcc_bytes",
            "dds_classic_mip_bytes",
            "dds_classic_cubemap_bytes",
        ],
    );
    for (label, source) in [
        (
            "texture upload readiness boundaries",
            readiness_boundaries.as_str(),
        ),
        (
            "texture upload readiness containers",
            readiness_containers.as_str(),
        ),
        ("texture upload readiness DDS", readiness_dds.as_str()),
    ] {
        assert_contains_all(label, source, &["dds_classic_"]);
    }
    for retired in [
        concat!("dds_", "legacy_", "upload_layout"),
        concat!("dds_", "legacy_", "bytes"),
        concat!("dds_", "legacy_", "mip_bytes"),
        concat!("dds_", "legacy_", "cubemap_bytes"),
        concat!("legacy_", "faces"),
        concat!("legacy_", "cubemap"),
    ] {
        for (label, source) in [
            ("DDS upload support", dds_upload_support.as_str()),
            (
                "texture upload readiness fixtures",
                container_fixtures.as_str(),
            ),
            (
                "texture upload readiness boundaries",
                readiness_boundaries.as_str(),
            ),
            (
                "texture upload readiness containers",
                readiness_containers.as_str(),
            ),
            ("texture upload readiness DDS", readiness_dds.as_str()),
        ] {
            assert!(
                !source.contains(retired),
                "{label} should not keep retired DDS container policy name {retired}"
            );
        }
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("render assets doc", render_assets_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 DDS upload policy naming hard cutover",
                "runtime_15_dds_upload_policy_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/assets/texture/upload_support/dds.rs",
                "dds_classic_fourcc_upload_layout",
                "runtime_15_dds_upload_policy_uses_classic_container_names",
            ],
        );
    }
}
