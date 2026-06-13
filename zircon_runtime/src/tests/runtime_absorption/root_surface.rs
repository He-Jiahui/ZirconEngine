const LIB_RS: &str = include_str!("../../lib.rs");
const CORE_MOD_RS: &str = include_str!("../../core/mod.rs");
const ROOT_SURFACE_DOC: &str = include_str!("../../../../docs/zircon_runtime/core/root_surface.md");
const ROOT_SURFACE_M1_DOC: &str =
    include_str!("../../../../docs/engine-architecture/runtime-root-surface-m1.md");
const INTERFACE_CONVERGENCE_DOC: &str =
    include_str!("../../../../docs/engine-architecture/runtime-interface-convergence.md");
const RUNTIME_02_PLAN: &str =
    include_str!("../../../../docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md");
const RUNTIME_INDEX: &str = include_str!("../../../../docs/plans/zircon_runtime/runtime/index.md");

#[test]
fn runtime_crate_root_public_surface_stays_curated() {
    let public_modules: Vec<_> = LIB_RS
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|decl| decl.trim_end_matches(';'))
        .collect();
    let expected_modules = [
        "core",
        "diagnostic_log",
        "dynamic_api",
        "engine_module",
        "prelude",
        "animation",
        "asset",
        "scene",
        "ui",
        "graphics",
        "render_graph",
        "rhi",
        "rhi_wgpu",
        "builtin",
        "foundation",
        "input",
        "navigation",
        "platform",
        "plugin",
        "script",
    ];
    assert_eq!(public_modules.as_slice(), &expected_modules);

    let public_use_count = LIB_RS
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use "))
        .count();
    assert_eq!(
        public_use_count, 3,
        "crate root should keep only resource, reflection macro, and builtin report public re-export blocks"
    );

    for required in [
        "pub use crate::core::resource;",
        "pub use zircon_runtime_reflection_macros::{",
        "pub use builtin::{",
    ] {
        assert!(
            LIB_RS.contains(required),
            "crate root is missing expected curated re-export `{required}`"
        );
    }

    for forbidden in [
        "pub use graphics::",
        "pub use render_graph::",
        "pub use rhi::",
        "pub use rhi_wgpu::",
        "pub use ui::",
        "pub use input::",
        "pub use scene::",
        "pub use asset::",
        "pub use plugin::",
        "pub use core::{",
        "pub use crate::core::{",
    ] {
        assert!(
            !LIB_RS.contains(forbidden),
            "crate root must not flatten subsystem namespace `{forbidden}`"
        );
    }

    assert!(
        LIB_RS.lines().count() <= 80,
        "pre-M3 crate root should stay thin until the graphics alias block is removed"
    );
}

#[test]
fn graphics_alias_debt_is_private_and_documented_until_m3_cutover() {
    for current_private_alias in [
        "pub(crate) use graphics::pipeline::RendererFeatureReferenceListKind;",
        "pub(crate) use graphics::scene::{",
        "pub(crate) use graphics::{",
    ] {
        assert!(
            LIB_RS.contains(current_private_alias),
            "current M3 alias debt anchor `{current_private_alias}` moved without updating Runtime 02"
        );
    }

    assert!(
        !LIB_RS.contains("pub use graphics::"),
        "graphics alias debt must remain crate-private until the M3 hard cutover removes it"
    );
    assert!(
        LIB_RS.contains("#[allow(unused_imports)]"),
        "the pre-M3 alias debt marker should stay visible until the alias block is removed"
    );

    for required_plan_anchor in [
        "pre_m3_root_surface_guard_static_passed_pending_render_owner",
        "root_surface.md",
        "M3 `lib.rs` graphics alias",
        "render owner",
    ] {
        assert!(
            RUNTIME_02_PLAN.contains(required_plan_anchor),
            "Runtime 02 plan must document root-surface guard anchor `{required_plan_anchor}`"
        );
    }
    for required_index_anchor in [
        "root_surface_guard",
        "root_surface.md",
        "M3 lib.rs graphics alias",
    ] {
        assert!(
            RUNTIME_INDEX.contains(required_index_anchor),
            "runtime index must document root-surface guard anchor `{required_index_anchor}`"
        );
    }
}

#[test]
fn graphics_type_alias_debt_has_m3_2_pre_guard_until_render_cutover() {
    let current_type_alias_symbols = [
        "RendererFeatureReferenceListKind",
        "GraphicsError",
        "SceneRenderer",
        "WgpuRenderFramework",
        "ViewportFrame",
        "HybridGiRuntimeProvider",
        "VirtualGeometryRuntimeProvider",
        "SolariRuntimeProvider",
    ];
    for current_type_alias_symbol in current_type_alias_symbols {
        assert!(
            LIB_RS.contains(current_type_alias_symbol),
            "M3.2 type alias debt symbol `{current_type_alias_symbol}` moved without updating Runtime 02"
        );
    }

    let crate_private_graphics_alias_blocks = LIB_RS
        .lines()
        .filter(|line| line.trim_start().starts_with("pub(crate) use graphics"))
        .count();
    assert_eq!(
        crate_private_graphics_alias_blocks, 3,
        "M3.2 pre-guard expects exactly the current crate-private graphics alias blocks"
    );
    assert!(
        !LIB_RS
            .lines()
            .any(|line| line.trim_start().starts_with("pub use graphics")),
        "M3.2 type alias debt must not become a public root graphics export"
    );

    for required_doc_anchor in [
        "M3.2 Type Alias Debt",
        "pre_m3_type_alias_guard_static_passed_pending_render_owner",
        "crate-private type alias debt",
        "SceneRenderer",
        "WgpuRenderFramework",
        "render owner",
    ] {
        assert!(
            ROOT_SURFACE_DOC.contains(required_doc_anchor),
            "root surface doc must document M3.2 type alias pre-guard anchor `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "pre_m3_type_alias_guard_static_passed_pending_render_owner",
        "M3.2 type alias debt",
        "actual type alias deletion not executed",
        "render owner",
    ] {
        assert!(
            RUNTIME_02_PLAN.contains(required_plan_anchor),
            "Runtime 02 plan must document M3.2 type alias pre-guard anchor `{required_plan_anchor}`"
        );
    }

    for required_index_anchor in [
        "pre_m3_type_alias_guard_static_passed_pending_render_owner",
        "M3.2 type alias debt",
        "root_surface_guard",
    ] {
        assert!(
            RUNTIME_INDEX.contains(required_index_anchor),
            "runtime index must document M3.2 type alias pre-guard anchor `{required_index_anchor}`"
        );
    }
}

#[test]
fn core_spine_and_root_surface_docs_stay_in_sync() {
    let core_modules: Vec<_> = CORE_MOD_RS
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|decl| decl.trim_end_matches(';'))
        .collect();
    let expected_core_modules = ["runtime", "framework", "manager", "math", "resource"];
    assert_eq!(core_modules.as_slice(), &expected_core_modules);

    for retired_root_module in [
        "pub mod config_store;",
        "pub mod diagnostics;",
        "pub mod event_bus;",
        "pub mod frame_clock;",
        "pub mod job_scheduler;",
        "pub mod lifecycle;",
        "pub mod modules;",
        "pub mod state;",
        "pub mod tasks;",
    ] {
        assert!(
            !CORE_MOD_RS.contains(retired_root_module),
            "core root must not recreate retired module `{retired_root_module}`"
        );
    }

    for required_doc_anchor in [
        "Current Public Root Surface",
        "20 public module declarations",
        "three public `pub use` sites",
        "M3 Alias Debt",
        "not a public API",
        "root_surface_guard_static_passed",
    ] {
        assert!(
            ROOT_SURFACE_DOC.contains(required_doc_anchor),
            "root surface doc is missing anchor `{required_doc_anchor}`"
        );
    }
}

#[test]
fn root_surface_m1_gate_matches_runtime_14_module_family_seats() {
    for required_m1_anchor in [
        "public_module_count = 20",
        "module_decision_count = 20",
        "unclassified_public_module_count = 0",
        "root_surface_migration_debt_count = 3",
        "crate_visible_graphics_reexport_count = 80",
        "`runtime-module-entry`: `animation`, `asset`, `diagnostic_log`, `foundation`, `input`, `navigation`, `platform`, `scene`, `script`, `ui`",
        "Runtime 14 classifies `animation` and `navigation`",
        "They are runtime module entries, not unclassified root modules.",
        "crate-visible graphics re-export fan-out: 80 symbols",
    ] {
        assert!(
            ROOT_SURFACE_M1_DOC.contains(required_m1_anchor),
            "runtime root-surface M1 gate doc is missing current audit anchor `{required_m1_anchor}`"
        );
    }

    for stale_m1_anchor in [
        "public_module_count = 18",
        "module_decision_count = 18",
        "crate_visible_graphics_reexport_count = 75",
        "crate-visible graphics re-export fan-out: 75 symbols",
        "unclassified runtime root modules:",
        "unclassified-root-module",
    ] {
        assert!(
            !ROOT_SURFACE_M1_DOC.contains(stale_m1_anchor),
            "runtime root-surface M1 gate doc still contains stale audit anchor `{stale_m1_anchor}`"
        );
    }
}

#[test]
fn root_surface_interface_convergence_mirror_uses_current_audit_counts() {
    for required_convergence_anchor in [
        "The runtime crate root should remain a narrow module entry surface.",
        "20 public modules",
        "3 public `pub use` locations",
        "80 crate-visible graphics re-export symbols",
        "direct `rhi_wgpu` backend exposure",
        "M1 gate status `migration-debt-present`",
        "runtime root public modules 20/20",
        "crate-visible graphics alias debt 80/80",
        "root_surface guard tests 6/6",
    ] {
        assert!(
            INTERFACE_CONVERGENCE_DOC.contains(required_convergence_anchor),
            "interface convergence doc is missing root-surface audit anchor `{required_convergence_anchor}`"
        );
    }

    for stale_convergence_anchor in [
        "17 public modules",
        "75 crate-visible graphics re-export symbols",
        "root_surface guard tests 5/5",
    ] {
        assert!(
            !INTERFACE_CONVERGENCE_DOC.contains(stale_convergence_anchor),
            "interface convergence doc still contains stale root-surface anchor `{stale_convergence_anchor}`"
        );
    }
}
