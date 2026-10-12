use super::*;
use crate::plugin::PluginPackageManifest;

#[test]
fn prepared_candidate_advances_once_without_mutating_the_base_snapshot() {
    let base = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(
        RuntimePluginCatalog::from_registration_reports([package_report("base")], []),
    ));
    let mut candidate = base.stage_update();
    candidate.append_registration(package_report("next"));

    let prepared = candidate.prepare().expect("candidate should prepare");

    assert_eq!(base.generation().get(), 1);
    assert_eq!(prepared.snapshot().generation().get(), 2);
    assert_eq!(base.catalog().registrations().len(), 1);
    assert_eq!(prepared.snapshot().catalog().registrations().len(), 2);
    assert_eq!(prepared.metrics().candidate_registration_rows_indexed, 1);
    assert_eq!(
        prepared
            .metrics()
            .candidate_feature_registration_rows_indexed,
        0
    );
    assert_eq!(prepared.metrics().candidate_projection_builds, 1);
    assert_eq!(prepared.metrics().candidate_diagnostic_builds, 1);
    assert_eq!(prepared.metrics().published_generations, 0);
}

#[test]
fn empty_candidate_cannot_consume_a_generation() {
    let base = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(
        RuntimePluginCatalog::from_registration_reports([package_report("base")], []),
    ));

    let error = base
        .stage_update()
        .prepare()
        .expect_err("empty candidate should not prepare");

    assert!(error
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.contains("does not contain changes")));
    assert_eq!(base.generation().get(), 1);
}

#[test]
fn invalid_candidate_keeps_the_base_snapshot_available() {
    let base = Arc::new(RuntimePluginCatalogSnapshot::from_catalog(
        RuntimePluginCatalog::from_registration_reports([package_report("base")], []),
    ));
    let mut candidate = base.stage_update();
    candidate.append_registration(package_report("base"));

    let error = candidate
        .prepare()
        .expect_err("duplicate package should not prepare");

    assert!(!error.diagnostics().is_empty());
    assert_eq!(base.catalog().registrations().len(), 1);
    assert_eq!(base.generation().get(), 1);
}

fn package_report(package_id: &str) -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_native_package_manifest(PluginPackageManifest::new(
        package_id, package_id,
    ))
}
