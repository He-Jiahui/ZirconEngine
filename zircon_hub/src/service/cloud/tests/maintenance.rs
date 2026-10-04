use super::support::*;
use crate::service::{
    cloud::*,
    error::ServiceError,
    storage::{receipt, Database},
};
use rusqlite::Connection;
use std::{sync::Arc, time::Duration};

fn request_maintenance() -> MaintenanceRequest {
    MaintenanceRequest {
        operation_id: uuid::Uuid::new_v4().to_string(),
    }
}

fn rolling(connection: &mut Connection, org: &str, project: &str) -> Result<(), ServiceError> {
    update_retention(
        connection,
        &principal("owner"),
        org,
        project,
        RetentionRequest {
            operation_id: uuid::Uuid::new_v4().to_string(),
            expected_revision: "0".into(),
            policy: RetentionPolicy {
                keep_latest: Some(1),
                trash_seconds: 0,
                legal_hold: false,
            },
        },
    )?;
    Ok(())
}

#[tokio::test]
async fn maintenance_replays_one_receipt_and_checks_current_authority() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            rolling(connection, &org, &project)?;
            upload(
                connection,
                &owner,
                &store,
                &org,
                &project,
                &digest(b"old"),
                b"old",
            )?;
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            let request = request_maintenance();
            let first = maintain(connection, &owner, &store, &org, &project, request.clone())?;
            assert_eq!(first.collected_objects, 1);
            assert_eq!(
                maintain(connection, &owner, &store, &org, &project, request.clone())?,
                first
            );
            let audits: u64 = connection.query_row(
                "SELECT COUNT(*) FROM audit_events WHERE action LIKE 'cloud.maintenance.%'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(audits, 2);
            let receipt::OperationStatus::Committed { result } =
                receipt::lookup(connection, &owner, &request.operation_id)?
            else {
                panic!("missing terminal maintenance receipt")
            };
            assert_eq!(result, serde_json::to_value(&first).unwrap());
            let (other, other_project) = seed(connection, &owner);
            assert!(matches!(
                maintain(
                    connection,
                    &owner,
                    &store,
                    &other,
                    &other_project,
                    request.clone()
                ),
                Err(ServiceError::OperationConflict)
            ));
            connection.execute(
                "UPDATE memberships SET role='viewer' WHERE organization_id=?1",
                [&org],
            )?;
            assert!(matches!(
                maintain(connection, &owner, &store, &org, &project, request.clone()),
                Err(ServiceError::Forbidden)
            ));
            connection.execute(
                "UPDATE memberships SET active=0 WHERE organization_id=?1",
                [&org],
            )?;
            assert!(matches!(
                authorize_receipt(connection, &owner, &result),
                Err(ServiceError::Forbidden)
            ));
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn prepared_maintenance_survives_delete_ack_fault_and_restart() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let request = request_maintenance();
    let prepared_request = request.clone();
    let (org, project) = database.execute(move |connection| {
        let owner = principal("owner");
        let (org, project) = seed(connection, &owner);
        rolling(connection, &org, &project)?;
        upload(connection, &owner, &store, &org, &project, &digest(b"old"), b"old")?;
        connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
        connection.execute_batch("CREATE TEMP TRIGGER fail_ack BEFORE DELETE ON cloud_gc_pending BEGIN SELECT RAISE(ABORT,'injected'); END;")?;
        assert!(matches!(maintain(connection, &owner, &store, &org, &project, prepared_request.clone()), Err(ServiceError::OutcomeUnknown)));
        assert!(!store.root.join(digest(b"old")).exists());
        assert!(matches!(receipt::lookup(connection, &owner, &prepared_request.operation_id)?, receipt::OperationStatus::Unknown));
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM cloud_maintenance_runs", [], |row| row.get::<_, u64>(0))?, 1);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM audit_events WHERE action='cloud.maintenance.prepare'", [], |row| row.get::<_, u64>(0))?, 1);
        assert!(matches!(update_retention(connection, &owner, &org, &project, RetentionRequest {
            operation_id: prepared_request.operation_id.clone(), expected_revision: "1".into(),
            policy: RetentionPolicy { keep_latest: Some(2), trash_seconds: 0, legal_hold: false },
        }), Err(ServiceError::OperationConflict)));
        assert_eq!(retention(connection, &owner, &org, &project)?.revision, "1");
        assert!(matches!(crate::service::organization::create(connection, &owner, &prepared_request.operation_id, "reserved"), Err(ServiceError::OperationConflict)));
        connection.execute_batch("DROP TRIGGER fail_ack;")?;
        Ok((org, project))
    }).await.unwrap();
    let config = files.config.clone();
    database
        .execute(move |connection| {
            let store = BlobStore::load(&config)?;
            store.recover(connection)?;
            let owner = principal("owner");
            let resumed = maintain(connection, &owner, &store, &org, &project, request.clone())?;
            assert_eq!(resumed.pending_objects, 0);
            assert_eq!(
                maintain(connection, &owner, &store, &org, &project, request)?,
                resumed
            );
            assert_eq!(
                connection
                    .query_row("SELECT COUNT(*) FROM cloud_maintenance_runs", [], |row| row
                        .get::<_, u64>(0))?,
                0
            );
            assert_eq!(
                connection.query_row(
                    "SELECT COUNT(*) FROM audit_events WHERE action='cloud.maintenance.prepare'",
                    [],
                    |row| row.get::<_, u64>(0)
                )?,
                1
            );
            assert_eq!(*store.physical.lock().unwrap(), Some((0, 0)));
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn concurrent_maintenance_retries_commit_one_terminal_result() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let second = Database::open(files.path.join("service.db")).unwrap();
    let setup_store = store.clone();
    let (org, project) = database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            rolling(connection, &org, &project)?;
            upload(
                connection,
                &owner,
                &setup_store,
                &org,
                &project,
                &digest(b"old"),
                b"old",
            )?;
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            Ok((org, project))
        })
        .await
        .unwrap();
    let request = request_maintenance();
    let barrier = Arc::new(tokio::sync::Barrier::new(3));
    let mut workers = Vec::new();
    for database in [database.clone(), second] {
        let (org, project, store, request, barrier) = (
            org.clone(),
            project.clone(),
            store.clone(),
            request.clone(),
            barrier.clone(),
        );
        workers.push(tokio::spawn(async move {
            barrier.wait().await;
            database
                .execute(move |connection| {
                    maintain(
                        connection,
                        &principal("owner"),
                        &store,
                        &org,
                        &project,
                        request,
                    )
                })
                .await
                .unwrap()
        }));
    }
    barrier.wait().await;
    let first = workers.remove(0).await.unwrap();
    let second = workers.remove(0).await.unwrap();
    assert_eq!(first, second);
    assert_eq!(first.collected_objects, 1);
    database
        .execute(move |connection| {
            assert_eq!(
                connection.query_row(
                    "SELECT COUNT(*) FROM operation_receipts WHERE operation_id=?1",
                    [&request.operation_id],
                    |row| row.get::<_, u64>(0)
                )?,
                1
            );
            assert_eq!(
                connection.query_row(
                    "SELECT COUNT(*) FROM audit_events WHERE action LIKE 'cloud.maintenance.%'",
                    [],
                    |row| row.get::<_, u64>(0)
                )?,
                2
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn recovery_waiting_for_writer_does_not_hold_physical_lock() {
    let files = Files::new();
    let database = Database::open(files.path.join("service.db")).unwrap();
    let connection = Connection::open(files.path.join("service.db")).unwrap();
    connection.execute_batch("BEGIN IMMEDIATE;").unwrap();
    let store = Arc::new(BlobStore::load(&files.config).unwrap());
    let recovery_store = store.clone();
    let recovery = tokio::spawn(async move {
        database
            .execute(move |connection| recovery_store.recover(connection))
            .await
    });
    let deadline = tokio::time::Instant::now() + Duration::from_secs(1);
    let mut started = false;
    while tokio::time::Instant::now() < deadline {
        if store.database_owner.try_lock().is_err() {
            started = true;
            break;
        }
        tokio::task::yield_now().await;
    }
    let writer_can_access_physical = store.physical.try_lock().is_ok();
    connection.execute_batch("ROLLBACK;").unwrap();
    recovery.await.unwrap().unwrap();
    assert!(started);
    assert!(writer_can_access_physical);
    let database = Database::open(files.path.join("service.db")).unwrap();
    let upload_store = store.clone();
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            upload(
                connection,
                &owner,
                &upload_store,
                &org,
                &project,
                &digest(b"after recovery"),
                b"after recovery",
            )?;
            assert_eq!(
                upload_store.read(&digest(b"after recovery"))?,
                b"after recovery"
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn demotion_after_prepare_reports_unknown_and_retry_preserves_pruned_count() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database.execute(move |connection| {
        let owner = principal("owner");
        let (org, project) = seed(connection, &owner);
        for (base, bytes) in [("0", b"old".as_slice()), ("1", b"head".as_slice())] {
            upload(connection, &owner, &store, &org, &project, &digest(bytes), bytes)?;
            commit(connection, &owner, &store, &org, &project, request(base, bytes))?;
        }
        update_retention(connection, &owner, &org, &project, RetentionRequest {
            operation_id: uuid::Uuid::new_v4().to_string(), expected_revision: "0".into(),
            policy: RetentionPolicy { keep_latest: Some(1), trash_seconds: 3600, legal_hold: false },
        })?;
        connection.execute_batch("UPDATE cloud_snapshot_trash SET purge_after=0; UPDATE cloud_upload_leases SET expires_at=0;
            CREATE TEMP TRIGGER demote_after_prepare AFTER INSERT ON audit_events
            WHEN NEW.action='cloud.maintenance.prepare' BEGIN UPDATE memberships SET role='viewer'; END;")?;
        let request = request_maintenance();
        assert!(matches!(maintain(connection, &owner, &store, &org, &project, request.clone()), Err(ServiceError::OutcomeUnknown)));
        assert_eq!(usage(connection, &owner, &org, &project)?.revisions, 1);
        assert_eq!(usage(connection, &owner, &org, &project)?.pending_gc_objects, 1);
        assert_eq!(store.read(&digest(b"old"))?, b"old");
        assert!(matches!(receipt::lookup(connection, &owner, &request.operation_id)?, receipt::OperationStatus::Unknown));
        assert!(matches!(maintain(connection, &owner, &store, &org, &project, request.clone()), Err(ServiceError::OutcomeUnknown)));
        assert!(matches!(maintain(connection, &owner, &store, &org, &project, request_maintenance()), Err(ServiceError::Forbidden)));
        connection.execute_batch("DROP TRIGGER demote_after_prepare; UPDATE memberships SET role='owner';")?;
        let resumed = maintain(connection, &owner, &store, &org, &project, request.clone())?;
        assert_eq!((resumed.pruned_revisions, resumed.collected_objects, resumed.pending_objects), (1, 1, 0));
        assert_eq!(maintain(connection, &owner, &store, &org, &project, request)?, resumed);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM audit_events WHERE action='cloud.maintenance.prepare'", [], |row| row.get::<_, u64>(0))?, 1);
        assert_eq!(read_blob(connection, &owner, &store, &org, &project, &digest(b"head"))?, b"head");
        Ok(())
    }).await.unwrap();
}

#[tokio::test]
async fn completion_audit_and_receipt_failures_are_unknown_and_replayable() {
    for trigger in [
        "CREATE TEMP TRIGGER fail_completion BEFORE INSERT ON audit_events WHEN NEW.action='cloud.maintenance.collect' BEGIN SELECT RAISE(ABORT,'injected'); END;",
        "CREATE TEMP TRIGGER fail_completion BEFORE INSERT ON operation_receipts WHEN json_extract(NEW.result_json,'$.status')='maintenanceCompleted' BEGIN SELECT RAISE(ABORT,'injected'); END;",
    ] {
        let files = Files::new();
        let (database, store) = files.database().await;
        database.execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            rolling(connection, &org, &project)?;
            upload(connection, &owner, &store, &org, &project, &digest(b"old"), b"old")?;
            connection.execute("UPDATE cloud_upload_leases SET expires_at=0", [])?;
            connection.execute_batch(trigger)?;
            let request = request_maintenance();
            assert!(matches!(maintain(connection, &owner, &store, &org, &project, request.clone()), Err(ServiceError::OutcomeUnknown)));
            assert!(!store.root.join(digest(b"old")).exists());
            assert!(matches!(receipt::lookup(connection, &owner, &request.operation_id)?, receipt::OperationStatus::Unknown));
            assert_eq!(connection.query_row("SELECT COUNT(*) FROM cloud_maintenance_runs", [], |row| row.get::<_, u64>(0))?, 1);
            connection.execute_batch("DROP TRIGGER fail_completion;")?;
            let resumed = maintain(connection, &owner, &store, &org, &project, request.clone())?;
            assert_eq!((resumed.collected_objects, resumed.pending_objects), (1, 0));
            assert_eq!(maintain(connection, &owner, &store, &org, &project, request)?, resumed);
            assert_eq!(connection.query_row("SELECT COUNT(*) FROM audit_events WHERE action LIKE 'cloud.maintenance.%'", [], |row| row.get::<_, u64>(0))?, 2);
            Ok(())
        }).await.unwrap();
    }
}
