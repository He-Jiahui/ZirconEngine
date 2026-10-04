use super::support::*;
use crate::service::{cloud::*, error::ServiceError};
use rusqlite::params;
use std::fs;

#[tokio::test]
async fn encrypted_blobs_are_idempotent_and_require_current_tenant_snapshot_reference() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let root = files.config.root.clone();
    database
        .execute(move |connection| {
            let alice = principal("alice");
            let bob = principal("bob");
            let (a, project) = seed(connection, &alice);
            let (b, _) = seed(connection, &bob);
            let bytes = b"snapshot payload";
            let hash = digest(bytes);
            upload(connection, &alice, &store, &a, &project, &hash, bytes)?;
            let ciphertext = fs::read(root.join(&hash)).unwrap();
            upload(connection, &alice, &store, &a, &project, &hash, bytes)?;
            assert_eq!(fs::read(root.join(&hash)).unwrap(), ciphertext);
            assert!(!ciphertext.windows(bytes.len()).any(|part| part == bytes));
            assert!(matches!(
                read_blob(connection, &alice, &store, &a, &project, &hash),
                Err(ServiceError::Forbidden)
            ));
            commit(
                connection,
                &alice,
                &store,
                &a,
                &project,
                request("0", bytes),
            )?;
            assert_eq!(
                read_blob(connection, &alice, &store, &a, &project, &hash)?,
                bytes
            );
            assert!(matches!(
                read_blob(connection, &bob, &store, &a, &project, &hash),
                Err(ServiceError::Forbidden)
            ));
            assert!(matches!(
                read_blob(connection, &bob, &store, &b, &project, &hash),
                Err(ServiceError::Forbidden)
            ));
            upload(connection, &bob, &store, &b, &project, &hash, bytes)?;
            assert!(matches!(
                read_blob(connection, &bob, &store, &b, &project, &hash),
                Err(ServiceError::Forbidden)
            ));
            commit(connection, &bob, &store, &b, &project, request("0", bytes))?;
            assert_eq!(fs::read(root.join(&hash)).unwrap(), ciphertext);
            connection.execute(
                "UPDATE memberships SET role='viewer' WHERE organization_id=?1",
                [&a],
            )?;
            assert!(matches!(
                upload(connection, &alice, &store, &a, &project, &hash, bytes),
                Err(ServiceError::Forbidden)
            ));
            assert_eq!(
                read_blob(connection, &alice, &store, &a, &project, &hash)?,
                bytes
            );
            connection.execute(
                "UPDATE memberships SET active=0 WHERE organization_id=?1",
                [&a],
            )?;
            assert!(matches!(
                read_blob(connection, &alice, &store, &a, &project, &hash),
                Err(ServiceError::Forbidden)
            ));
            assert_eq!(
                read_blob(connection, &bob, &store, &b, &project, &hash)?,
                bytes
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn quotas_and_bad_digest_do_not_publish_or_admit_blob_rows() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (org, project) = seed(connection, &owner);
            assert!(matches!(
                upload(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &"a".repeat(64),
                    b"bad digest"
                ),
                Err(ServiceError::InvalidRequest)
            ));
            connection.execute(
                "INSERT INTO cloud_blobs VALUES (?1,?2,?3,?4)",
                params![
                    org,
                    project,
                    "b".repeat(64),
                    crate::service::cloud::manifest::MAX_PROJECT_BYTES
                ],
            )?;
            assert!(matches!(
                upload(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(b"next"),
                    b"next"
                ),
                Err(ServiceError::Capacity)
            ));
            assert!(!store.root.join(digest(b"next")).exists());
            connection.execute("DELETE FROM cloud_upload_leases", [])?;
            connection.execute("DELETE FROM cloud_blobs", [])?;
            *store.physical.lock().unwrap() =
                Some((crate::service::cloud::quota::MAX_GLOBAL_BYTES, 1));
            assert!(matches!(
                upload(
                    connection,
                    &owner,
                    &store,
                    &org,
                    &project,
                    &digest(b"next"),
                    b"next"
                ),
                Err(ServiceError::Capacity)
            ));
            assert_eq!(usage(connection, &owner, &org, &project)?.objects, 0);
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn global_quota_counts_shared_digest_once_across_tenants() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (first, project) = seed(connection, &owner);
            let (second, _) = seed(connection, &owner);
            let (third, _) = seed(connection, &owner);
            let shared_digest = digest(b"shared");
            let half_global = crate::service::cloud::quota::MAX_GLOBAL_BYTES / 2 - 1;
            for organization in [&first, &second] {
                connection.execute(
                    "INSERT INTO cloud_blobs VALUES (?1,?2,?3,?4)",
                    params![
                        organization.as_str(),
                        project.as_str(),
                        &shared_digest,
                        half_global
                    ],
                )?;
            }

            // Two tenant rows point at one physical CAS object. Their logical
            // rows nearly fill the global byte budget, but the deduplicated
            // physical digest plus this upload remains within the limit.
            let next_digest = digest(b"next");
            upload(
                connection,
                &owner,
                &store,
                &third,
                &project,
                &next_digest,
                b"next",
            )?;
            assert!(store.root.join(next_digest).is_file());
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn shared_digest_reuse_is_allowed_when_global_quota_is_full() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (first, project) = seed(connection, &owner);
            let (second, _) = seed(connection, &owner);
            let shared_bytes = b"shared";
            let shared_digest = digest(shared_bytes);
            upload(
                connection,
                &owner,
                &store,
                &first,
                &project,
                &shared_digest,
                shared_bytes,
            )?;

            // Fill the logical edge with a separate digest. The shared digest
            // already has a physical CAS file, so the second tenant needs no
            // additional global allocation even though its project row is new.
            connection.execute(
                "INSERT INTO cloud_blobs VALUES (?1,?2,?3,?4)",
                params![
                    first.as_str(),
                    project.as_str(),
                    &"f".repeat(64),
                    crate::service::cloud::quota::MAX_GLOBAL_BYTES - shared_bytes.len() as u64
                ],
            )?;
            upload(
                connection,
                &owner,
                &store,
                &second,
                &project,
                &shared_digest,
                shared_bytes,
            )?;
            assert_eq!(store.read(&shared_digest)?, shared_bytes);
            assert_eq!(
                connection.query_row(
                    "SELECT COUNT(*) FROM cloud_blobs WHERE digest=?1",
                    [&shared_digest],
                    |row| row.get::<_, u64>(0),
                )?,
                2
            );
            Ok(())
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn restart_accounts_orphans_removes_partial_and_rejects_corruption() {
    let files = Files::new();
    let (database, store) = files.database().await;
    assert!(matches!(
        BlobStore::load(&files.config),
        Err(ServiceError::Capacity)
    ));
    let partial = files
        .config
        .root
        .join(format!("{}.partial", uuid::Uuid::new_v4()));
    fs::write(&partial, b"interrupted").unwrap();
    let hash = digest(b"orphan");
    let hash_for_upload = hash.clone();
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
                &hash_for_upload,
                b"orphan",
            )?;
            connection.execute("DELETE FROM cloud_upload_leases", [])?;
            connection.execute("DELETE FROM cloud_blobs", [])?;
            Ok(())
        })
        .await
        .unwrap();
    let reopened = BlobStore::load(&files.config).unwrap();
    database
        .execute(move |connection| {
            reopened.recover(connection)?;
            assert!(!partial.exists());
            assert_eq!(*reopened.physical.lock().unwrap(), Some((6 + 28, 1)));
            assert_eq!(reopened.read(&hash)?, b"orphan");
            fs::write(reopened.root.join(&hash), [0u8; 34]).unwrap();
            assert!(matches!(reopened.read(&hash), Err(ServiceError::Storage)));
            Ok(())
        })
        .await
        .unwrap();
}
