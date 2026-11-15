//! Real catalog -> immutable plan -> composition regressions for custom availability facts.
use std::sync::Arc;

use crate::builtin::{RuntimeModuleCompositionCompiler, RuntimePluginId};
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use crate::plugin::{
    PluginMaturity, PluginPackageRole, RuntimePluginAvailabilityCategory as Category,
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
    RuntimeProfileDescriptor,
};

const PACKAGE: &str = "availability_owner";
const RUNTIME: &str = "availability_runtime";

fn registration(
    package: &str,
    runtime: &str,
    maturity: PluginMaturity,
    target: RuntimeTargetMode,
) -> RuntimePluginRegistrationReport {
    let descriptor = RuntimePluginDescriptor::builder(
        package,
        package,
        RuntimePluginId::new(runtime),
        format!("zircon_plugin_{package}_runtime"),
    )
    .with_target_modes([target])
    .with_capability(format!("runtime.{package}"))
    .with_maturity(maturity)
    .with_package_role(PluginPackageRole::Production)
    .build();
    let mut report = RuntimePluginRegistrationReport::from_plugin(&descriptor);
    assert!(report.is_success(), "{:?}", report.diagnostics);
    // Keep raw spelling so canonical duplicate precedence is tested, not normalized away.
    report.project_selection.id = runtime.to_owned();
    report
}

fn manifest(runtime: &str, target: RuntimeTargetMode) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(runtime, true, true).with_target_modes([target])
        ],
    }
}

#[test]
fn required_custom_catalog_provider_reaches_composition_with_distinct_package_identity() {
    let target = RuntimeTargetMode::ServerRuntime;
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration(
            PACKAGE,
            RUNTIME,
            PluginMaturity::Stable,
            target,
        )],
        [],
    );
    let plan = catalog.compiled_project_plan(&manifest(RUNTIME, target), target);
    assert_eq!(plan.linked_provider_package_ids(), &[PACKAGE.to_owned()]);
    let composition = RuntimeModuleCompositionCompiler::new(&plan)
        .compile()
        .expect("a required custom linked provider must survive final availability projection");
    let availability = composition.runtime_plugin_availability();
    let row = availability
        .entry_for(Category::Linked, RuntimePluginId::new(RUNTIME))
        .expect("the actual registered runtime ID must be linked");
    assert_eq!(row.id, PACKAGE, "provider matching must use the package ID");
    assert_eq!(row.maturity, PluginMaturity::Stable);
    assert!(row.required);
    assert!(!availability.has_missing_required());
    assert!(composition
        .module_descriptors()
        .iter()
        .any(|descriptor| descriptor.name == format!("{PACKAGE}.runtime")));
}

#[test]
fn required_unknown_custom_id_still_rejects_composition() {
    let target = RuntimeTargetMode::ServerRuntime;
    let catalog = RuntimePluginCatalog::from_registration_reports([], []);
    let plan = catalog.compiled_project_plan(&manifest(RUNTIME, target), target);
    let rejection = RuntimeModuleCompositionCompiler::new(&plan)
        .compile()
        .expect_err("a requested custom ID is not evidence of a provider");
    assert!(rejection
        .runtime_plugin_availability()
        .has_missing_required());
    assert!(plan.module_proposals().is_empty());
}

#[test]
fn custom_snapshot_does_not_bypass_package_target_policy() {
    let target = RuntimeTargetMode::ClientRuntime;
    let mut report = registration(
        PACKAGE,
        RUNTIME,
        PluginMaturity::Stable,
        RuntimeTargetMode::ServerRuntime,
    );
    // A broad project request cannot expand the package's actual supported targets.
    report.project_selection.target_modes.clear();
    let catalog = RuntimePluginCatalog::from_registration_reports([report], []);
    let plan = catalog.compiled_project_plan(&manifest(RUNTIME, target), target);
    assert_eq!(plan.linked_provider_package_ids(), &[PACKAGE.to_owned()]);
    let rejection = RuntimeModuleCompositionCompiler::new(&plan)
        .compile()
        .expect_err("catalog presence is not target admission");
    assert!(rejection
        .runtime_plugin_availability()
        .contains(Category::BlockedByTarget, RuntimePluginId::new(RUNTIME),));
}

#[test]
fn custom_snapshot_does_not_bypass_profile_maturity_policy() {
    let target = RuntimeTargetMode::ServerRuntime;
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration(
            PACKAGE,
            RUNTIME,
            PluginMaturity::Experimental,
            target,
        )],
        [],
    );
    let profile = RuntimeProfileDescriptor::for_id(RuntimeProfileId::Server)
        .with_minimum_maturity(PluginMaturity::Stable);
    let plan =
        catalog.compiled_project_plan_for_runtime_profile(&manifest(RUNTIME, target), &profile);
    assert!(plan.linked_provider_package_ids().is_empty());
    let rejection = RuntimeModuleCompositionCompiler::new(&plan)
        .compile()
        .expect_err("a blocked catalog provider must not become Ready in composition");
    assert!(rejection
        .runtime_plugin_availability()
        .contains(Category::BlockedByMaturity, RuntimePluginId::new(RUNTIME),));
    assert!(plan.module_proposals().is_empty());
}

#[test]
fn invalid_custom_providers_remain_rejected() {
    let target = RuntimeTargetMode::ServerRuntime;
    let mut disabled = registration(PACKAGE, RUNTIME, PluginMaturity::Stable, target);
    disabled.project_selection.enabled = false;
    let mut fixture_only = registration(PACKAGE, RUNTIME, PluginMaturity::Stable, target);
    fixture_only.package_manifest.package_role = PluginPackageRole::TestFixture;
    let mut diagnosed = registration(PACKAGE, RUNTIME, PluginMaturity::Stable, target);
    diagnosed
        .diagnostics
        .push("invalid provider fixture".to_owned());
    for (case, report) in [
        ("disabled", disabled),
        ("role", fixture_only),
        ("diagnostics", diagnosed),
    ] {
        let catalog = RuntimePluginCatalog::from_registration_reports([report], []);
        let plan = catalog.compiled_project_plan(&manifest(RUNTIME, target), target);
        let result = RuntimeModuleCompositionCompiler::new(&plan).compile();
        assert!(
            result.is_err(),
            "{case} provider must not become a runnable composition"
        );
    }
}

#[test]
fn custom_native_packaging_is_not_authenticated_loader_authority() {
    let target = RuntimeTargetMode::ServerRuntime;
    let mut report = registration(PACKAGE, RUNTIME, PluginMaturity::Stable, target);
    report.project_selection.packaging = ExportPackagingStrategy::NativeDynamic;
    let mut requested = manifest(RUNTIME, target);
    requested.selections[0].packaging = ExportPackagingStrategy::NativeDynamic;
    let catalog = RuntimePluginCatalog::from_registration_reports([report], []);
    let plan = catalog.compiled_project_plan(&requested, target);
    assert_eq!(
        plan.native_dynamic_provider_package_ids(),
        &[PACKAGE.to_owned()]
    );
    let rejection = RuntimeModuleCompositionCompiler::new(&plan)
        .compile()
        .expect_err("a compiled native package has no authenticated loader receipt");
    let availability = rejection.runtime_plugin_availability();
    assert!(availability.has_missing_required());
    assert!(!availability.contains(Category::NativeDynamic, RuntimePluginId::new(RUNTIME)));
    assert!(!availability.contains(Category::Linked, RuntimePluginId::new(RUNTIME)));
}

#[test]
fn catalog_generation_changes_cannot_rewrite_an_existing_plans_facts() {
    let first_target = RuntimeTargetMode::ClientRuntime;
    let mut catalog = RuntimePluginCatalog::from_registration_reports(
        [registration(
            PACKAGE,
            RUNTIME,
            PluginMaturity::Stable,
            first_target,
        )],
        [],
    );
    let requested = manifest(RUNTIME, first_target);
    let old = catalog.compiled_project_plan(&requested, first_target);
    assert!(Arc::ptr_eq(
        &old,
        &catalog.compiled_project_plan(&requested, first_target)
    ));
    let before = RuntimeModuleCompositionCompiler::new(&old)
        .compile()
        .expect("initial custom provider should compose")
        .runtime_plugin_availability()
        .clone();
    let generation = catalog.generation();
    let mut update = catalog.update();
    assert!(update.remove_registration(PACKAGE));
    update.append_registration(registration(
        "availability_next_owner",
        "availability_next_runtime",
        PluginMaturity::Beta,
        RuntimeTargetMode::ServerRuntime,
    ));
    let outcome = update.publish();
    assert!(outcome.is_published(), "{:?}", outcome.diagnostics());
    let new = catalog.compiled_project_plan(
        &manifest(
            "availability_next_runtime",
            RuntimeTargetMode::ServerRuntime,
        ),
        RuntimeTargetMode::ServerRuntime,
    );
    assert_eq!(new.catalog_generation().get(), generation.get() + 1);
    assert_eq!(old.catalog_generation(), generation);
    drop(catalog);

    let retained = RuntimeModuleCompositionCompiler::new(&old)
        .compile()
        .expect("old facts must outlive both a catalog update and the catalog itself");
    assert_eq!(retained.runtime_plugin_availability(), &before);
    let fresh = RuntimeModuleCompositionCompiler::new(&new)
        .compile()
        .expect("the new generation must retain its own package, runtime, maturity and target");
    let row = fresh
        .runtime_plugin_availability()
        .entry_for(
            Category::Linked,
            RuntimePluginId::new("availability_next_runtime"),
        )
        .expect("new generation provider");
    assert_eq!(row.id, "availability_next_owner");
    assert_eq!(row.maturity, PluginMaturity::Beta);
}

#[test]
fn plan_snapshots_preserve_the_descriptor_free_builtin_ui_policy() {
    let target = RuntimeTargetMode::ServerRuntime;
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [registration(
            "availability_ui_owner",
            "ui",
            PluginMaturity::Stable,
            target,
        )],
        [],
    );
    let plan = catalog.compiled_project_plan(&manifest("ui", target), target);
    let profile = RuntimeProfileDescriptor::new(RuntimeProfileId::Server, "UI policy", target);
    let builtins = RuntimePluginDescriptor::builtin_catalog();
    let original = profile.availability_report_for_manifest_with_providers(
        builtins.iter(),
        plan.completed_manifest(),
        plan.linked_provider_package_ids(),
        std::iter::empty::<String>(),
    );
    let actual = profile.availability_report_for_compiled_plan(builtins.iter(), &plan);
    assert_eq!(
        actual, original,
        "a catalog snapshot cannot replace the built-in UI feature rule"
    );
    if !cfg!(feature = "ui") {
        let rejection = RuntimeModuleCompositionCompiler::new(&plan)
            .compile()
            .expect_err("a linked UI report cannot enable an uncompiled built-in domain");
        assert!(rejection
            .runtime_plugin_availability()
            .contains(Category::Stub, RuntimePluginId::Ui));
        assert!(rejection
            .runtime_plugin_availability()
            .has_missing_required());
    }
}

#[test]
fn snapshot_uses_selected_canonical_provider_instead_of_last_duplicate() {
    let target = RuntimeTargetMode::ServerRuntime;
    // Cover exact spelling beating an alias, and first-row priority for an exact tie.
    for duplicate_id in [RUNTIME.to_uppercase(), RUNTIME.to_owned()] {
        let preferred = registration(PACKAGE, RUNTIME, PluginMaturity::Stable, target);
        let duplicate = registration(
            "availability_duplicate",
            &duplicate_id,
            PluginMaturity::Beta,
            target,
        );
        let catalog = RuntimePluginCatalog::from_registration_reports([preferred, duplicate], []);
        let plan = catalog.compiled_project_plan(&manifest(RUNTIME, target), target);
        assert_eq!(plan.linked_provider_package_ids(), &[PACKAGE.to_owned()]);
        let composition = RuntimeModuleCompositionCompiler::new(&plan)
            .compile()
            .expect("unselected duplicate metadata cannot replace the selected package facts");
        let row = composition
            .runtime_plugin_availability()
            .entry_for(Category::Linked, RuntimePluginId::new(RUNTIME))
            .expect("preferred provider remains linked");
        assert_eq!(row.id, PACKAGE);
        assert_eq!(row.maturity, PluginMaturity::Stable);
        assert!(plan
            .module_proposals()
            .iter()
            .all(|proposal| proposal.provider_package_id() == PACKAGE));
    }
}
