use std::path::Path;

use super::super::{assert_contains_all, read_repo_text, read_text};

#[test]
fn runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_watch_dir = manifest_root.join("src/asset/watch");
    let retired_drop_impl = asset_watch_dir.join("drop_impl.rs");
    let asset_watch_mod = read_text(
        &asset_watch_dir.join("mod.rs"),
        "asset watcher module entry should be readable",
    );
    let shutdown_on_drop = read_text(
        &asset_watch_dir.join("shutdown_on_drop.rs"),
        "asset watcher shutdown-on-drop owner should be readable",
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
    let asset_watcher_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/watcher.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_drop_impl.exists(),
        "asset watcher should not keep banned-name module file {:?}",
        retired_drop_impl
    );
    assert_contains_all(
        "asset watcher module entry",
        &asset_watch_mod,
        &["mod shutdown_on_drop;"],
    );
    assert!(
        !asset_watch_mod.contains("mod drop_impl;"),
        "asset/watch/mod.rs should not preserve the banned drop_impl module name"
    );
    assert_contains_all(
        "asset watcher shutdown-on-drop owner",
        &shutdown_on_drop,
        &[
            "impl Drop for AssetWatcher",
            "self.stop_tx.send(())",
            "join.join()",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("asset watcher doc", asset_watcher_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 asset watcher shutdown-on-drop module naming hard cutover",
                "runtime_15_asset_watcher_shutdown_on_drop_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/watch/shutdown_on_drop.rs",
                "runtime_15_asset_watcher_shutdown_on_drop_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_asset_change_construction_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_watch_dir = manifest_root.join("src/asset/watch");
    let retired_asset_change_new = asset_watch_dir.join("asset_change_new.rs");
    let asset_watch_mod = read_text(
        &asset_watch_dir.join("mod.rs"),
        "asset watcher module entry should be readable",
    );
    let asset_change_construction = read_text(
        &asset_watch_dir.join("asset_change_construction.rs"),
        "asset change construction owner should be readable",
    );
    let fold_events = read_text(
        &asset_watch_dir.join("fold_events.rs"),
        "asset watcher fold events owner should be readable",
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
    let asset_watcher_doc = read_repo_text(manifest_root, "docs/zircon_runtime/asset/watcher.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_asset_change_new.exists(),
        "asset watcher should not keep *_new construction owner file {:?}",
        retired_asset_change_new
    );
    assert_contains_all(
        "asset watcher module entry",
        &asset_watch_mod,
        &["mod asset_change_construction;"],
    );
    assert!(
        !asset_watch_mod.contains("mod asset_change_new;"),
        "asset/watch/mod.rs should not preserve the retired asset_change_new module name"
    );
    assert_contains_all(
        "asset change construction owner",
        &asset_change_construction,
        &[
            "impl AssetChange",
            "pub fn new(",
            "kind: AssetChangeKind",
            "previous_uri: Option<AssetUri>",
        ],
    );
    assert_contains_all(
        "asset watcher fold events owner",
        &fold_events,
        &["AssetChange::new("],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("asset watcher doc", asset_watcher_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 asset change construction module naming hard cutover",
                "runtime_15_asset_change_construction_naming_hard_cutover_static_passed_cargo_deferred",
                "asset/watch/asset_change_construction.rs",
                "runtime_15_asset_change_construction_uses_owner_name",
            ],
        );
    }
}

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
    let render_assets_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

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
fn runtime_15_scene_ecs_query_cached_queries_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ecs_query_dir = manifest_root.join("src/scene/tests/ecs_query");
    let retired_cache_helpers = ecs_query_dir.join("cache_helpers.rs");
    let ecs_query_parent = read_text(
        &manifest_root.join("src/scene/tests/ecs_query.rs"),
        "scene ECS query test module parent should be readable",
    );
    let cached_queries = read_text(
        &ecs_query_dir.join("cached_queries.rs"),
        "scene ECS query cached queries owner should be readable",
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
    let scene_ecs_doc = read_repo_text(manifest_root, "docs/zircon_runtime/scene/ecs.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_cache_helpers.exists(),
        "scene ECS query tests should not keep banned-name module file {:?}",
        retired_cache_helpers
    );
    assert_contains_all(
        "scene ECS query test parent",
        &ecs_query_parent,
        &["mod cached_queries;"],
    );
    assert!(
        !ecs_query_parent.contains("mod cache_helpers;"),
        "scene/tests/ecs_query.rs should not preserve the banned cache_helpers module name"
    );
    assert_contains_all(
        "scene ECS query cached queries owner",
        &cached_queries,
        &[
            "fn query_state_cached_iteration_rebuilds_only_for_structural_changes",
            "fn query_state_count_and_empty_helpers_can_use_cached_candidates",
            "fn query_state_cached_direct_iteration_reads_storage_locations",
            "fn query_state_cached_direct_iteration_reads_sparse_locations",
            "fn query_state_cached_archetypes_do_not_require_optional_reads",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("scene ECS doc", scene_ecs_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 scene ECS query cached queries module naming hard cutover",
                "runtime_15_scene_ecs_query_cached_queries_naming_hard_cutover_static_passed_cargo_deferred",
                "scene/tests/ecs_query/cached_queries.rs",
                "runtime_15_scene_ecs_query_cached_queries_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests_dir = manifest_root.join("src/dynamic_api/session/tests");
    let retired_helpers = tests_dir.join("helpers.rs");
    let tests_mod = read_text(
        &tests_dir.join("mod.rs"),
        "dynamic API session tests module parent should be readable",
    );
    let vampire_runtime_support = read_text(
        &tests_dir.join("vampire_runtime_support.rs"),
        "dynamic API vampire runtime support owner should be readable",
    );
    let frame_diagnostics = read_text(
        &tests_dir.join("frame_diagnostics.rs"),
        "dynamic API frame diagnostics tests should be readable",
    );
    let vampire_gameplay = read_text(
        &tests_dir.join("vampire_gameplay.rs"),
        "dynamic API vampire gameplay tests should be readable",
    );
    let vampire_hud = read_text(
        &tests_dir.join("vampire_hud.rs"),
        "dynamic API vampire HUD tests should be readable",
    );
    let vampire_menu = read_text(
        &tests_dir.join("vampire_menu.rs"),
        "dynamic API vampire menu tests should be readable",
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
    let dynamic_api_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/dynamic_api/session.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status map should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date map should be readable",
    );

    assert!(
        !retired_helpers.exists(),
        "dynamic API session tests should not keep banned-name module file {:?}",
        retired_helpers
    );
    assert_contains_all(
        "dynamic API session tests module parent",
        &tests_mod,
        &["mod vampire_runtime_support;"],
    );
    assert!(
        !tests_mod.contains("mod helpers;"),
        "dynamic_api/session/tests/mod.rs should not preserve the banned helpers module name"
    );
    assert_contains_all(
        "dynamic API vampire runtime support owner",
        &vampire_runtime_support,
        &[
            "fn vampire_project_config",
            "fn start_vampire_game",
            "fn count_hud_panel_pixels",
            "fn diagnostic_current",
            "fn small_headless_frame_request",
        ],
    );

    for (label, source) in [
        ("frame diagnostics tests", frame_diagnostics.as_str()),
        ("vampire gameplay tests", vampire_gameplay.as_str()),
        ("vampire HUD tests", vampire_hud.as_str()),
        ("vampire menu tests", vampire_menu.as_str()),
    ] {
        assert_contains_all(label, source, &["super::vampire_runtime_support::*"]);
        assert!(
            !source.contains("super::helpers::*"),
            "{label} should not import the retired helpers owner"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("dynamic API session doc", dynamic_api_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
        ("expected status map", expected_status.as_str()),
        ("expected date map", expected_date.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M2 dynamic API vampire runtime support module naming hard cutover",
                "runtime_15_dynamic_api_vampire_runtime_support_naming_hard_cutover_static_passed_cargo_deferred",
                "dynamic_api/session/tests/vampire_runtime_support.rs",
                "runtime_15_dynamic_api_vampire_runtime_support_uses_owner_name",
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
    let render_assets_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/asset/render-assets.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let expected_status = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let expected_date = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

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
