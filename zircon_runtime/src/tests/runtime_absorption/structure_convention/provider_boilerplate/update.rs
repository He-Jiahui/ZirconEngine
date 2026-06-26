use super::*;

#[test]
fn runtime_15_provider_update_uses_shared_stats_owner() {
    let shared_update = read_runtime_src("graphics/runtime_provider/update.rs");
    let runtime_provider_mod = read_runtime_src("graphics/runtime_provider/mod.rs");
    let hybrid_update = read_runtime_src("graphics/hybrid_gi_runtime_provider/runtime_update.rs");
    let virtual_geometry_update =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/runtime_update.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let provider_doc = read_repo("docs/zircon_runtime/graphics/runtime_provider/update.md");

    assert_contains_all(
        "shared runtime provider update owner",
        &shared_update,
        &[
            "pub(crate) struct RuntimeProviderUpdate<S>",
            "stats: S",
            "macro_rules! define_runtime_provider_update",
            "=> copy",
            "=> ref",
        ],
    );
    assert_contains_all(
        "runtime provider owner exports",
        &runtime_provider_mod,
        &[
            "mod update;",
            "define_runtime_provider_update",
            "RuntimeProviderUpdate",
        ],
    );

    for (label, source, stats_type) in [
        (
            "hybrid GI provider update",
            hybrid_update.as_str(),
            "HybridGiRuntimeStats",
        ),
        (
            "virtual geometry provider update",
            virtual_geometry_update.as_str(),
            "VirtualGeometryRuntimeStats",
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "define_runtime_provider_update!",
                stats_type,
                "RuntimeProviderUpdate",
            ],
        );
        let duplicated_stats_field = format!("stats: {stats_type},");
        assert!(
            !source.contains(&duplicated_stats_field),
            "{label} should delegate update stats storage to RuntimeProviderUpdate instead of declaring `{duplicated_stats_field}`"
        );
        assert!(
            !source.contains("Self { stats }"),
            "{label} should delegate update construction to RuntimeProviderUpdate instead of constructing a local stats field"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime provider doc", provider_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 F13 provider update shared stats owner",
                "runtime_15_provider_update_shared_stats_owner_coremin_check_passed",
                "runtime_15_provider_update_uses_shared_stats_owner",
            ],
        );
    }
}
