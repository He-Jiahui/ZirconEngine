use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::super::NativePluginLoader;
use super::super::{
    test_native_plugin_discovery_refresh_service, test_native_plugin_discovery_root,
    NativePluginDiscoveryRefreshBudgetKind, NativePluginDiscoveryRefreshError,
    NativePluginDiscoveryRefreshService, NativePluginDiscoveryRefreshTerminal,
};
use super::support::{
    root, test_budget, wait_for_terminal, AdmissionProbeCollector, AdmissionProbeKind,
};

#[test]
fn every_declared_resource_budget_is_enforced() {
    let budget = test_budget();

    for (candidate_count, diagnostic_count, read_bytes, scratch_bytes, expected_kind) in [
        (
            budget.max_candidates + 1,
            0,
            0,
            0,
            NativePluginDiscoveryRefreshBudgetKind::CandidateCount,
        ),
        (
            0,
            budget.max_diagnostics + 1,
            0,
            0,
            NativePluginDiscoveryRefreshBudgetKind::DiagnosticCount,
        ),
        (
            0,
            0,
            budget.max_read_bytes + 1,
            0,
            NativePluginDiscoveryRefreshBudgetKind::ReadBytes,
        ),
        (
            0,
            0,
            0,
            budget.max_scratch_bytes + 1,
            NativePluginDiscoveryRefreshBudgetKind::ScratchBytes,
        ),
    ] {
        let error = super::super::contract::validate_accounting(
            &budget,
            candidate_count,
            diagnostic_count,
            read_bytes,
            scratch_bytes,
        )
        .expect_err("budget must reject an over-limit accounting report");
        assert!(matches!(
            error,
            NativePluginDiscoveryRefreshError::BudgetExceeded {
                kind,
                actual: _,
                limit: _,
            } if kind == expected_kind
        ));
    }
}

#[test]
fn collector_contract_requires_pre_materialization_runtime_admission() {
    let contract = include_str!("../contract.rs");

    assert!(contract.contains("pub(crate) struct NativePluginDiscoveryRefreshSink"));
    assert!(contract.contains("pub(crate) fn reserve_candidate"));
    assert!(contract.contains("pub(crate) fn reserve_diagnostic"));
    assert!(contract.contains("pub(crate) fn reserve_read_bytes"));
    assert!(contract.contains("pub(crate) fn reserve_scratch_bytes"));
    assert!(contract.contains("pub(crate) fn insert"));
    assert!(contract.contains("pub(crate) fn commit"));
    assert!(!contract.contains("pub(crate) fn execute"));
    assert!(!contract.contains("pub fn reserve_candidate"));
    assert!(!contract.contains("pub struct NativePluginDiscoveryRefreshPayload"));
}

#[test]
fn production_api_keeps_collector_admission_inside_native_discovery_authority() {
    let loader_api = include_str!("../../mod.rs");
    let authority = include_str!("../../discover/authority.rs");
    let contract = include_str!("../contract.rs");
    let service = include_str!("../service.rs");

    assert!(!loader_api.contains("NativePluginDiscoveryCollector"));
    assert!(!loader_api.contains("NativePluginDiscoveryRefreshSink"));
    assert!(!contract.contains("NativePluginDiscoveryCollector"));
    assert!(!service.contains("pub fn new("));
    assert!(service.contains("#[cfg(test)]\n    pub(crate) fn new("));
    assert!(service.contains("NativePluginAuthority"));
    assert!(authority.contains("pub(in crate::plugin::native_plugin_loader) fn collect_refresh"));
}

#[test]
fn collector_admission_rejects_each_resource_before_materialization() {
    for (kind, expected_budget, expected_materialized_units) in [
        (
            AdmissionProbeKind::Candidate,
            NativePluginDiscoveryRefreshBudgetKind::CandidateCount,
            test_budget().max_candidates,
        ),
        (
            AdmissionProbeKind::Diagnostic,
            NativePluginDiscoveryRefreshBudgetKind::DiagnosticCount,
            test_budget().max_diagnostics,
        ),
        (
            AdmissionProbeKind::ReadBytes,
            NativePluginDiscoveryRefreshBudgetKind::ReadBytes,
            0,
        ),
        (
            AdmissionProbeKind::ScratchBytes,
            NativePluginDiscoveryRefreshBudgetKind::ScratchBytes,
            0,
        ),
    ] {
        let collector = Arc::new(AdmissionProbeCollector::new(kind));
        let service = NativePluginDiscoveryRefreshService::new(collector.clone(), test_budget());
        let ticket = service.submit(root("pre-materialization-admission"));
        wait_for_terminal(&ticket);

        let error = match ticket.terminal() {
            Some(NativePluginDiscoveryRefreshTerminal::Failed(error)) => error,
            terminal => panic!("expected budget failure, got {terminal:?}"),
        };
        assert!(matches!(
            error.as_ref(),
            NativePluginDiscoveryRefreshError::BudgetExceeded { kind, .. }
                if *kind == expected_budget
        ));
        assert_eq!(
            collector.materialized_units(),
            expected_materialized_units,
            "{kind:?} must reject before creating its first over-budget unit"
        );
    }
}

#[test]
fn authority_rejects_an_over_budget_manifest_read_before_snapshot_publication() {
    let root_path = temporary_authority_root("read-admission");
    let package_root = root_path.join("metered");
    fs::create_dir_all(&package_root).expect("create native plugin package");
    let manifest = r#"
id = "metered"
version = "0.1.0"
display_name = "metered"

[[modules]]
name = "metered.runtime"
kind = "runtime"
crate_name = "zircon_plugin_metered_runtime"
"#;
    fs::write(package_root.join("plugin.toml"), manifest).expect("write native plugin manifest");

    let mut budget = test_budget();
    budget.max_read_bytes = manifest.len() as u64 - 1;
    let service = test_native_plugin_discovery_refresh_service(budget);
    let root = test_native_plugin_discovery_root(root_path.clone());
    let ticket = service.submit(root.clone());
    wait_for_terminal(&ticket);

    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Failed(error))
            if matches!(
                error.as_ref(),
                NativePluginDiscoveryRefreshError::BudgetExceeded {
                    kind: NativePluginDiscoveryRefreshBudgetKind::ReadBytes,
                    ..
                }
            )
    ));
    assert!(service.snapshot(&root).is_none());
    let _ = fs::remove_dir_all(root_path);
}

#[test]
fn authority_accepts_a_manifest_exactly_at_the_read_limit() {
    let root_path = temporary_authority_root("exact-read-boundary");
    let package_root = root_path.join("metered");
    fs::create_dir_all(&package_root).expect("create native plugin package");
    let manifest = metered_manifest();
    fs::write(package_root.join("plugin.toml"), manifest).expect("write native plugin manifest");

    let mut budget = test_budget();
    budget.max_read_bytes = manifest.len() as u64;
    let service = test_native_plugin_discovery_refresh_service(budget);
    let root = test_native_plugin_discovery_root(root_path.clone());
    let ticket = service.submit(root.clone());
    wait_for_terminal(&ticket);

    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Published(_))
    ));
    assert_eq!(
        service
            .snapshot(&root)
            .expect("exact-limit snapshot")
            .read_bytes(),
        manifest.len() as u64
    );
    let _ = fs::remove_dir_all(root_path);
}

#[test]
fn authority_rejects_when_manifest_buffer_would_exceed_scratch_budget() {
    let root_path = temporary_authority_root("scratch-read-separation");
    let package_root = root_path.join("metered");
    fs::create_dir_all(&package_root).expect("create native plugin package");
    let manifest = metered_manifest();
    fs::write(package_root.join("plugin.toml"), manifest).expect("write native plugin manifest");

    let mut budget = test_budget();
    budget.max_read_bytes = manifest.len() as u64 + 1;
    budget.max_scratch_bytes = manifest.len() as u64 - 1;
    let service = test_native_plugin_discovery_refresh_service(budget);
    let root = test_native_plugin_discovery_root(root_path.clone());
    let ticket = service.submit(root.clone());
    wait_for_terminal(&ticket);

    assert!(matches!(
        ticket.terminal(),
        Some(NativePluginDiscoveryRefreshTerminal::Failed(error))
            if matches!(
                error.as_ref(),
                NativePluginDiscoveryRefreshError::BudgetExceeded {
                    kind: NativePluginDiscoveryRefreshBudgetKind::ScratchBytes,
                    ..
                }
            )
    ));
    assert!(service.snapshot(&root).is_none());
    let _ = fs::remove_dir_all(root_path);
}

#[test]
fn service_uses_the_runtime_io_lane_and_never_calls_the_synchronous_loader() {
    let source = include_str!("../service.rs");

    assert!(source.contains("TaskPools::process_default().io()"));
    assert!(source.contains("handle.on_terminal"));
    assert!(!source.contains("NativePluginLoader"));
    assert!(!source.contains("load_discovered"));
}

#[test]
fn manifest_reads_use_one_handle_and_reject_mutated_lengths() {
    let source = include_str!("../../candidate_from_manifest.rs");

    assert!(source.contains("let mut manifest = fs::File::open"));
    assert!(source.matches("manifest.metadata()").count() >= 2);
    assert!(source.contains("ManifestChangedDuringRead"));
    assert!(source.contains("ensure_manifest_read_is_stable("));
    assert!(source.contains("fn ensure_manifest_read_is_stable"));
    assert!(source.contains("try_reserve_exact(manifest_len)"));
}

fn temporary_authority_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-native-refresh-{label}-{}-{stamp}",
        std::process::id()
    ))
}

fn metered_manifest() -> &'static str {
    r#"
id = "metered"
version = "0.1.0"
display_name = "metered"

[[modules]]
name = "metered.runtime"
kind = "runtime"
crate_name = "zircon_plugin_metered_runtime"
"#
}
