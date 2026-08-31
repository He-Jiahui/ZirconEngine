use super::inventory::{
    CORE_MOD_RS, INTERFACE_CONVERGENCE_DOC, ROOT_SURFACE_DOC, ROOT_SURFACE_M1_DOC,
};

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
        "21 public module declarations",
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
        "public_module_count = 21",
        "module_decision_count = 21",
        "unclassified_public_module_count = 0",
        "root_surface_migration_debt_count = 0",
        "crate_visible_graphics_reexport_count = 0",
        "`runtime-module-entry`: `animation`, `asset`, `diagnostic_log`, `foundation`, `input`, `navigation`, `operation`, `platform`, `scene`, `script`, `text`, `ui`",
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
        "21 public modules",
        "2 public `pub use` locations",
        "0 crate-visible graphics re-export symbols",
        "`rhi_wgpu` is crate-private backend owner",
        "builtin facade cutover",
        "M1 gate status `classified-and-clear`",
        "runtime root public modules 21/21",
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
