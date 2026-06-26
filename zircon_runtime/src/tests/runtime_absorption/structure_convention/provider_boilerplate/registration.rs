use super::*;

#[test]
fn runtime_15_provider_registration_uses_shared_owner() {
    let shared_registration = read_runtime_src("graphics/runtime_provider/registration.rs");
    let graphics_mod = read_runtime_src("graphics/mod.rs");
    let hybrid_registration =
        read_runtime_src("graphics/hybrid_gi_runtime_provider/provider_registration.rs");
    let virtual_geometry_registration =
        read_runtime_src("graphics/virtual_geometry_runtime_provider/provider_registration.rs");
    let solari_registration =
        read_runtime_src("graphics/solari_runtime_provider/provider_registration.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let provider_doc = read_repo("docs/zircon_runtime/graphics/runtime_provider/registration.md");

    assert_contains_all(
        "shared runtime provider registration owner",
        &shared_registration,
        &[
            "pub(crate) struct RuntimeProviderRegistration<P: ?Sized>",
            "provider_id: String",
            "priority: i32",
            "provider: Arc<P>",
            "debug_name: &'static str",
            "macro_rules! define_runtime_provider_registration",
        ],
    );
    assert_contains_all(
        "graphics runtime provider owner wiring",
        &graphics_mod,
        &["pub(crate) mod runtime_provider;"],
    );

    for (label, source, provider_type) in [
        (
            "hybrid GI provider registration",
            hybrid_registration.as_str(),
            "HybridGiRuntimeProvider",
        ),
        (
            "virtual geometry provider registration",
            virtual_geometry_registration.as_str(),
            "VirtualGeometryRuntimeProvider",
        ),
        (
            "Solari provider registration",
            solari_registration.as_str(),
            "SolariRuntimeProvider",
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "define_runtime_provider_registration!",
                provider_type,
                "RuntimeProviderRegistration",
            ],
        );
        for duplicated_field in [
            "provider_id: String",
            "priority: i32",
            "provider: Arc<dyn",
            "impl fmt::Debug for",
        ] {
            assert!(
                !source.contains(duplicated_field),
                "{label} should delegate registration storage/debug boilerplate to shared owner instead of containing `{duplicated_field}`"
            );
        }
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
                "Runtime 15 F13 provider registration shared owner",
                "runtime_15_provider_registration_shared_owner_coremin_check_passed",
                "runtime_15_provider_registration_uses_shared_owner",
            ],
        );
    }
}
