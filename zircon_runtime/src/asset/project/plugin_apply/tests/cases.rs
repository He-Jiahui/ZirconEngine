#![cfg(windows)]
use super::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Fixture {
    root: PathBuf,
    paths: ProjectPaths,
}
impl Fixture {
    fn new(label: &str) -> Self {
        use std::os::windows::fs::MetadataExt;
        let base = Path::new("E:/cargo-targets/zircon-local");
        for ancestor in base.ancestors() {
            assert_eq!(
                fs::symlink_metadata(ancestor).unwrap().file_attributes() & 0x400,
                0,
                "fixture root must not traverse a reparse point"
            );
        }
        let resolved = ProjectPaths::resolve_existing(base).unwrap();
        let physical = resolved.operation_path();
        let display = physical.to_str().unwrap().replace('\\', "/");
        let display = display.strip_prefix("//?/").unwrap_or(&display);
        assert!(
            display.eq_ignore_ascii_case("E:/cargo-targets/zircon-local"),
            "unexpected physical fixture root"
        );
        let root = physical.join(format!(
            "e06-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let paths = ProjectPaths::from_root(&root).unwrap();
        let mut manifest = ProjectManifest::new(
            "Enablement test",
            crate::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        );
        manifest.plugins.selections.push(
            serde_json::from_value(serde_json::json!({ "id": "fixture.editor", "enabled": false }))
                .unwrap(),
        );
        manifest.save(paths.manifest_path()).unwrap();
        Self { root, paths }
    }
    fn plan(&self) -> (PluginApplyDocument, ProjectManifest, PendingPluginApply) {
        let document = PluginApplyDocument::read(&self.paths).unwrap();
        let mut candidate = document.manifest.clone();
        candidate.plugins.selections[0].enabled = true;
        let receipt = PendingPluginApply {
            version: 1,
            operation_id: "enable-1".into(),
            plugin_id: "fixture.editor".into(),
            enabled: true,
            previous_enabled: false,
            project_guid: candidate.project_guid.to_string(),
            manifest_digest: String::new(),
            session_instance: "fixture-session".into(),
            session_generation: 1,
            catalog_generation: 1,
            acknowledged: false,
            resolved_before_activation: false,
        };
        (document, candidate, receipt)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
#[test]
fn stale_manifest_revision_rejects_manifest_and_pending_publication() {
    let fixture = Fixture::new("stale");
    let (document, candidate, receipt) = fixture.plan();
    let mut newer = document.manifest.clone();
    newer.plugins.selections[0].enabled = true;
    newer.save(fixture.paths.manifest_path()).unwrap();
    let before = fs::read(fixture.paths.manifest_path()).unwrap();
    let result = commit_plugin_apply(&fixture.paths, &document, &candidate, receipt, None);
    assert!(result.is_err());
    assert_eq!(fs::read(fixture.paths.manifest_path()).unwrap(), before);
    assert!(pending_plugin_apply(&fixture.paths).unwrap().is_none());
}
#[test]
fn interrupted_manifest_replace_blocks_parse_until_owned_recovery_restores_pair() {
    let fixture = Fixture::new("interrupted");
    let original = fs::read(fixture.paths.manifest_path()).unwrap();
    let (document, candidate, receipt) = fixture.plan();
    assert!(commit_with_fault(
        &fixture.paths,
        &document,
        &candidate,
        receipt,
        None,
        TransactionFault::CrashAfterTargetReplace(0)
    )
    .is_err());
    assert_eq!(
        PluginApplyDocument::read(&fixture.paths)
            .err()
            .unwrap()
            .kind,
        PluginApplyErrorKind::RecoveryRequired
    );
    drop(document);
    recover_plugin_apply_files(&fixture.paths).unwrap();
    assert_eq!(fs::read(fixture.paths.manifest_path()).unwrap(), original);
    assert!(pending_plugin_apply(&fixture.paths).unwrap().is_none());
}
#[test]
fn cleanup_and_commit_recovery_dispositions_have_different_read_gates() {
    for (label, fault, expected) in [
        (
            "cleanup",
            TransactionFault::FailCommittedCleanup,
            PluginApplyDurability::CleanupDeferred,
        ),
        (
            "commit-point",
            TransactionFault::FailCommitPointSync,
            PluginApplyDurability::RecoveryDeferred,
        ),
    ] {
        let fixture = Fixture::new(label);
        let (document, candidate, receipt) = fixture.plan();
        let committed =
            commit_with_fault(&fixture.paths, &document, &candidate, receipt, None, fault).unwrap();
        assert_eq!(committed.durability, expected);
        assert_eq!(
            ensure_plugin_apply_ready(&fixture.paths).unwrap_err().kind,
            PluginApplyErrorKind::RecoveryRequired
        );
        drop(committed);
        drop(document);
        recover_plugin_apply_files(&fixture.paths).unwrap();
        ensure_plugin_apply_ready(&fixture.paths).unwrap();
    }
}
#[test]
fn stale_acknowledgement_keeps_pending_and_preflight_blocked() {
    let fixture = Fixture::new("ack");
    let (document, candidate, receipt) = fixture.plan();
    let committed =
        commit_plugin_apply(&fixture.paths, &document, &candidate, receipt, None).unwrap();
    let revision = committed.committed_revision.as_ref().unwrap();
    assert_eq!(
        acknowledge_plugin_apply(&fixture.paths, "wrong-operation", revision)
            .unwrap_err()
            .kind,
        PluginApplyErrorKind::StaleRevision
    );
    assert_eq!(
        ensure_plugin_apply_preflight_ready(&fixture.paths)
            .unwrap_err()
            .kind,
        PluginApplyErrorKind::PendingOperation
    );
    assert!(pending_plugin_apply(&fixture.paths).unwrap().is_some());
}
#[test]
fn explicit_preopen_revert_is_a_new_durable_resolution_at_expected_revision() {
    let fixture = Fixture::new("revert");
    let (document, candidate, receipt) = fixture.plan();
    let committed =
        commit_plugin_apply(&fixture.paths, &document, &candidate, receipt, None).unwrap();
    let revision = committed.committed_revision.as_ref().unwrap();
    assert!(revert_plugin_apply_before_activation(
        &fixture.paths,
        "wrong-operation",
        revision,
        "revert-1".into()
    )
    .is_err());
    let reverted = revert_plugin_apply_before_activation(
        &fixture.paths,
        "enable-1",
        revision,
        "revert-1".into(),
    )
    .unwrap();
    assert!(reverted.receipt.resolved_before_activation);
    assert!(!reverted.receipt.acknowledged);
    assert_eq!(reverted.receipt.operation_id, "revert-1");
    assert!(
        !PluginApplyDocument::read(&fixture.paths)
            .unwrap()
            .manifest
            .plugins
            .selections[0]
            .enabled
    );
    ensure_plugin_apply_preflight_ready(&fixture.paths).unwrap();
    assert!(pending_plugin_apply(&fixture.paths).unwrap().is_none());
}

#[test]
fn recovery_refuses_a_foreign_target_before_mutating_its_bytes() {
    let fixture = Fixture::new("policy");
    let foreign = fixture.root.join("foreign.txt");
    fs::write(&foreign, b"foreign-original").unwrap();
    let write = PreparedFileWrite::new(foreign.clone(), b"forbidden-replacement".to_vec())
        .expecting_source(SourceRevision::observe(&foreign).unwrap());
    let mut report = DurableCommitReport::default();
    assert!(commit_prepared_files_with_revisions(
        &journal(&fixture.paths),
        TAG,
        vec![write],
        TransactionFault::CrashAfterStaging(0),
        &mut report
    )
    .is_err());
    assert!(recover_plugin_apply_files(&fixture.paths).is_err());
    assert_eq!(fs::read(&foreign).unwrap(), b"foreign-original");
    assert!(ensure_plugin_apply_ready(&fixture.paths).is_err());
}

#[test]
fn bounded_manifest_snapshot_rejects_oversize_before_parse() {
    let fixture = Fixture::new("oversize");
    let bytes = vec![b'x'; zircon_runtime_interface::project::MAX_PROJECT_MANIFEST_BYTES + 1];
    fs::write(fixture.paths.manifest_path(), &bytes).unwrap();
    let error = PluginApplyDocument::read(&fixture.paths).err().unwrap();
    assert_eq!(error.kind, PluginApplyErrorKind::Io);
    assert_eq!(fs::read(fixture.paths.manifest_path()).unwrap(), bytes);
    assert!(pending_plugin_apply(&fixture.paths).unwrap().is_none());
}
