//! A new manager must not turn its own publication or restoration into an authored edit.

use std::fs::{self, File, FileTimes};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{
    artifact_revision, assert_shader_payloads, millis, modified, set_mtime, Fixture, Scan,
};
use crate::asset::project::{AssetMetaDocument, ProjectCatalogInputDelta, ProjectManager};
use crate::asset::{AssetImportError, ImportedAsset};
use crate::core::resource::io::transaction::{DurableTransactionError, TransactionPhase};
use crate::core::resource::io::{replace_staged_file, SourceRevision};
use crate::core::resource::ResourceRecord;

struct Published {
    scan: Scan,
    records: [ResourceRecord; 3],
    payloads: [ImportedAsset; 3],
    manifests: [SourceRevision; 3],
}

impl Published {
    fn first(fixture: &Fixture) -> (ProjectManager, Self) {
        let (manager, scan, records) = fixture.open_ready();
        assert_eq!(scan.meta.source_mtime_unix_ms, super::SEEDED_MS);
        let payloads = fixture.payloads(&manager);
        assert_shader_payloads(&payloads, &scan.members);
        let manifests = records
            .each_ref()
            .map(|record| artifact_revision(&manager, record));
        (
            manager,
            Self {
                scan,
                records,
                payloads,
                manifests,
            },
        )
    }

    fn cold_scan(&self, fixture: &Fixture, mut manager: ProjectManager) -> Scan {
        // Open recovers durable files, but does not populate the live catalog or load payloads.
        let empty = manager.catalog_input_generation();
        assert_eq!(empty.records().count(), 0);
        assert_eq!(manager.registry().values().count(), 0);
        let (scan, records) = fixture.scan_ready(&mut manager, self.scan.meta.uuid);
        assert_eq!(
            scan.catalog.delta_since(&empty),
            ProjectCatalogInputDelta {
                added: vec![scan.row().clone()],
                ..Default::default()
            }
        );
        assert_eq!(records, self.records);
        assert_eq!(fixture.payloads(&manager), self.payloads);
        self.assert_same_manifests(&manager);
        assert_eq!(scan.members, self.scan.members);
        scan.assert_same_meta_except_mtime(&self.scan);
        assert_eq!(
            scan.row().direct_references(),
            self.scan.row().direct_references()
        );
        assert_eq!(
            scan.row().reference_repairs(),
            self.scan.row().reference_repairs()
        );
        scan
    }

    fn assert_same_manifests(&self, manager: &ProjectManager) {
        assert_eq!(
            self.records
                .each_ref()
                .map(|record| artifact_revision(manager, record)),
            self.manifests,
            "cold restore must not rebuild even a byte-identical artifact manifest"
        );
    }

    fn assert_clock(&self, scan: &Scan, expected_ms: u64, observed: SystemTime, case: &str) {
        // This cross-manager delta only compares durable content, not activation or Arc identity.
        let delta = scan.catalog.delta_since(&self.scan.catalog);
        eprintln!(
            "COMPOUND_SOURCE_TIME_REOPEN case={case} first_raw_ms={} observed_raw={observed:?} \
             postscan_raw_ms={} saved_ms={}->{} expected_ms={expected_ms} bytes_equal={} \
             added={} modified={} removed={} renamed={} project_changed={}",
            self.scan.descriptor_ms,
            scan.descriptor_ms,
            self.scan.meta.source_mtime_unix_ms,
            scan.meta.source_mtime_unix_ms,
            self.scan.descriptor_text == scan.descriptor_text,
            delta.added.len(),
            delta.modified.len(),
            delta.removed.len(),
            delta.renamed.len(),
            delta.project_metadata_changed,
        );
        assert_eq!(
            scan.meta.source_mtime_unix_ms, expected_ms,
            "{case}: durable source clock"
        );
        if expected_ms == self.scan.meta.source_mtime_unix_ms {
            assert_eq!(scan.descriptor_text, self.scan.descriptor_text);
            assert!(delta.is_unchanged(), "{case}: {delta:#?}");
        } else {
            scan.assert_only_modified_since(&self.scan);
        }
    }
}

#[test]
fn compound_source_time_cold_open_keeps_generated_clock_stable() {
    let fixture = Fixture::new("compound_clock_reopen");
    let (manager, before) = Published::first(&fixture);
    let observed = modified(&fixture.descriptor);
    let object = descriptor_revision(&fixture);
    drop(manager);

    let reopened = ProjectManager::open(&fixture.root).unwrap();
    assert_eq!(
        fs::read(&fixture.descriptor).unwrap(),
        before.scan.descriptor_text.as_bytes()
    );
    assert_eq!(modified(&fixture.descriptor), observed);
    assert_eq!(descriptor_revision(&fixture), object);
    let scan = before.cold_scan(&fixture, reopened);
    before.assert_clock(
        &scan,
        before.scan.meta.source_mtime_unix_ms,
        observed,
        "natural",
    );
    assert_eq!(
        modified(&fixture.descriptor),
        observed,
        "no second descriptor write"
    );
    assert_eq!(
        descriptor_revision(&fixture),
        object,
        "no second descriptor write"
    );
}

#[test]
fn compound_source_time_cold_open_sees_same_object_author_touch() {
    let fixture = Fixture::new("compound_clock_reopen_touch");
    let (manager, before) = Published::first(&fixture);
    drop(manager);
    // Below the baseline's recent generated timestamp but above the actual members. Do not use
    // max(previous physical time, observed time); a real author can also backdate a source.
    let mut touched_ms = super::SEEDED_MS + 60_000;
    if touched_ms == before.scan.descriptor_ms {
        touched_ms += 60_000;
    }
    let touch = touch_descriptor(&fixture, &before, touched_ms);
    let reopened = ProjectManager::open(&fixture.root).unwrap();
    assert_eq!(modified(&fixture.descriptor), touch);
    let scan = before.cold_scan(&fixture, reopened);
    before.assert_clock(&scan, touched_ms, touch, "same-object-author-touch");
}

#[test]
fn compound_source_time_cold_open_distinguishes_same_bytes_same_stamp_replacement() {
    let fixture = Fixture::new("compound_clock_reopen_replacement");
    let (manager, before) = Published::first(&fixture);
    let observed = modified(&fixture.descriptor);
    let original = descriptor_revision(&fixture);
    drop(manager);

    // A physical author's replacement, not a ProjectManager publication. Keep content and raw
    // mtime exactly equal so that bytes + timestamp alone cannot identify the old generated file.
    let staging = fixture
        .descriptor
        .with_file_name(".compound-clock-author-replacement");
    let mut file = File::create_new(&staging).unwrap();
    file.write_all(before.scan.descriptor_text.as_bytes())
        .unwrap();
    file.set_times(FileTimes::new().set_modified(observed))
        .unwrap();
    file.sync_all().unwrap();
    drop(file);
    let staged = SourceRevision::observe(&staging).unwrap();
    replace_staged_file(&staging, &fixture.descriptor).unwrap();
    let replacement = descriptor_revision(&fixture);
    assert_eq!(replacement, staged);
    assert_ne!(
        replacement, original,
        "different physical file, same content and mtime"
    );
    assert_eq!(modified(&fixture.descriptor), observed);
    assert_eq!(
        fs::read(&fixture.descriptor).unwrap(),
        before.scan.descriptor_text.as_bytes()
    );

    let reopened = ProjectManager::open(&fixture.root).unwrap();
    assert_eq!(descriptor_revision(&fixture), replacement);
    let scan = before.cold_scan(&fixture, reopened);
    // If a design preserves physical time, equal aggregate time is not a synthetic Modified.
    // If it leaves a high generated stamp on disk, the new object is not that generated write.
    let expected = millis(observed).max(super::SEEDED_MS);
    before.assert_clock(&scan, expected, observed, "external-replacement");
}

#[test]
fn compound_source_time_live_rollback_does_not_erase_author_touch() {
    assert_author_clock_after_rollback(false);
}

#[test]
fn compound_source_time_reopen_recovery_does_not_erase_author_touch() {
    assert_author_clock_after_rollback(true);
}

fn assert_author_clock_after_rollback(interrupted: bool) {
    let case = if interrupted {
        "recovery"
    } else {
        "live-rollback"
    };
    let fixture = Fixture::new(case);
    let (mut manager, before) = Published::first(&fixture);
    let registry = manager.paths().registry_root().join("asset-registry.json");
    let registry_bytes = fs::read(&registry).unwrap();
    let active_index = manager.asset_registry_shared();
    let journal = manager.paths().derived_root().join("project-generation");
    assert_eq!(entries(&journal), 0);

    // This real author change forces the otherwise-restored Ready descriptor into the write set;
    // the authored clock has not yet been published. No member or artifact changes are needed.
    let author_ms = before.scan.descriptor_ms.max(super::SEEDED_MS) + 60_000;
    let authored = touch_descriptor(&fixture, &before, author_ms);
    let author_object = descriptor_revision(&fixture);
    let (failure, detail) = if interrupted {
        (
            manager.scan_and_import_with_last_commit_interruption(),
            "injected interruption after committed transition",
        )
    } else {
        (
            manager.scan_and_import_with_commit_failure_before_registry(None),
            "injected transaction commit failure",
        )
    };
    let failure = failure.expect_err("same project-generation transaction must be interrupted");
    let AssetImportError::Io(error) = &failure else {
        panic!("unexpected failure: {failure}")
    };
    let Some(DurableTransactionError::Operation {
        phase,
        path,
        source,
    }) = error
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<DurableTransactionError>())
    else {
        panic!("unexpected transaction failure: {failure}")
    };
    assert_eq!(*phase, TransactionPhase::Commit);
    assert_eq!(path.file_name(), registry.file_name());
    assert_eq!(source.to_string(), detail);
    assert!(Arc::ptr_eq(
        &before.scan.catalog,
        &manager.catalog_input_generation()
    ));
    assert!(Arc::ptr_eq(&active_index, &manager.asset_registry_shared()));
    before.assert_same_manifests(&manager);
    if interrupted {
        // Prove the descriptor has actually been published by the interrupted real transaction.
        assert_eq!(entries(&journal), 1);
        let mut staged = before.scan.meta.clone();
        staged.source_mtime_unix_ms = author_ms;
        assert_eq!(
            AssetMetaDocument::load(&fixture.descriptor).unwrap(),
            staged
        );
    } else {
        assert_eq!(entries(&journal), 0);
        assert_eq!(fs::read(&registry).unwrap(), registry_bytes);
        assert_eq!(
            fs::read(&fixture.descriptor).unwrap(),
            before.scan.descriptor_text.as_bytes()
        );
    }
    drop(manager);

    let reopened = ProjectManager::open(&fixture.root).unwrap();
    assert_eq!(entries(&journal), 0);
    assert_eq!(fs::read(&registry).unwrap(), registry_bytes);
    assert_eq!(
        fs::read(&fixture.descriptor).unwrap(),
        before.scan.descriptor_text.as_bytes()
    );
    before.assert_same_manifests(&reopened);
    let restored = modified(&fixture.descriptor);
    eprintln!(
        "COMPOUND_SOURCE_TIME_ROLLBACK case={case} authored_raw={authored:?} \
         restored_raw={restored:?} object_preserved={} prior_logical_ms={} \
         journal_cleared=true restored_bytes_equal=true",
        descriptor_revision(&fixture) == author_object,
        before.scan.meta.source_mtime_unix_ms,
    );
    let scan = before.cold_scan(&fixture, reopened);
    before.assert_clock(&scan, author_ms, restored, case);
}

fn touch_descriptor(fixture: &Fixture, before: &Published, stamp_ms: u64) -> SystemTime {
    let original = descriptor_revision(fixture);
    let stamp = UNIX_EPOCH + Duration::from_millis(stamp_ms);
    assert_ne!(modified(&fixture.descriptor), stamp);
    set_mtime(&fixture.descriptor, stamp);
    assert_eq!(modified(&fixture.descriptor), stamp);
    assert_eq!(descriptor_revision(fixture), original);
    assert_eq!(
        fs::read(&fixture.descriptor).unwrap(),
        before.scan.descriptor_text.as_bytes()
    );
    stamp
}

fn descriptor_revision(fixture: &Fixture) -> SourceRevision {
    let revision = SourceRevision::observe(&fixture.descriptor).unwrap();
    assert!(matches!(&revision, SourceRevision::Present(_)));
    revision
}

fn entries(directory: &Path) -> usize {
    fs::read_dir(directory).unwrap().map(Result::unwrap).count()
}
