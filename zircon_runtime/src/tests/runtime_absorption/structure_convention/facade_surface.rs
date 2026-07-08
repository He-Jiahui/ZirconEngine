use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_prelude_covers_required_types() {
    let crate_prelude = read_runtime_src("prelude.rs");
    let asset_mod = read_runtime_src("asset/mod.rs");
    let scene_mod = read_runtime_src("scene/mod.rs");
    let ui_mod = read_runtime_src("ui/mod.rs");
    let graphics_mod = read_runtime_src("graphics/mod.rs");
    let asset_prelude = read_runtime_src("asset/prelude.rs");
    let scene_prelude = read_runtime_src("scene/prelude.rs");
    let ui_prelude = read_runtime_src("ui/prelude.rs");
    let graphics_prelude = read_runtime_src("graphics/prelude.rs");
    let prelude_tests = read_runtime_src("tests/prelude.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "crate prelude subsystem aggregation",
        &crate_prelude,
        &[
            "pub use crate::asset::prelude::*;",
            "pub use crate::scene::prelude::*;",
            "pub use crate::ui::prelude::*;",
            "pub use crate::graphics::prelude::*;",
        ],
    );

    for (label, module_source) in [
        ("asset", asset_mod.as_str()),
        ("scene", scene_mod.as_str()),
        ("ui", ui_mod.as_str()),
        ("graphics", graphics_mod.as_str()),
    ] {
        assert!(
            module_source.contains("pub mod prelude;"),
            "{label} module should expose a subsystem prelude before crate-level aggregation"
        );
    }

    assert_contains_all(
        "asset prelude required gameplay and authoring imports",
        &asset_prelude,
        &[
            "TextureAssetDescriptor",
            "AssetLoadState",
            "Assets",
            "Handle",
            "AssetManager",
            "ProjectAssetManager",
            "RGBA8_UNORM_SRGB_FORMAT",
        ],
    );
    assert_contains_all(
        "scene prelude required ECS imports",
        &scene_prelude,
        &[
            "World",
            "EntityId",
            "Bundle",
            "Component",
            "Resource",
            "Commands",
            "Query",
            "Res",
            "ResMut",
            "SceneError",
            "SceneResult",
            "SystemStage",
            "Schedule",
        ],
    );
    assert_contains_all(
        "ui prelude required surface/template imports",
        &ui_prelude,
        &[
            "UiSurface",
            "UiTreeId",
            "UiConfig",
            "UiModule",
            "UiTemplateLoader",
            "UiV2DocumentCompiler",
            "UiV2SurfaceBuilder",
        ],
    );
    assert_contains_all(
        "graphics prelude required render imports",
        &graphics_prelude,
        &[
            "GraphicsModule",
            "WgpuRenderFramework",
            "RenderPipelineAsset",
            "RenderFeatureDescriptor",
            "ViewportFrame",
            "ViewportRenderRegion",
            "GraphicsError",
        ],
    );
    assert_contains_all(
        "prelude behavior test",
        &prelude_tests,
        &["runtime_prelude_exports_asset_scene_ui_and_graphics_contracts"],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F9 runtime prelude required type coverage",
                "runtime_15_prelude_required_types_coremin_check_passed",
                "runtime_15_prelude_covers_required_types",
                "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred",
            ],
        );
    }
    let f9_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F9 |"))
        .expect("F9 review findings top row");
    assert!(
        f9_row.contains(
            "f8_f9_f10_runtime_surface_top_row_closed_status_static_passed_cargo_deferred"
        ) && f9_row.ends_with("| Runtime 15 / review closed |"),
        "F9 top row should record runtime surface review closed status"
    );
}

#[test]
fn runtime_15_mixed_visibility_has_facade_note() {
    let graphics_mod = read_runtime_src("graphics/mod.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "graphics facade visibility notes",
        &graphics_mod,
        &[
            "Crate-private implementation owners",
            "Public module entries",
            "Public facade exports",
            "Crate-visible bridge",
            "Test-only access",
            "pub(crate) mod backend;",
            "pub(crate) mod scene;",
            "pub mod prelude;",
            "pub mod runtime_builtin_graphics;",
        ],
    );
    for public_leak in ["pub mod backend;", "pub mod scene;", "pub mod types;"] {
        assert!(
            !graphics_mod.contains(public_leak),
            "graphics facade should not expose implementation module entry {public_leak}"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 graphics facade visibility note",
                "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift",
                "runtime_15_mixed_visibility_has_facade_note",
            ],
        );
    }
}

#[test]
fn runtime_15_graphics_facade_visibility_review_findings_mirror_is_recorded() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
    );
    let expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation/core_cleanup.rs",
    );
    let expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation/core_cleanup.rs",
    );

    let slice = "Runtime 15 M1 graphics facade visibility review findings mirror";
    let status =
        "runtime_15_graphics_facade_visibility_review_findings_mirror_static_passed_cargo_deferred";
    let original_status =
        "runtime_15_graphics_facade_visibility_note_static_passed_cargo_blocked_graphics_drift";
    let guard = "runtime_15_graphics_facade_visibility_review_findings_mirror_is_recorded";
    let review_doc = "docs/plans/engine-code-review-findings-2026-06.md";

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                slice,
                status,
                original_status,
                review_doc,
                "runtime_15_mixed_visibility_has_facade_note",
                guard,
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 expected status map",
        &expected_status_map,
        &[slice, status],
    );
    assert_contains_all(
        "Runtime 15 expected date map",
        &expected_date_map,
        &[slice, "2026-07-01"],
    );
}

#[test]
fn runtime_15_facade_surface_guard_is_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let child = read_runtime_src("tests/runtime_absorption/structure_convention/facade_surface.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "structure convention parent facade surface mount",
        &parent,
        &[
            "#[path = \"structure_convention/facade_surface.rs\"]",
            "mod facade_surface;",
        ],
    );

    for moved_guard in [
        "fn runtime_15_prelude_covers_required_types",
        "fn runtime_15_mixed_visibility_has_facade_note",
        "fn runtime_15_graphics_facade_visibility_review_findings_mirror_is_recorded",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "top-level structure_convention.rs should mount facade surface guards instead of defining {moved_guard}"
        );
        assert!(
            child.contains(moved_guard),
            "facade_surface.rs should own moved guard {moved_guard}"
        );
    }

    let parent_lines = parent.lines().count();
    assert!(
        parent_lines < 500,
        "structure_convention.rs should remain a small aggregator after facade surface split; got {parent_lines} lines"
    );
    let child_lines = child.lines().count();
    assert!(
        child_lines < 700,
        "facade_surface.rs should stay below the local guard module limit; got {child_lines} lines"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 facade surface guard module split",
                "runtime_15_facade_surface_guard_module_split_static_passed_cargo_lock_blocked",
                "structure_convention/facade_surface.rs",
                "runtime_15_facade_surface_guard_is_folder_backed",
                "runtime_15_prelude_covers_required_types",
            ],
        );
    }
}

fn read_runtime_src(relative: &str) -> String {
    std::fs::read_to_string(runtime_src_path(relative))
        .unwrap_or_else(|error| panic!("failed to read runtime source `{relative}`: {error}"))
}

fn read_repo(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read repository file `{relative}`: {error}"))
}
