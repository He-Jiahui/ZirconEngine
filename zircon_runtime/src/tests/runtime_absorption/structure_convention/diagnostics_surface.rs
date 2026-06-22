use super::{assert_contains_all, repo_path, runtime_src_path};

#[test]
fn runtime_15_diagnostics_use_frame_trait_without_world_wrapper() {
    let diagnostics_mod = read_runtime_src("core/runtime/diagnostics/mod.rs");
    let frame_diagnostics = read_runtime_src("core/runtime/diagnostics/frame_diagnostics.rs");
    let diagnostics_snapshot = read_runtime_src("core/runtime/diagnostics/snapshot.rs");
    let render_diagnostics = read_runtime_src("core/runtime/diagnostics/render.rs");
    let physics_diagnostics = read_runtime_src("core/runtime/diagnostics/physics.rs");
    let animation_diagnostics = read_runtime_src("core/runtime/diagnostics/animation.rs");
    let ecs_frame_diagnostics = read_runtime_src("scene/ecs/frame_performance_diagnostics.rs");
    let world = read_runtime_src("scene/world/world.rs");
    let world_performance_diagnostics = read_runtime_src("scene/world/performance_diagnostics.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let diagnostics_doc = read_repo("docs/zircon_runtime/core/diagnostics.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "runtime diagnostics frame trait",
        &diagnostics_mod,
        &[
            "mod frame_diagnostics;",
            "pub use frame_diagnostics::{FrameDiagnostics, FrameDiagnosticsStatus};",
        ],
    );
    assert_contains_all(
        "frame diagnostics trait owner",
        &frame_diagnostics,
        &[
            "pub struct FrameDiagnosticsStatus",
            "pub trait FrameDiagnostics",
            "fn diagnostics_domain(&self) -> &'static str",
            "fn frame_diagnostics_status(&self) -> FrameDiagnosticsStatus<'_>",
        ],
    );
    assert_contains_all(
        "runtime diagnostics snapshot subdomain composition",
        &diagnostics_snapshot,
        &[
            "pub fn frame_diagnostics_statuses(&self) -> [FrameDiagnosticsStatus<'_>; 3]",
            "self.render.frame_diagnostics_status()",
            "self.physics.frame_diagnostics_status()",
            "self.animation.frame_diagnostics_status()",
            "runtime_snapshot_frame_diagnostics_statuses_preserve_subdomains",
        ],
    );
    for (label, source, domain) in [
        (
            "render diagnostics",
            render_diagnostics.as_str(),
            "\"render\"",
        ),
        (
            "physics diagnostics",
            physics_diagnostics.as_str(),
            "\"physics\"",
        ),
        (
            "animation diagnostics",
            animation_diagnostics.as_str(),
            "\"animation\"",
        ),
        (
            "ECS frame diagnostics",
            ecs_frame_diagnostics.as_str(),
            "\"scene.ecs\"",
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "impl FrameDiagnostics for",
                "fn diagnostics_domain(&self) -> &'static str",
                domain,
            ],
        );
    }
    assert_contains_all(
        "world ECS frame diagnostics direct storage",
        &world,
        &[
            "EcsFramePerformanceDiagnostics",
            "pub(super) ecs_frame_performance_diagnostics: EcsFramePerformanceDiagnostics",
        ],
    );
    assert!(
        !world.contains("WorldEcsFramePerformanceDiagnostics"),
        "World should not carry a World* wrapper around EcsFramePerformanceDiagnostics"
    );
    assert!(
        !world_performance_diagnostics.contains("struct WorldEcsFramePerformanceDiagnostics"),
        "world performance diagnostics should not define a pure wrapper for ECS frame diagnostics"
    );
    assert!(
        !world_performance_diagnostics.contains(".0"),
        "world performance diagnostics should call EcsFramePerformanceDiagnostics directly"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("diagnostics doc", diagnostics_doc.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F14 diagnostics normalization",
                "runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed",
                "runtime_15_diagnostics_use_frame_trait_without_world_wrapper",
            ],
        );
    }
}

#[test]
fn runtime_15_diagnostics_guard_is_folder_backed() {
    let parent = read_runtime_src("tests/runtime_absorption/structure_convention.rs");
    let child =
        read_runtime_src("tests/runtime_absorption/structure_convention/diagnostics_surface.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs",
    );

    assert_contains_all(
        "structure convention parent diagnostics mount",
        &parent,
        &[
            "#[path = \"structure_convention/diagnostics_surface.rs\"]",
            "mod diagnostics_surface;",
        ],
    );
    let moved_guard = "fn runtime_15_diagnostics_use_frame_trait_without_world_wrapper";
    assert!(
        !parent.contains(moved_guard),
        "top-level structure_convention.rs should mount diagnostics guards instead of defining {moved_guard}"
    );
    assert!(
        child.contains(moved_guard),
        "diagnostics_surface.rs should own moved guard {moved_guard}"
    );

    let parent_lines = parent.lines().count();
    assert!(
        parent_lines < 80,
        "structure_convention.rs should remain a thin aggregator after diagnostics split; got {parent_lines} lines"
    );
    let child_lines = child.lines().count();
    assert!(
        child_lines < 500,
        "diagnostics_surface.rs should stay below the local guard module limit; got {child_lines} lines"
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
                "Runtime 15 M3 diagnostics guard module split",
                "runtime_15_diagnostics_guard_module_split_static_passed_cargo_lock_blocked",
                "structure_convention/diagnostics_surface.rs",
                "runtime_15_diagnostics_guard_is_folder_backed",
                "runtime_15_diagnostics_use_frame_trait_without_world_wrapper",
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
