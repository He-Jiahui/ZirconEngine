const LIB_RS: &str = include_str!("../../lib.rs");
const PRELUDE_RS: &str = include_str!("../../prelude.rs");
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
        "builtin",
        "foundation",
        "input",
        "navigation",
        "platform",
        "plugin",
        "script",
    ];
    assert_eq!(public_modules.as_slice(), &expected_modules);
    assert!(
        LIB_RS.contains("mod rhi_wgpu;"),
        "Runtime 02 hard-cutover keeps the WGPU backend module crate-private behind the RHI owner"
    );
    assert!(
        !LIB_RS.contains("pub mod rhi_wgpu;"),
        "the concrete WGPU backend must not return to the runtime crate-root public surface"
    );

    let public_use_count = LIB_RS
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use "))
        .count();
    assert_eq!(
        public_use_count, 2,
        "crate root should keep only resource and reflection macro public re-export blocks"
    );

    for required in [
        "pub use crate::core::resource;",
        "pub use zircon_runtime_reflection_macros::{",
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
        "pub use builtin::{",
        "pub use crate::builtin::{",
        "pub use plugin::",
        "pub use core::{",
        "pub use crate::core::{",
    ] {
        assert!(
            !LIB_RS.contains(forbidden),
            "crate root must not flatten subsystem namespace `{forbidden}`"
        );
    }
    for builtin_symbol in [
        "RuntimeModuleLoadReport",
        "RuntimePluginId",
        "RuntimeRequiredPluginMissing",
        "RuntimeTargetMode",
    ] {
        assert!(
            !LIB_RS.contains(builtin_symbol),
            "Runtime 02 builtin facade cutover keeps `{builtin_symbol}` under `zircon_runtime::builtin`"
        );
        assert!(
            !PRELUDE_RS.contains(builtin_symbol),
            "Runtime 02 builtin facade cutover must not re-expose `{builtin_symbol}` through the runtime prelude"
        );
    }

    assert!(
        LIB_RS.lines().count() <= 45,
        "crate root should stay thin after the Runtime 02 M3 graphics alias block cutover"
    );
}

#[test]
fn graphics_alias_debt_is_removed_from_runtime_root() {
    for retired_private_alias in [
        "pub(crate) use graphics::pipeline::RendererFeatureReferenceListKind;",
        "pub(crate) use graphics::scene::{",
        "pub(crate) use graphics::{",
    ] {
        assert!(
            !LIB_RS.contains(retired_private_alias),
            "Runtime 02 M3 cutover removed `{retired_private_alias}` from the crate root"
        );
    }

    assert!(
        !LIB_RS.contains("pub use graphics::"),
        "graphics symbols must not be flattened as public root exports"
    );
    assert!(
        !LIB_RS.contains("#[allow(unused_imports)]"),
        "crate root should not need unused-import allowances after removing graphics aliases"
    );

    for required_plan_anchor in [
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "root_surface.md",
        "M3 `lib.rs` graphics alias removal",
        "crate-visible graphics alias debt 0/0",
    ] {
        assert!(
            RUNTIME_02_PLAN.contains(required_plan_anchor),
            "Runtime 02 plan must document root-surface guard anchor `{required_plan_anchor}`"
        );
    }
    for required_index_anchor in [
        "Runtime 02 root graphics alias block removal",
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "root_surface.md",
        "crate-visible graphics alias debt 0/0",
    ] {
        assert!(
            RUNTIME_INDEX.contains(required_index_anchor),
            "runtime index must document root-surface guard anchor `{required_index_anchor}`"
        );
    }
}

#[test]
fn graphics_type_alias_debt_symbols_are_only_available_through_graphics_namespace() {
    let retired_root_alias_symbols = [
        "RendererFeatureReferenceListKind",
        "GraphicsError",
        "SceneRenderer",
        "WgpuRenderFramework",
        "ViewportFrame",
        "HybridGiRuntimeProvider",
        "VirtualGeometryRuntimeProvider",
        "SolariRuntimeProvider",
    ];
    for retired_root_alias_symbol in retired_root_alias_symbols {
        assert!(
            !LIB_RS.contains(retired_root_alias_symbol),
            "Runtime 02 M3.2 cutover removed root graphics alias symbol `{retired_root_alias_symbol}`"
        );
    }

    let crate_private_graphics_alias_blocks = LIB_RS
        .lines()
        .filter(|line| line.trim_start().starts_with("pub(crate) use graphics"))
        .count();
    assert_eq!(
        crate_private_graphics_alias_blocks, 0,
        "Runtime 02 M3.2 should leave no crate-private graphics alias blocks in lib.rs"
    );
    assert!(
        !LIB_RS
            .lines()
            .any(|line| line.trim_start().starts_with("pub use graphics")),
        "M3.2 type alias debt must not become a public root graphics export"
    );

    for required_doc_anchor in [
        "M3 Alias Cutover",
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "crate-visible graphics alias debt 0/0",
        "Graphics and render callers must use `crate::graphics::...`",
    ] {
        assert!(
            ROOT_SURFACE_DOC.contains(required_doc_anchor),
            "root surface doc must document M3/M3.2 alias cutover anchor `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "M3 `lib.rs` graphics alias removal",
        "crate-visible graphics alias debt 0/0",
    ] {
        assert!(
            RUNTIME_02_PLAN.contains(required_plan_anchor),
            "Runtime 02 plan must document M3/M3.2 alias cutover anchor `{required_plan_anchor}`"
        );
    }

    for required_index_anchor in [
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "Runtime 02 root graphics alias block removal",
        "crate-visible graphics alias debt 0/0",
    ] {
        assert!(
            RUNTIME_INDEX.contains(required_index_anchor),
            "runtime index must document M3/M3.2 alias cutover anchor `{required_index_anchor}`"
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
        "19 public module declarations",
        "two public `pub use` sites",
        "M3 Alias Cutover",
        "crate-visible graphics alias debt 0/0",
        "root_surface_guard_static_passed",
        "`rhi_wgpu` is now a crate-private backend owner",
        "builtin facade cutover",
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
        "public_module_count = 19",
        "module_decision_count = 19",
        "unclassified_public_module_count = 0",
        "root_surface_migration_debt_count = 0",
        "crate_visible_graphics_reexport_count = 0",
        "`runtime-module-entry`: `animation`, `asset`, `diagnostic_log`, `foundation`, `input`, `navigation`, `platform`, `scene`, `script`, `ui`",
        "`namespace-entry`: `builtin`, `plugin`",
        "M1 gate status `classified-and-clear`",
        "`rhi_wgpu` backend root public exposure is removed",
        "builtin root facade exposure is removed",
        "Runtime 14 classifies `animation` and `navigation`",
        "They are runtime module entries, not unclassified root modules.",
        "crate-visible graphics re-export fan-out removed by Runtime 02 M3",
    ] {
        assert!(
            ROOT_SURFACE_M1_DOC.contains(required_m1_anchor),
            "runtime root-surface M1 gate doc is missing current audit anchor `{required_m1_anchor}`"
        );
    }

    for stale_m1_anchor in [
        "public_module_count = 18",
        "public_module_count = 20",
        "module_decision_count = 18",
        "module_decision_count = 20",
        "root_surface_migration_debt_count = 1",
        "root_surface_migration_debt_count = 2",
        "broad builtin assembly root pub-use facade requires continued M2 review",
        "assembly-facade-review",
        "crate_visible_graphics_reexport_count = 75",
        "crate-visible graphics re-export fan-out: 75 symbols",
        "crate_visible_graphics_reexport_count = 80",
        "crate-visible graphics re-export fan-out: 80 symbols",
        "backend-public-debt",
        "backend module exposed at runtime root: `rhi_wgpu`",
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
        "19 public modules",
        "2 public `pub use` locations",
        "0 crate-visible graphics re-export symbols",
        "`rhi_wgpu` is crate-private backend owner",
        "builtin facade cutover",
        "M1 gate status `classified-and-clear`",
        "runtime root public modules 19/19",
        "crate-visible graphics alias debt 0/0",
        "root_surface guard tests 6/6",
    ] {
        assert!(
            INTERFACE_CONVERGENCE_DOC.contains(required_convergence_anchor),
            "interface convergence doc is missing root-surface audit anchor `{required_convergence_anchor}`"
        );
    }

    for stale_convergence_anchor in [
        "17 public modules",
        "20 public modules",
        "3 public `pub use` locations",
        "direct `rhi_wgpu` backend exposure",
        "runtime root public modules 20/20",
        "75 crate-visible graphics re-export symbols",
        "80 crate-visible graphics re-export symbols",
        "crate-visible graphics alias debt 80/80",
        "root_surface guard tests 5/5",
    ] {
        assert!(
            !INTERFACE_CONVERGENCE_DOC.contains(stale_convergence_anchor),
            "interface convergence doc still contains stale root-surface anchor `{stale_convergence_anchor}`"
        );
    }
}
