use super::support::*;
use crate::service::{
    cloud::*,
    error::ServiceError,
    identity::Principal,
    storage::{receipt, Database},
};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Arc;

fn maintain(
    connection: &mut Connection,
    owner: &Principal,
    store: &BlobStore,
    org: &str,
    project: &str,
) -> Result<MaintenanceReport, ServiceError> {
    crate::service::cloud::maintain(
        connection,
        owner,
        store,
        org,
        project,
        MaintenanceRequest {
            operation_id: uuid::Uuid::new_v4().to_string(),
        },
    )
}

fn policy_request(revision: &str, keep: Option<u64>, trash: u64, hold: bool) -> RetentionRequest {
    RetentionRequest {
        operation_id: uuid::Uuid::new_v4().to_string(),
        expected_revision: revision.into(),
        policy: RetentionPolicy {
            keep_latest: keep,
            trash_seconds: trash,
            legal_hold: hold,
        },
    }
}

fn set_policy(
    connection: &mut Connection,
    owner: &Principal,
    org: &str,
    project: &str,
    keep: Option<u64>,
    trash: u64,
    hold: bool,
) -> Result<(), ServiceError> {
    let revision = retention(connection, owner, org, project)?.revision;
    assert!(matches!(
        update_retention(
            connection,
            owner,
            org,
            project,
            policy_request(&revision, keep, trash, hold)
        )?,
        RetentionOutcome::RetentionUpdated { .. }
    ));
    Ok(())
}

fn append(
    connection: &mut Connection,
    owner: &Principal,
    store: &BlobStore,
    org: &str,
    project: &str,
    base: u64,
    bytes: &[u8],
) -> Result<CommitOutcome, ServiceError> {
    upload(
        connection,
        owner,
        store,
        org,
        project,
        &digest(bytes),
        bytes,
    )?;
    commit(
        connection,
        owner,
        store,
        org,
        project,
        request(&base.to_string(), bytes),
    )
}

#[tokio::test]
async fn policy_has_current_admin_authority_revision_and_terminal_operation_receipts() {
    let files = Files::new();
    let (database, _) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let member = principal("member");
            let (org, project) = seed(connection, &owner);
            connection.execute(
                "INSERT INTO memberships VALUES (?1,?2,?3,'member',1)",
                params![org, member.issuer, member.subject],
            )?;
            let initial = retention(connection, &member, &org, &project)?;
            assert_eq!(initial.revision, "0");
            assert_eq!(initial.policy, RetentionPolicy::default());
            let request = policy_request("0", Some(2), 0, false);
            assert!(matches!(
                update_retention(connection, &member, &org, &project, request.clone()),
                Err(ServiceError::Forbidden)
            ));
            let first = serde_json::to_value(update_retention(
                connection,
                &owner,
                &org,
                &project,
                request.clone(),
            )?)
            .unwrap();
            assert_eq!(first["retention"]["revision"], "1");
            assert_eq!(
                serde_json::to_value(update_retention(
                    connection,
                    &owner,
                    &org,
                    &project,
                    request.clone()
                )?)
                .unwrap(),
                first
            );
            let mut changed = request.clone();
            changed.policy.legal_hold = true;
            assert!(matches!(
                update_retention(connection, &owner, &org, &project, changed),
                Err(ServiceError::OperationConflict)
            ));
            let stale = policy_request("0", Some(1), 0, false);
            let conflict = serde_json::to_value(update_retention(
                connection,
                &owner,
                &org,
                &project,
                stale.clone(),
            )?)
            .unwrap();
            assert_eq!(conflict["status"], "retentionConflict");
            assert_eq!(conflict["currentRevision"], "1");
            set_policy(connection, &owner, &org, &project, Some(3), 0, false)?;
            assert_eq!(
                serde_json::to_value(update_retention(connection, &owner, &org, &project, stale)?)
                    .unwrap(),
                conflict
            );
            for invalid in [
                policy_request("2", Some(0), 0, false),
                policy_request("2", Some(129), 0, false),
                policy_request("2", Some(1), 2_592_001, false),
                policy_request("02", None, 0, false),
            ] {
                assert!(matches!(
                    update_retention(connection, &owner, &org, &project, invalid),
                    Err(ServiceError::InvalidRequest)
                ));
            }
            let receipt::OperationStatus::Committed { result } =
                receipt::lookup(connection, &owner, &request.operation_id)?
            else {
                panic!("missing retention receipt")
            };
            assert_eq!(result, first);
            connection.execute("UPDATE memberships SET active=0 WHERE subject='owner'", [])?;
            assert!(matches!(
                update_retention(connection, &owner, &org, &project, request),
                Err(ServiceError::Forbidden)
            ));
            assert!(matches!(
                authorize_receipt(connection, &owner, &result),
                Err(ServiceError::Forbidden)
            ));
            assert_eq!(
                retention(connection, &member, &org, &project)?.revision,
                "2"
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn rolling_policy_sustains_more_than_128_commits_with_bounded_cas() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            set_policy(connection, &owner, &org, &project, Some(2), 0, false)?;
            for base in 0..140 {
                let bytes = format!("revision-{base}");
                upload(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(bytes.as_bytes()),
                    bytes.as_bytes(),
                )?;
                connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
                assert!(matches!(
                    commit(
                        connection,
                        &owner,
                        &store,
                        &org,
                        &project,
                        request(&base.to_string(), bytes.as_bytes())
                    )?,
                    CommitOutcome::Committed { .. }
                ));
                assert!(usage(connection, &owner, &org, &project)?.revisions <= 2);
                assert!(store.physical.lock().unwrap().unwrap().1 <= 3);
            }
            let report = maintain(connection, &owner, &store, &org, &project)?;
            assert_eq!(report.pending_objects, 0);
            let usage = usage(connection, &owner, &org, &project)?;
            assert_eq!((usage.revisions, usage.objects), (2, 2));
            assert_eq!(usage.retention, "keep-latest-with-trash");
            assert_eq!(store.physical.lock().unwrap().unwrap().1, 2);
            assert_eq!(
                head(connection, &owner, &org, &project)?.unwrap().revision,
                "140"
            );
            assert_eq!(
                read_blob(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(b"revision-139")
                )?,
                b"revision-139"
            );
            assert!(connection
                .query_row("PRAGMA foreign_key_check", [], |_| Ok(()))
                .optional()?
                .is_none());
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn grace_hold_and_new_head_protect_snapshot_references() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            set_policy(connection, &owner, &org, &project, Some(1), 3600, false)?;
            append(connection, &owner, &store, &org, &project, 0, b"first")?;
            append(connection, &owner, &store, &org, &project, 1, b"second")?;
            assert_eq!(
                usage(connection, &owner, &org, &project)?.trash_revisions,
                1
            );
            assert_eq!(
                read_blob(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(b"first")
                )?,
                b"first"
            );
            set_policy(connection, &owner, &org, &project, Some(1), 3600, true)?;
            append(connection, &owner, &store, &org, &project, 2, b"third")?;
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            assert_eq!(
                maintain(connection, &owner, &store, &org, &project)?.pruned_revisions,
                0
            );
            assert_eq!(usage(connection, &owner, &org, &project)?.revisions, 3);
            assert_eq!(
                usage(connection, &owner, &org, &project)?.trash_revisions,
                0
            );
            set_policy(connection, &owner, &org, &project, Some(1), 3600, false)?;
            assert_eq!(
                usage(connection, &owner, &org, &project)?.trash_revisions,
                2
            );
            assert_eq!(
                maintain(connection, &owner, &store, &org, &project)?.pruned_revisions,
                0
            );
            connection.execute("UPDATE cloud_snapshot_trash SET purge_after=0", [])?;
            let report = maintain(connection, &owner, &store, &org, &project)?;
            assert_eq!((report.pruned_revisions, report.collected_objects), (2, 2));
            assert_eq!(
                head(connection, &owner, &org, &project)?.unwrap().revision,
                "3"
            );
            assert_eq!(
                read_blob(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(b"third")
                )?,
                b"third"
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn shared_tenant_blob_and_upload_lease_survive_unreferenced_retirement() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (a, project) = seed(connection, &owner);
            let (b, _) = seed(connection, &owner);
            set_policy(connection, &owner, &a, &project, Some(1), 0, false)?;
            append(connection, &owner, &store, &a, &project, 0, b"shared")?;
            append(connection, &owner, &store, &b, &project, 0, b"shared")?;
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            append(connection, &owner, &store, &a, &project, 1, b"replacement")?;
            maintain(connection, &owner, &store, &a, &project)?;
            assert!(matches!(
                read_blob(connection, &owner, &store, &a, &project, &digest(b"shared")),
                Err(ServiceError::Forbidden)
            ));
            assert_eq!(
                read_blob(connection, &owner, &store, &b, &project, &digest(b"shared"))?,
                b"shared"
            );
            connection.execute(
                "INSERT INTO cloud_gc_pending VALUES (?1,6,?2,?3)",
                params![digest(b"shared"), a, project],
            )?;
            maintain(connection, &owner, &store, &a, &project)?;
            assert_eq!(store.read(&digest(b"shared"))?, b"shared");
            upload(
                connection,
                &owner,
                &store,
                &a,
                &project,
                &digest(b"pending"),
                b"pending",
            )?;
            assert_eq!(
                maintain(connection, &owner, &store, &a, &project)?.collected_objects,
                0
            );
            assert_eq!(store.read(&digest(b"pending"))?, b"pending");
            connection.execute(
                "UPDATE cloud_upload_leases SET expires_at=0 WHERE digest=?1",
                [digest(b"pending")],
            )?;
            assert_eq!(
                maintain(connection, &owner, &store, &a, &project)?.collected_objects,
                1
            );
            assert!(!store.root.join(digest(b"pending")).exists());
            assert_eq!(store.read(&digest(b"shared"))?, b"shared");
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn failed_policy_audit_rolls_back_pruning_and_tombstones() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database.execute(move |connection| {
        let owner = principal("owner");
        let (org, project) = seed(connection, &owner);
        append(connection, &owner, &store, &org, &project, 0, b"first")?;
        append(connection, &owner, &store, &org, &project, 1, b"head")?;
        connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
        connection.execute_batch("CREATE TEMP TRIGGER fail_retention BEFORE INSERT ON audit_events WHEN NEW.action='cloud.retention' BEGIN SELECT RAISE(ABORT,'injected'); END;")?;
        let request = policy_request("0", Some(1), 0, false);
        assert!(matches!(update_retention(connection, &owner, &org, &project, request.clone()), Err(ServiceError::Storage)));
        assert_eq!(retention(connection, &owner, &org, &project)?.revision, "0");
        let before = usage(connection, &owner, &org, &project)?;
        assert_eq!((before.revisions, before.objects, before.pending_gc_objects), (2, 2, 0));
        assert!(matches!(receipt::lookup(connection, &owner, &request.operation_id)?, receipt::OperationStatus::Unknown));
        assert_eq!(store.physical.lock().unwrap().unwrap().1, 2);
        connection.execute_batch("DROP TRIGGER fail_retention;")?;
        update_retention(connection, &owner, &org, &project, request)?;
        assert_eq!(before.physical_bytes, usage(connection, &owner, &org, &project)?.physical_bytes);
        assert_eq!(maintain(connection, &owner, &store, &org, &project)?.collected_objects, 1);
        Ok(())
    }).await.unwrap();
}

#[tokio::test]
async fn failed_file_delete_preserves_tombstone_and_retry_collects_once() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            append(connection, &owner, &store, &org, &project, 0, b"old")?;
            append(connection, &owner, &store, &org, &project, 1, b"head")?;
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            set_policy(connection, &owner, &org, &project, Some(1), 0, false)?;
            let path = store.root.join(digest(b"old"));
            let encrypted = std::fs::read(&path).unwrap();
            std::fs::write(&path, [0u8; 31]).unwrap();
            assert!(matches!(
                maintain(connection, &owner, &store, &org, &project),
                Err(ServiceError::OutcomeUnknown)
            ));
            assert_eq!(
                usage(connection, &owner, &org, &project)?.pending_gc_objects,
                1
            );
            assert_eq!(store.physical.lock().unwrap().unwrap().1, 2);
            std::fs::write(&path, encrypted).unwrap();
            let report = maintain(connection, &owner, &store, &org, &project)?;
            assert_eq!((report.collected_objects, report.pending_objects), (1, 0));
            assert_eq!(
                maintain(connection, &owner, &store, &org, &project)?.collected_objects,
                0
            );
            assert_eq!(store.physical.lock().unwrap().unwrap().1, 1);
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn recovery_replays_file_delete_when_database_ack_rolled_back() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let (org, project) = database.execute(move |connection| {
        let owner = principal("owner");
        let (org, project) = seed(connection, &owner);
        append(connection, &owner, &store, &org, &project, 0, b"old")?;
        append(connection, &owner, &store, &org, &project, 1, b"head")?;
        connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
        set_policy(connection, &owner, &org, &project, Some(1), 0, false)?;
        connection.execute_batch("CREATE TEMP TRIGGER fail_gc_ack BEFORE DELETE ON cloud_gc_pending BEGIN SELECT RAISE(ABORT,'injected'); END;")?;
        assert!(matches!(maintain(connection, &owner, &store, &org, &project), Err(ServiceError::OutcomeUnknown)));
        assert!(!store.root.join(digest(b"old")).exists());
        assert_eq!(usage(connection, &owner, &org, &project)?.pending_gc_objects, 1);
        connection.execute_batch("DROP TRIGGER fail_gc_ack;")?;
        Ok((org, project))
    }).await.unwrap();
    let config = files.config.clone();
    database
        .execute(move |connection| {
            let store = BlobStore::load(&config)?;
            store.recover(connection)?;
            let owner = principal("owner");
            assert_eq!(
                usage(connection, &owner, &org, &project)?.pending_gc_objects,
                0
            );
            assert_eq!(store.physical.lock().unwrap().unwrap().1, 1);
            assert_eq!(
                read_blob(connection, &owner, &store, &org, &project, &digest(b"head"))?,
                b"head"
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn same_base_writers_and_maintenance_never_collect_winner_or_live_uploads() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let second = Database::open(files.path.join("service.db")).unwrap();
    let third = Database::open(files.path.join("service.db")).unwrap();
    let setup_store = store.clone();
    let (org, project) = database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            set_policy(connection, &owner, &org, &project, Some(1), 0, false)?;
            append(connection, &owner, &setup_store, &org, &project, 0, b"old")?;
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            for bytes in [b"left".as_slice(), b"right".as_slice()] {
                upload(
                    connection,
                    &owner,
                    &setup_store,
                    &org,
                    &project,
                    &digest(bytes),
                    bytes,
                )?;
            }
            Ok((org, project))
        })
        .await
        .unwrap();
    let barrier = Arc::new(tokio::sync::Barrier::new(4));
    let mut writers = Vec::new();
    for (database, bytes) in [
        (database.clone(), b"left".as_slice()),
        (second, b"right".as_slice()),
    ] {
        let (org, project, store, barrier) =
            (org.clone(), project.clone(), store.clone(), barrier.clone());
        writers.push(tokio::spawn(async move {
            barrier.wait().await;
            database
                .execute(move |connection| {
                    commit(
                        connection,
                        &principal("owner"),
                        &store,
                        &org,
                        &project,
                        request("1", bytes),
                    )
                })
                .await
                .unwrap()
        }));
    }
    let maintenance = tokio::spawn({
        let (org, project, store, barrier) =
            (org.clone(), project.clone(), store.clone(), barrier.clone());
        async move {
            barrier.wait().await;
            third
                .execute(move |connection| {
                    maintain(connection, &principal("owner"), &store, &org, &project)
                })
                .await
                .unwrap()
        }
    });
    barrier.wait().await;
    let mut committed = 0;
    let mut conflicted = 0;
    for writer in writers {
        match writer.await.unwrap() {
            CommitOutcome::Committed { .. } => committed += 1,
            CommitOutcome::Conflict { .. } => conflicted += 1,
        }
    }
    maintenance.await.unwrap();
    assert_eq!((committed, conflicted), (1, 1));
    database
        .execute(move |connection| {
            let owner = principal("owner");
            maintain(connection, &owner, &store, &org, &project)?;
            let head = head(connection, &owner, &org, &project)?.unwrap();
            assert_eq!(head.revision, "2");
            let bytes = read_blob(
                connection,
                &owner,
                &store,
                &org,
                &project,
                &head.manifest.files[0].digest,
            )?;
            assert!(bytes.as_slice() == b"left" || bytes.as_slice() == b"right");
            assert_eq!(store.read(&digest(b"left"))?, b"left");
            assert_eq!(store.read(&digest(b"right"))?, b"right");
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            maintain(connection, &owner, &store, &org, &project)?;
            assert_eq!(usage(connection, &owner, &org, &project)?.objects, 1);
            assert_eq!(store.physical.lock().unwrap().unwrap().1, 1);
            assert_eq!(
                store.read(&head.manifest.files[0].digest)?.len() as u64,
                head.manifest.files[0].bytes
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn failed_head_commit_cannot_publish_pruning_or_lose_the_old_snapshot() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database.execute(move |connection| {
        let owner = principal("owner");
        let (org, project) = seed(connection, &owner);
        set_policy(connection, &owner, &org, &project, Some(1), 0, false)?;
        append(connection, &owner, &store, &org, &project, 0, b"old head")?;
        connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
        upload(connection, &owner, &store, &org, &project, &digest(b"new head"), b"new head")?;
        connection.execute_batch("CREATE TEMP TRIGGER fail_commit BEFORE INSERT ON audit_events WHEN NEW.action='cloud.commit' BEGIN SELECT RAISE(ABORT,'injected'); END;")?;
        let request = request("1", b"new head");
        assert!(matches!(commit(connection, &owner, &store, &org, &project, request.clone()), Err(ServiceError::Storage)));
        assert_eq!(head(connection, &owner, &org, &project)?.unwrap().revision, "1");
        assert_eq!(read_blob(connection, &owner, &store, &org, &project, &digest(b"old head"))?, b"old head");
        assert_eq!(usage(connection, &owner, &org, &project)?.pending_gc_objects, 0);
        assert!(matches!(receipt::lookup(connection, &owner, &request.operation_id)?, receipt::OperationStatus::Unknown));
        connection.execute_batch("DROP TRIGGER fail_commit;")?;
        commit(connection, &owner, &store, &org, &project, request)?;
        maintain(connection, &owner, &store, &org, &project)?;
        assert_eq!(head(connection, &owner, &org, &project)?.unwrap().revision, "2");
        assert!(!store.root.join(digest(b"old head")).exists());
        Ok(())
    }).await.unwrap();
}

#[tokio::test]
async fn restart_preserves_an_upload_lease_until_its_snapshot_commit() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let (org, project) = database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            set_policy(connection, &owner, &org, &project, Some(1), 0, false)?;
            upload(
                connection,
                &owner,
                &store,
                &org,
                &project,
                &digest(b"pending"),
                b"pending",
            )?;
            maintain(connection, &owner, &store, &org, &project)?;
            Ok((org, project))
        })
        .await
        .unwrap();
    let config = files.config.clone();
    database
        .execute(move |connection| {
            let store = BlobStore::load(&config)?;
            store.recover(connection)?;
            let owner = principal("owner");
            assert_eq!(
                maintain(connection, &owner, &store, &org, &project)?.collected_objects,
                0
            );
            commit(
                connection,
                &owner,
                &store,
                &org,
                &project,
                request("0", b"pending"),
            )?;
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            maintain(connection, &owner, &store, &org, &project)?;
            assert_eq!(
                read_blob(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(b"pending")
                )?,
                b"pending"
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn bounded_maintenance_reports_expired_uploads_not_yet_staged() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            set_policy(connection, &owner, &org, &project, Some(1), 0, false)?;
            for value in 0..65u64 {
                let bytes = value.to_le_bytes();
                upload(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(&bytes),
                    &bytes,
                )?;
            }
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            let first = maintain(connection, &owner, &store, &org, &project)?;
            assert_eq!((first.collected_objects, first.pending_objects), (64, 1));
            assert_eq!(store.physical.lock().unwrap().unwrap().1, 1);
            let second = maintain(connection, &owner, &store, &org, &project)?;
            assert_eq!((second.collected_objects, second.pending_objects), (1, 0));
            assert_eq!(*store.physical.lock().unwrap(), Some((0, 0)));
            assert!(head(connection, &owner, &org, &project)?.is_none());
            Ok(())
        })
        .await
        .unwrap();
}
