use super::super::support::assert_contains_all_exact;
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
    let current_anchor_owner = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-17-registration-filter-plan-anchor-current-owner.md",
    );
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

    assert_contains_all_exact(
        "Runtime 15 registration-filter current child owner",
        &current_anchor_owner,
        &[
            "Runtime 15 F13 provider registration shared owner",
            "runtime_15_provider_registration_shared_owner_coremin_check_passed",
            "graphics/runtime_provider/registration.rs",
            "docs/zircon_runtime/graphics/runtime_provider/registration.md",
            "runtime_15_provider_registration_uses_shared_owner",
            "2026-06-22",
        ],
    );
    for (label, source) in [
        ("module convention doc", module_doc.as_str()),
        ("runtime provider doc", provider_doc.as_str()),
    ] {
        assert_contains_all_exact(
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
