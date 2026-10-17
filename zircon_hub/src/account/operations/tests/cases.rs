use super::*;

const OP: &str = "00000000-0000-4000-8000-000000000003";
const NEXT: &str = "00000000-0000-4000-8000-000000000004";

struct Fixture {
    path: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "zircon-hub-journal-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        Self { path }
    }
    fn journal(&self) -> OperationJournal {
        OperationJournal::new(self.path.join("operations.dat"))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn identity() -> OperationIdentity {
    OperationIdentity {
        issuer: "https://identity.example/realm".into(),
        subject: "alice".into(),
        client_id: "desktop".into(),
        service_url: "https://service.example".into(),
    }
}
fn payload() -> OperationPayload {
    OperationPayload::CreateOrganization {
        name: "Project Team".into(),
    }
}

#[test]
fn restart_keeps_unknown_and_rejects_ack_new_id_and_changed_payload() {
    let fixture = Fixture::new();
    let identity = identity();
    assert!(fixture
        .journal()
        .admit(&identity, OP, "7", payload())
        .unwrap());
    let restarted = fixture.journal();
    assert_eq!(
        restarted.list(&identity).unwrap()[0].status,
        OperationStatus::Unknown
    );
    assert!(!restarted.admit(&identity, OP, "9", payload()).unwrap());
    assert!(matches!(
        restarted.acknowledge(&identity, OP),
        Err(AccountError::OutcomeUnknown)
    ));
    assert!(matches!(
        restarted.admit(&identity, NEXT, "9", payload()),
        Err(AccountError::Busy)
    ));
    assert!(matches!(
        restarted.admit(
            &identity,
            OP,
            "9",
            OperationPayload::CreateOrganization {
                name: "Changed".into()
            }
        ),
        Err(AccountError::OperationConflict)
    ));
}

#[test]
fn legacy_unknown_package_remains_blocking_until_target_migration() {
    let request = |target: Option<serde_json::Value>| {
        let mut value = serde_json::json!({
            "operationId": OP,
            "identityDigest": "aa".repeat(32),
            "packageId": NEXT,
            "version": "1.0.0",
            "releaseRevision": "1",
            "artifactDigest": "bb".repeat(32),
            "artifactSize": 1,
            "expectedInventoryRevision": "0"
        });
        if let Some(target) = target {
            value["schemaVersion"] = serde_json::json!(2);
            value["target"] = target;
        }
        serde_json::from_value::<super::super::package::InstallRequest>(value).unwrap()
    };
    let install_payload = |target| OperationPayload::InstallPackage {
        organization: NEXT.into(),
        owner: "cc".repeat(32),
        request: request(target),
    };

    let fixture = Fixture::new();
    let identity = identity();
    let journal = fixture.journal();
    journal
        .admit(&identity, OP, "7", install_payload(None))
        .unwrap();
    assert!(matches!(
        journal.admit(&identity, NEXT, "7", payload()),
        Err(AccountError::Busy)
    ));

    // A targetless v1 result cannot be retried or silently removed.
    assert!(matches!(
        journal.acknowledge(&identity, OP),
        Err(AccountError::OutcomeUnknown)
    ));
    assert_eq!(journal.list(&identity).unwrap().len(), 1);
    assert!(matches!(
        journal.admit(&identity, NEXT, "8", payload()),
        Err(AccountError::Busy)
    ));

    let second = Fixture::new();
    let second_journal = second.journal();
    second_journal
        .admit(
            &identity,
            OP,
            "7",
            install_payload(Some(serde_json::json!({
                "runtime_mode": "editor_host",
                "platform": "windows"
            }))),
        )
        .unwrap();
    assert!(matches!(
        second_journal.acknowledge(&identity, OP),
        Err(AccountError::OutcomeUnknown)
    ));
}

#[test]
fn journal_preserves_every_qualified_identity_and_commit_until_acknowledged() {
    let fixture = Fixture::new();
    let alice = identity();
    fixture.journal().admit(&alice, OP, "7", payload()).unwrap();
    for field in ["subject", "issuer", "client", "service"] {
        let mut other = alice.clone();
        match field {
            "subject" => other.subject.push('2'),
            "issuer" => other.issuer.push('2'),
            "client" => other.client_id.push('2'),
            _ => other.service_url.push('2'),
        }
        assert!(fixture.journal().list(&other).unwrap().is_empty());
        fixture.journal().admit(&other, OP, "7", payload()).unwrap();
        assert_eq!(fixture.journal().list(&alice).unwrap().len(), 1);
    }
    fixture
        .journal()
        .finish(&alice, OP, OperationStatus::Committed)
        .unwrap();
    fixture
        .journal()
        .finish(&alice, OP, OperationStatus::Unknown)
        .unwrap();
    assert_eq!(
        fixture.journal().list(&alice).unwrap()[0].status,
        OperationStatus::Committed
    );
    fixture.journal().acknowledge(&alice, OP).unwrap();
    assert!(fixture.journal().list(&alice).unwrap().is_empty());
}

#[test]
fn invalid_or_full_journal_is_preserved_and_rejects_admission() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    journal
        .access(true, |file| {
            while file.operations.len() < MAX_RECORDS {
                let mut record = file.operations[0].clone();
                record.identity.subject = format!("subject-{}", file.operations.len());
                file.advance_revision(&record.identity)?;
                file.operations.push(record);
            }
            Ok(())
        })
        .unwrap();
    let mut other = identity.clone();
    other.subject = "new-account".into();
    assert!(matches!(
        journal.admit(&other, OP, "7", payload()),
        Err(AccountError::OperationStore)
    ));
    assert_eq!(
        journal.list(&identity).unwrap()[0].status,
        OperationStatus::Unknown
    );
    std::fs::write(&journal.path, b"invalid protected journal").unwrap();
    assert!(journal.list(&identity).is_err());
    assert!(journal.admit(&other, OP, "7", payload()).is_err());
    assert_eq!(
        std::fs::read(&journal.path).unwrap(),
        b"invalid protected journal"
    );
}

#[test]
fn persisted_payload_rejects_unknown_fields_and_dto_never_exposes_payload() {
    assert!(serde_json::from_value::<OperationPayload>(serde_json::json!({"action":"mutate","organization":OP,"expectedPolicyRevision":"1","mutation":{"action":"create-project","name":"P","refresh_token":"secret"}})).is_err());
    let fixture = Fixture::new();
    let identity = identity();
    fixture
        .journal()
        .admit(&identity, OP, "7", payload())
        .unwrap();
    let dto = serde_json::to_string(&fixture.journal().list(&identity).unwrap()).unwrap();
    for secret in [
        "Project Team",
        "mutation",
        "generation",
        "issuer",
        "subject",
        "token",
    ] {
        assert!(!dto.contains(secret));
    }
    #[cfg(windows)]
    assert!(!std::fs::read(&fixture.journal().path)
        .unwrap()
        .windows(12)
        .any(|bytes| bytes == b"Project Team"));
}

#[test]
fn operation_summary_projects_only_the_persisted_v2_package_target() {
    let install_request = |schema_version: Option<u8>, target: Option<serde_json::Value>| {
        let mut request = serde_json::json!({
            "operationId": OP,
            "identityDigest": "aa".repeat(32),
            "packageId": NEXT,
            "version": "1.0.0",
            "releaseRevision": "1",
            "artifactDigest": "bb".repeat(32),
            "artifactSize": 1,
            "expectedInventoryRevision": "0"
        });
        if let Some(schema_version) = schema_version {
            request["schemaVersion"] = serde_json::json!(schema_version);
        }
        if let Some(target) = target {
            request["target"] = target;
        }
        serde_json::from_value::<super::super::package::InstallRequest>(request).unwrap()
    };
    let record = |request| OperationRecord {
        identity: identity(),
        operation_id: OP.into(),
        payload: OperationPayload::InstallPackage {
            organization: NEXT.into(),
            owner: "cc".repeat(32),
            request,
        },
        generation: "7".into(),
        attempts: 1,
        status: OperationStatus::Unknown,
    };

    let v2 = record(install_request(
        Some(2),
        Some(serde_json::json!({"runtime_mode":"editor_host","platform":"windows"})),
    ));
    let v2_summary = serde_json::to_value(v2.summary()).unwrap();
    assert_eq!(v2_summary["targetMode"], "editor_host");
    assert!(v2_summary.get("request").is_none());

    let legacy_v1 = record(install_request(None, None));
    let legacy_summary = serde_json::to_value(legacy_v1.summary()).unwrap();
    assert!(legacy_summary.get("targetMode").is_none());
    assert!(legacy_summary.get("request").is_none());
}

#[test]
fn revisions_are_durable_ordered_and_overflow_never_replaces_the_journal() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    assert_eq!(journal.snapshot(&identity).unwrap().1, "0");
    journal.admit(&identity, OP, "7", payload()).unwrap();
    let first = journal
        .snapshot(&identity)
        .unwrap()
        .1
        .parse::<u64>()
        .unwrap();
    journal.admit(&identity, OP, "9", payload()).unwrap();
    journal
        .finish(&identity, OP, OperationStatus::Failed)
        .unwrap();
    let (records, revision) = fixture.journal().snapshot(&identity).unwrap();
    assert_eq!(records[0].status, OperationStatus::Unknown);
    assert!(revision.parse::<u64>().unwrap() > first);
    journal
        .access(true, |file| {
            file.revisions
                .iter_mut()
                .find(|entry| entry.identity == identity)
                .unwrap()
                .revision = u64::MAX.to_string();
            Ok(())
        })
        .unwrap();
    let before = std::fs::read(&journal.path).unwrap();
    assert!(journal
        .finish(&identity, OP, OperationStatus::Committed)
        .is_err());
    assert_eq!(std::fs::read(&journal.path).unwrap(), before);
}

#[test]
fn revisions_are_isolated_for_every_qualified_identity_even_after_acknowledgement() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let alice = identity();
    journal.admit(&alice, OP, "7", payload()).unwrap();
    let revision = journal.snapshot(&alice).unwrap().1;
    for field in ["subject", "issuer", "client", "service"] {
        let mut other = alice.clone();
        match field {
            "subject" => other.subject.push('2'),
            "issuer" => other.issuer.push('2'),
            "client" => other.client_id.push('2'),
            _ => other.service_url.push('2'),
        }
        assert_eq!(journal.snapshot(&other).unwrap().1, "0");
        journal.admit(&other, OP, "7", payload()).unwrap();
        journal
            .finish(&other, OP, OperationStatus::Committed)
            .unwrap();
        journal.acknowledge(&other, OP).unwrap();
        let (records, other_revision) = fixture.journal().snapshot(&other).unwrap();
        assert!(records.is_empty());
        assert_eq!(other_revision, "3");
        assert_eq!(journal.snapshot(&alice).unwrap().1, revision);
    }
}

#[test]
fn acknowledgement_between_retry_lookup_and_admission_cannot_recreate_an_operation() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    journal
        .finish(&identity, OP, OperationStatus::Failed)
        .unwrap();
    let captured = journal.get(&identity, OP).unwrap();
    journal.acknowledge(&identity, OP).unwrap();
    let before = std::fs::read(&journal.path).unwrap();
    assert!(matches!(
        journal.admit_retry(&identity, OP, "7", captured.payload),
        Err(AccountError::ServiceFailure)
    ));
    assert_eq!(std::fs::read(&journal.path).unwrap(), before);
    assert!(journal.snapshot(&identity).unwrap().0.is_empty());
}

#[test]
fn retry_of_a_known_commit_is_rejected_before_admission() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    journal
        .finish(&identity, OP, OperationStatus::Committed)
        .unwrap();
    assert!(matches!(
        journal.admit_retry(&identity, OP, "7", payload()),
        Err(AccountError::OperationConflict)
    ));
    assert_eq!(journal.snapshot(&identity).unwrap().1, "2");
}

#[test]
fn legacy_global_revisions_migrate_without_losing_pending_operations() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let alice = identity();
    let mut bob = alice.clone();
    bob.subject = "bob".into();
    journal.admit(&alice, OP, "7", payload()).unwrap();
    journal.admit(&bob, OP, "7", payload()).unwrap();
    let mut legacy = journal
        .access(false, |file| {
            serde_json::to_value(file).map_err(|_| AccountError::OperationStore)
        })
        .unwrap();
    legacy.as_object_mut().unwrap().remove("revisions");
    legacy["version"] = serde_json::json!(1);
    legacy["revision"] = serde_json::json!("27");
    #[cfg(windows)]
    let bytes = super::windows::protect(&serde_json::to_vec(&legacy).unwrap(), false).unwrap();
    #[cfg(not(windows))]
    let bytes = serde_json::to_vec(&legacy).unwrap();
    std::fs::write(&journal.path, bytes).unwrap();
    assert_eq!(journal.snapshot(&alice).unwrap().1, "27");
    journal
        .finish(&bob, OP, OperationStatus::Committed)
        .unwrap();
    assert_eq!(fixture.journal().snapshot(&bob).unwrap().1, "28");
    let (records, revision) = fixture.journal().snapshot(&alice).unwrap();
    assert_eq!(revision, "27");
    assert_eq!(records[0].status, OperationStatus::Unknown);
}

#[test]
fn acknowledged_identity_history_does_not_exhaust_the_operation_capacity() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    journal
        .access(true, |file| {
            for index in 0..1024 {
                let mut previous = identity();
                previous.subject = format!("previous-{index}");
                file.revisions.push(IdentityRevision {
                    identity: previous,
                    revision: "3".into(),
                });
            }
            Ok(())
        })
        .unwrap();
    let current = identity();
    journal.admit(&current, OP, "7", payload()).unwrap();
    journal
        .finish(&current, OP, OperationStatus::Committed)
        .unwrap();
    journal.acknowledge(&current, OP).unwrap();
    let reopened = fixture.journal();
    assert_eq!(reopened.snapshot(&current).unwrap().1, "3");
    let mut previous = current;
    previous.subject = "previous-0".into();
    assert_eq!(reopened.snapshot(&previous).unwrap().1, "3");
    assert!(reopened.snapshot(&previous).unwrap().0.is_empty());
}

#[cfg(windows)]
#[test]
fn held_journal_transaction_denies_lock_and_parent_replacement() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    journal
        .access(false, |_| {
            let lock_path = journal.path.with_extension("lock");
            assert!(std::fs::remove_file(&lock_path).is_err());
            assert!(std::fs::rename(&lock_path, fixture.path.join("moved.lock")).is_err());
            assert!(std::fs::rename(&fixture.path, fixture.path.with_extension("moved")).is_err());
            Ok(())
        })
        .unwrap();
}

#[cfg(unix)]
#[test]
fn replacing_the_lock_inode_cannot_admit_a_second_writer_or_publish_the_first() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    let before = std::fs::read(&journal.path).unwrap();
    assert!(matches!(
        journal.access(true, |_| {
            let lock_path = journal.path.with_extension("lock");
            std::fs::remove_file(&lock_path).unwrap();
            std::fs::write(&lock_path, b"").unwrap();
            assert!(journal.snapshot(&identity).is_err());
            Ok(())
        }),
        Err(AccountError::OperationStore)
    ));
    assert_eq!(std::fs::read(&journal.path).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn parent_replacement_cannot_redirect_an_admitted_journal_write() {
    let fixture = Fixture::new();
    let moved = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    let before = std::fs::read(&journal.path).unwrap();
    assert!(matches!(
        journal.access(true, |_| {
            std::fs::rename(&fixture.path, &moved.path).unwrap();
            std::fs::create_dir(&fixture.path).unwrap();
            Ok(())
        }),
        Err(AccountError::OperationStore)
    ));
    assert_eq!(std::fs::read(moved.journal().path).unwrap(), before);
    assert!(!journal.path.exists());
}

#[test]
fn denied_atomic_replacement_retains_the_previous_unknown_record() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    let original = std::fs::read(&journal.path).unwrap();
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let held = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(3)
            .open(&journal.path)
            .unwrap();
        assert!(journal
            .finish(&identity, OP, OperationStatus::Committed)
            .is_err());
        assert_eq!(std::fs::read(&journal.path).unwrap(), original);
        drop(held);
    }
    #[cfg(not(windows))]
    assert_eq!(std::fs::read(&journal.path).unwrap(), original);
    assert_eq!(
        journal.list(&identity).unwrap()[0].status,
        OperationStatus::Unknown
    );
}

fn cloud_payload() -> OperationPayload {
    use crate::account::cloud::manifest::{FileEntry, Manifest};
    use zircon_runtime_interface::project::{
        ProjectGuid, ProjectManifestDigest, ProjectPackageLock, ProjectPackageLockAuthority,
        ProjectPackageLockPlatform, ProjectPackageLockProject, ProjectPackageLockRuntimeMode,
        ProjectPackageLockState, ProjectPackageLockTarget, PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
    };
    let lock = ProjectPackageLock {
        schema_version: PROJECT_PACKAGE_LOCK_SCHEMA_VERSION_V1,
        project: ProjectPackageLockProject {
            project_guid: ProjectGuid::new(),
            manifest_digest: ProjectManifestDigest::from_bytes(b"operations fixture"),
        },
        target: ProjectPackageLockTarget {
            runtime_mode: ProjectPackageLockRuntimeMode::EditorHost,
            platform: ProjectPackageLockPlatform::Windows,
        },
        authority: ProjectPackageLockAuthority {
            index_sha256: "a".repeat(64),
            policy_sha256: "b".repeat(64),
            build_set_id: "c".repeat(64),
            capability_digest: "d".repeat(64),
            provider_revision_digest: "e".repeat(64),
        },
        entries: Vec::new(),
    };
    let package_lock_digest = lock.digest().unwrap();
    OperationPayload::CloudCommit {
        organization: "00000000-0000-4000-8000-000000000001".into(),
        project: "00000000-0000-4000-8000-000000000002".into(),
        base_revision: "0".into(),
        manifest: Manifest {
            schema_version: 1,
            engine: "Zircon".into(),
            package_lock_digest,
            package_lock: Some(ProjectPackageLockState::present(lock).unwrap()),
            ignore_policy: "zircon-project-v1".into(),
            source_revision: None,
            files: vec![FileEntry {
                path: "Content/scene.zui".into(),
                digest: "b".repeat(64),
                bytes: 5,
            }],
        },
    }
}

#[test]
fn cloud_cleanup_snapshot_keeps_project_and_raw_status_in_exact_identity_scope() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let alice = identity();
    let bob = OperationIdentity {
        subject: "bob".into(),
        ..alice.clone()
    };

    journal.admit(&alice, OP, "7", cloud_payload()).unwrap();
    journal
        .finish(&alice, OP, OperationStatus::Conflict)
        .unwrap();
    journal.admit(&alice, NEXT, "7", cloud_payload()).unwrap();
    journal.admit(&bob, OP, "7", cloud_payload()).unwrap();
    journal
        .finish(&bob, OP, OperationStatus::Committed)
        .unwrap();

    let (public_operations, cloud_operations, revision) =
        journal.cloud_cleanup_snapshot(&alice).unwrap();
    assert_eq!(public_operations.len(), 2);
    assert_eq!(public_operations[0].status, OperationStatus::Failed);
    assert_eq!(cloud_operations.len(), 2);
    assert_eq!(cloud_operations[0].operation_id, OP);
    assert_eq!(
        cloud_operations[0].organization_id,
        "00000000-0000-4000-8000-000000000001"
    );
    assert_eq!(
        cloud_operations[0].project_id,
        "00000000-0000-4000-8000-000000000002"
    );
    assert_eq!(cloud_operations[0].status, OperationStatus::Conflict);
    assert_eq!(cloud_operations[1].operation_id, NEXT);
    assert_eq!(cloud_operations[1].status, OperationStatus::Unknown);
    assert!(!revision.is_empty());

    let (bob_operations, bob_cloud_operations, _) = journal.cloud_cleanup_snapshot(&bob).unwrap();
    assert_eq!(bob_operations.len(), 1);
    assert_eq!(bob_cloud_operations.len(), 1);
    assert_eq!(bob_cloud_operations[0].operation_id, OP);
    assert_eq!(bob_cloud_operations[0].status, OperationStatus::Committed);
}

#[test]
fn cloud_commit_upgrades_the_journal_and_retries_only_the_identical_payload() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    assert_eq!(journal.access(false, |file| Ok(file.version)).unwrap(), 2);
    journal
        .finish(&identity, OP, OperationStatus::Committed)
        .unwrap();
    journal.acknowledge(&identity, OP).unwrap();

    let cloud = cloud_payload();
    journal.admit(&identity, NEXT, "7", cloud.clone()).unwrap();
    let restarted = fixture.journal();
    assert_eq!(restarted.access(false, |file| Ok(file.version)).unwrap(), 4);
    assert!(restarted.get(&identity, NEXT).unwrap().payload == cloud);
    assert!(!restarted
        .admit_retry(&identity, NEXT, "9", cloud.clone())
        .unwrap());
    let mut changed = cloud.clone();
    if let OperationPayload::CloudCommit { base_revision, .. } = &mut changed {
        *base_revision = "1".into();
    }
    assert!(matches!(
        restarted.admit_retry(&identity, NEXT, "9", changed),
        Err(AccountError::OperationConflict)
    ));
    restarted
        .finish(&identity, NEXT, OperationStatus::Conflict)
        .unwrap();
    assert!(matches!(
        restarted.finish(&identity, NEXT, OperationStatus::Committed),
        Err(AccountError::OperationConflict)
    ));
    assert_eq!(
        restarted.get(&identity, NEXT).unwrap().status,
        OperationStatus::Conflict
    );
    assert_eq!(
        restarted.snapshot(&identity).unwrap().0[0].status,
        OperationStatus::Failed
    );
    assert!(matches!(
        restarted.admit_retry(&identity, NEXT, "9", cloud),
        Err(AccountError::OperationConflict)
    ));
}

#[test]
fn legacy_journal_versions_read_and_upgrade_to_four_on_cloud_commit() {
    for prior_version in [1, 2, 3] {
        let fixture = Fixture::new();
        let journal = fixture.journal();
        let identity = identity();
        let original = if prior_version == 3 {
            cloud_payload()
        } else {
            payload()
        };
        journal.admit(&identity, OP, "7", original.clone()).unwrap();
        journal
            .finish(&identity, OP, OperationStatus::Committed)
            .unwrap();
        if prior_version == 1 {
            let mut legacy = journal
                .access(false, |file| {
                    serde_json::to_value(file).map_err(|_| AccountError::OperationStore)
                })
                .unwrap();
            legacy.as_object_mut().unwrap().remove("revisions");
            legacy["version"] = serde_json::json!(1);
            legacy["revision"] = serde_json::json!("2");
            #[cfg(windows)]
            let bytes =
                super::windows::protect(&serde_json::to_vec(&legacy).unwrap(), false).unwrap();
            #[cfg(not(windows))]
            let bytes = serde_json::to_vec(&legacy).unwrap();
            std::fs::write(&journal.path, bytes).unwrap();
        } else {
            journal
                .access(true, |file| {
                    file.version = prior_version;
                    Ok(())
                })
                .unwrap();
        }
        let reopened = fixture.journal();
        assert!(reopened.get(&identity, OP).unwrap().payload == original);
        assert_eq!(reopened.snapshot(&identity).unwrap().1, "2");
        reopened
            .admit(&identity, NEXT, "8", cloud_payload())
            .unwrap();
        assert_eq!(reopened.access(false, |file| Ok(file.version)).unwrap(), 4);
        assert_eq!(
            reopened.get(&identity, OP).unwrap().status,
            OperationStatus::Committed
        );
    }
}

#[test]
fn version_three_cloud_retry_upgrades_before_terminal_id_conflict() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    let cloud = cloud_payload();
    journal.admit(&identity, OP, "7", cloud.clone()).unwrap();
    journal
        .access(true, |file| {
            file.version = 3;
            Ok(())
        })
        .unwrap();
    let restarted = fixture.journal();
    assert!(!restarted.admit_retry(&identity, OP, "8", cloud).unwrap());
    assert_eq!(restarted.access(false, |file| Ok(file.version)).unwrap(), 4);
    restarted
        .finish(&identity, OP, OperationStatus::OperationIdConflict)
        .unwrap();
    assert_eq!(
        fixture.journal().get(&identity, OP).unwrap().status,
        OperationStatus::OperationIdConflict
    );
}

#[test]
fn service_valid_cloud_commit_over_the_legacy_cap_is_journaled_and_retried() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    let mut cloud = cloud_payload();
    if let OperationPayload::CloudCommit { manifest, .. } = &mut cloud {
        let original = manifest.files[0].clone();
        manifest.files = (0..1000)
            .map(|index| {
                let mut file = original.clone();
                file.path = format!("Content/file-{index:04}.zr");
                file
            })
            .collect();
    }
    let (_, bytes) = cloud.request(OP).unwrap();
    assert!(bytes.len() > MAX_REQUEST_BYTES);
    assert!(bytes.len() <= MAX_CLOUD_COMMIT_REQUEST_BYTES);
    assert!(journal.admit(&identity, OP, "7", cloud.clone()).unwrap());
    let restarted = fixture.journal();
    assert!(restarted.get(&identity, OP).unwrap().payload == cloud);
    assert!(!restarted.admit_retry(&identity, OP, "9", cloud).unwrap());
    assert_eq!(restarted.access(false, |file| Ok(file.version)).unwrap(), 4);
}

#[test]
fn service_maximum_file_count_fits_the_bounded_journal() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let mut cloud = cloud_payload();
    if let OperationPayload::CloudCommit { manifest, .. } = &mut cloud {
        let original = manifest.files[0].clone();
        let first = "a".repeat(200);
        let second = "b".repeat(200);
        let final_suffix = "c".repeat(97);
        manifest.files = (0..10_000)
            .map(|index| {
                let mut file = original.clone();
                file.path = format!("Content/{first}/{second}/{index:05}{final_suffix}");
                assert_eq!(file.path.len(), 512);
                file
            })
            .collect();
    }
    let (_, request) = cloud.request(OP).unwrap();
    assert!(request.len() > MAX_REQUEST_BYTES);
    assert!(request.len() <= MAX_CLOUD_COMMIT_REQUEST_BYTES);
    journal.admit(&identity(), OP, "7", cloud.clone()).unwrap();
    assert!(std::fs::metadata(&journal.path).unwrap().len() <= MAX_JOURNAL_BYTES as u64);
    assert!(fixture.journal().get(&identity(), OP).unwrap().payload == cloud);
}

#[test]
fn cloud_commit_over_service_file_limit_and_large_legacy_mutation_fail_closed() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let mut cloud = cloud_payload();
    if let OperationPayload::CloudCommit { manifest, .. } = &mut cloud {
        let original = manifest.files[0].clone();
        manifest.files = (0..10_001)
            .map(|index| {
                let mut file = original.clone();
                file.path = format!("Content/file-{index:05}.zr");
                file
            })
            .collect();
    }
    assert!(cloud.request(OP).is_err());
    assert!(journal.admit(&identity(), OP, "7", cloud).is_err());
    assert!(OperationPayload::CreateOrganization {
        name: "x".repeat(MAX_REQUEST_BYTES)
    }
    .request(OP)
    .is_err());
    assert!(!journal.path.exists());
}

#[test]
fn acknowledged_cloud_id_reuse_is_terminal_without_poisoning_new_operations() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", cloud_payload()).unwrap();
    journal
        .finish(&identity, OP, OperationStatus::Committed)
        .unwrap();
    journal.acknowledge(&identity, OP).unwrap();
    let mut reused = cloud_payload();
    if let OperationPayload::CloudCommit { base_revision, .. } = &mut reused {
        *base_revision = "1".into();
    }
    // The service still owns the original receipt; its exact operation-ID
    // rejection must remain terminal across restart, even without an ack.
    assert!(journal.admit(&identity, OP, "8", reused.clone()).unwrap());
    journal
        .finish(&identity, OP, OperationStatus::OperationIdConflict)
        .unwrap();
    let restarted = fixture.journal();
    assert_eq!(
        restarted.get(&identity, OP).unwrap().status,
        OperationStatus::OperationIdConflict
    );
    let before = std::fs::read(&journal.path).unwrap();
    assert!(matches!(
        restarted.admit_retry(&identity, OP, "9", reused.clone()),
        Err(AccountError::OperationConflict)
    ));
    assert!(matches!(
        restarted.admit(&identity, OP, "9", reused),
        Err(AccountError::OperationConflict)
    ));
    assert_eq!(std::fs::read(&journal.path).unwrap(), before);
    restarted
        .finish(&identity, OP, OperationStatus::Unknown)
        .unwrap();
    assert!(matches!(
        restarted.finish(&identity, OP, OperationStatus::Committed),
        Err(AccountError::OperationConflict)
    ));
    assert_eq!(
        restarted.get(&identity, OP).unwrap().status,
        OperationStatus::OperationIdConflict
    );
    assert_eq!(
        restarted.snapshot(&identity).unwrap().0[0].status,
        OperationStatus::Failed
    );
    assert_eq!(
        restarted.snapshot(&identity).unwrap().0[0].error.as_deref(),
        Some("account_operation_id_conflict")
    );
    assert!(restarted
        .admit(&identity, NEXT, "9", cloud_payload())
        .unwrap());
    assert_eq!(
        restarted.get(&identity, NEXT).unwrap().status,
        OperationStatus::Unknown
    );
    restarted.acknowledge(&identity, OP).unwrap();
    assert_eq!(restarted.snapshot(&identity).unwrap().0.len(), 1);
}

#[test]
fn cloud_commit_downgrade_and_conflict_on_another_action_fail_closed() {
    let fixture = Fixture::new();
    let journal = fixture.journal();
    let identity = identity();
    journal.admit(&identity, OP, "7", cloud_payload()).unwrap();
    journal
        .access(true, |file| {
            file.version = 2;
            Ok(())
        })
        .unwrap();
    assert!(matches!(
        journal.snapshot(&identity),
        Err(AccountError::OperationStore)
    ));

    let terminal = Fixture::new();
    let terminal_journal = terminal.journal();
    terminal_journal
        .admit(&identity, OP, "7", cloud_payload())
        .unwrap();
    terminal_journal
        .finish(&identity, OP, OperationStatus::OperationIdConflict)
        .unwrap();
    terminal_journal
        .access(true, |file| {
            file.version = 3;
            Ok(())
        })
        .unwrap();
    assert!(matches!(
        terminal_journal.snapshot(&identity),
        Err(AccountError::OperationStore)
    ));

    let other = Fixture::new();
    let journal = other.journal();
    journal.admit(&identity, OP, "7", payload()).unwrap();
    assert!(matches!(
        journal.finish(&identity, OP, OperationStatus::Conflict),
        Err(AccountError::ServiceFailure)
    ));
    assert!(matches!(
        journal.finish(&identity, OP, OperationStatus::OperationIdConflict),
        Err(AccountError::ServiceFailure)
    ));
    assert_eq!(
        journal.get(&identity, OP).unwrap().status,
        OperationStatus::Unknown
    );
}
