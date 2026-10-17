use super::support::*;
use crate::service::{
    cloud::*,
    error::ServiceError,
    storage::{receipt, Database},
};
use std::sync::Arc;

#[tokio::test]
async fn concurrent_same_base_has_one_commit_and_one_durable_conflict() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let second = Database::open(files.path.join("service.db")).unwrap();
    let owner = principal("owner");
    let setup_owner = owner.clone();
    let setup_store = store.clone();
    let (org, project) = database
        .execute(move |connection| {
            let ids = seed(connection, &setup_owner);
            upload(
                connection,
                &setup_owner,
                &setup_store,
                &ids.0,
                &ids.1,
                &digest(b"content"),
                b"content",
            )?;
            Ok(ids)
        })
        .await
        .unwrap();
    let barrier = Arc::new(tokio::sync::Barrier::new(3));
    let requests = [request("0", b"content"), request("0", b"content")];
    let mut tasks = Vec::new();
    for (database, request) in [database.clone(), second].into_iter().zip(requests.clone()) {
        let (store, owner, org, project, barrier) = (
            store.clone(),
            owner.clone(),
            org.clone(),
            project.clone(),
            barrier.clone(),
        );
        tasks.push(tokio::spawn(async move {
            barrier.wait().await;
            database
                .execute(move |connection| {
                    commit(connection, &owner, &store, &org, &project, request)
                })
                .await
                .unwrap()
        }));
    }
    barrier.wait().await;
    let mut successes = 0;
    let mut conflicts = 0;
    for task in tasks {
        match task.await.unwrap() {
            CommitOutcome::Committed { snapshot } => {
                successes += 1;
                assert_eq!(snapshot.revision, "1");
            }
            CommitOutcome::Conflict {
                base_revision,
                current_revision,
                ..
            } => {
                conflicts += 1;
                assert_eq!(base_revision, "0");
                assert_eq!(current_revision, "1");
            }
        }
    }
    assert_eq!((successes, conflicts), (1, 1));
    database
        .execute(move |connection| {
            assert_eq!(
                head(connection, &owner, &org, &project)?.unwrap().revision,
                "1"
            );
            for request in requests {
                let operation = request.operation_id.clone();
                let first = serde_json::to_value(commit(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    request.clone(),
                )?)
                .unwrap();
                let second = serde_json::to_value(commit(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    request.clone(),
                )?)
                .unwrap();
                assert_eq!(first, second);
                let receipt::OperationStatus::Committed { result } =
                    receipt::lookup(connection, &owner, &operation)?
                else {
                    panic!("missing terminal receipt")
                };
                assert_eq!(result, first);
                if first["status"] == "committed" {
                    assert!(first["snapshot"].get("manifest").is_none());
                }
                let mut changed = request.clone();
                changed.base_revision = "1".into();
                assert!(matches!(
                    commit(connection, &owner, &store, &org, &project, changed),
                    Err(ServiceError::OperationConflict)
                ));
            }
            let count: u64 =
                connection.query_row("SELECT COUNT(*) FROM project_snapshots", [], |row| {
                    row.get(0)
                })?;
            assert_eq!(count, 1);
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn durable_commit_and_conflict_replay_ignore_unrelated_collection_failure() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            upload(
                connection,
                &owner,
                &store,
                &org,
                &project,
                &digest(b"head"),
                b"head",
            )?;
            let requests = [request("0", b"head"), request("0", b"head")];
            let mut outcomes = Vec::new();
            for request in &requests {
                outcomes.push(
                    serde_json::to_value(commit(
                        connection,
                        &owner,
                        &store,
                        &org,
                        &project,
                        request.clone(),
                    )?)
                    .unwrap(),
                );
            }
            assert_eq!(outcomes[0]["status"], "committed");
            assert_eq!(outcomes[1]["status"], "conflict");

            let orphan = digest(b"orphan");
            std::fs::write(store.root.join(&orphan), b"invalid ciphertext").unwrap();
            connection.execute(
                "INSERT INTO cloud_gc_pending VALUES (?1,?2,?3,?4)",
                rusqlite::params![orphan, 6, org, project],
            )?;
            assert!(matches!(
                super::super::retention::collect_before_write(
                    connection, &owner, &store, &org, &project,
                ),
                Err(ServiceError::Storage)
            ));
            for (request, expected) in requests.into_iter().zip(outcomes) {
                let actual = commit(connection, &owner, &store, &org, &project, request.clone())?;
                assert_eq!(serde_json::to_value(actual).unwrap(), expected);
                let receipt::OperationStatus::Committed { result } =
                    receipt::lookup(connection, &owner, &request.operation_id)?
                else {
                    panic!("replay lost a durable terminal result")
                };
                assert_eq!(result, expected);
            }
            assert_eq!(
                head(connection, &owner, &org, &project)?.unwrap().revision,
                "1"
            );
            assert_eq!(
                connection.query_row("SELECT COUNT(*) FROM cloud_gc_pending", [], |row| row
                    .get::<_, u64>(0))?,
                1
            );
            assert_eq!(
                std::fs::read(store.root.join(&orphan)).unwrap(),
                b"invalid ciphertext"
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn missing_blob_and_failed_audit_leave_no_head_or_reference() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database.execute(move |connection| {
        let owner = principal("owner");
        let (org, project) = seed(connection, &owner);
        assert!(matches!(commit(connection, &owner, &store, &org, &project, request("0", b"content")), Err(ServiceError::InvalidRequest)));
        upload(connection, &owner, &store, &org, &project, &digest(b"content"), b"content")?;
        let commit_request = request("0", b"content");
        connection.execute_batch("CREATE TEMP TRIGGER fail_cloud_audit BEFORE INSERT ON audit_events WHEN NEW.action='cloud.commit' BEGIN SELECT RAISE(ABORT,'injected'); END;")?;
        assert!(matches!(commit(connection, &owner, &store, &org, &project, commit_request.clone()), Err(ServiceError::Storage)));
        assert!(head(connection, &owner, &org, &project)?.is_none());
        assert!(matches!(receipt::lookup(connection, &owner, &commit_request.operation_id)?, receipt::OperationStatus::Unknown));
        let refs: u64 = connection.query_row("SELECT COUNT(*) FROM snapshot_blobs", [], |row| row.get(0))?;
        assert_eq!(refs, 0);
        connection.execute_batch("DROP TRIGGER fail_cloud_audit;")?;
        assert!(matches!(commit(connection, &owner, &store, &org, &project, commit_request.clone())?, CommitOutcome::Committed { .. }));
        let receipt::OperationStatus::Committed { result } = receipt::lookup(connection, &owner, &commit_request.operation_id)? else { panic!("missing committed receipt") };
        connection.execute("UPDATE memberships SET active=0", [])?;
        assert!(matches!(commit(connection, &owner, &store, &org, &project, commit_request), Err(ServiceError::Forbidden)));
        assert!(matches!(authorize_receipt(connection, &owner, &result), Err(ServiceError::Forbidden)));
        Ok(())
    }).await.unwrap();
}

#[tokio::test]
async fn retention_limit_preserves_every_existing_revision_and_reference() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            upload(
                connection,
                &owner,
                &store,
                &org,
                &project,
                &digest(b"content"),
                b"content",
            )?;
            for base in 0..crate::service::cloud::quota::MAX_REVISIONS {
                assert!(matches!(
                    commit(
                        connection,
                        &owner,
                        &store,
                        &org,
                        &project,
                        request(&base.to_string(), b"content")
                    )?,
                    CommitOutcome::Committed { .. }
                ));
            }
            assert!(matches!(
                commit(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    request("128", b"content")
                ),
                Err(ServiceError::Capacity)
            ));
            assert_eq!(usage(connection, &owner, &org, &project)?.revisions, 128);
            assert_eq!(
                read_blob(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(b"content")
                )?,
                b"content"
            );
            Ok(())
        })
        .await
        .unwrap();
}
