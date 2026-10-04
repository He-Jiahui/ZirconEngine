use super::support::{digest, principal, seed, Files};
use crate::service::{
    cloud::{upload, BlobStore, CloudConfig},
    error::ServiceError,
    storage::Database,
};
use std::{fs, path::Path, sync::Arc};

async fn recover(database: &Database, config: CloudConfig) -> Result<BlobStore, ServiceError> {
    database
        .execute(move |connection| {
            let store = BlobStore::load(&config)?;
            store.recover(connection)?;
            Ok(store)
        })
        .await
}

fn copy_store(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() != ".store.lock" {
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
}

#[tokio::test]
async fn binding_rejects_another_root_and_another_key_independently() {
    let files = Files::new();
    let database = Database::open(files.path.join("service.db")).unwrap();
    let first = recover(&database, files.config.clone()).await.unwrap();
    let store_id = fs::read_to_string(first.root.join(".store-id")).unwrap();
    assert_eq!(
        uuid::Uuid::parse_str(&store_id).unwrap().to_string(),
        store_id
    );
    drop(first);

    let another_root = CloudConfig {
        root: files.path.join("another-cas"),
        key_file: files.config.key_file.clone(),
    };
    assert!(matches!(
        recover(&database, another_root.clone()).await,
        Err(ServiceError::Configuration)
    ));
    assert!(!another_root.root.join(".store-id").exists());

    let another_key = files.path.join("another.key");
    fs::write(&another_key, [9u8; 32]).unwrap();
    assert!(matches!(
        recover(
            &database,
            CloudConfig {
                root: files.config.root.clone(),
                key_file: another_key,
            },
        )
        .await,
        Err(ServiceError::Configuration)
    ));
    recover(&database, files.config.clone()).await.unwrap();
}

#[tokio::test]
async fn populated_backup_recovers_at_new_database_root_and_key_paths() {
    let files = Files::new();
    let (database, store) = files.database().await;
    let uploaded = store.clone();
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (organization, project) = seed(connection, &owner);
            upload(
                connection,
                &owner,
                &uploaded,
                &organization,
                &project,
                &digest(b"backup payload"),
                b"backup payload",
            )
        })
        .await
        .unwrap();

    let restored = files.path.join("restored");
    fs::create_dir_all(&restored).unwrap();
    let backup_path = restored.join("service.db");
    let backup_destination = backup_path.to_str().unwrap().to_string();
    database
        .execute(move |connection| {
            connection.execute("VACUUM INTO ?1", [backup_destination])?;
            Ok(())
        })
        .await
        .unwrap();
    let config = CloudConfig {
        root: restored.join("cas"),
        key_file: restored.join("cloud.key"),
    };
    copy_store(&files.config.root, &config.root);
    fs::copy(&files.config.key_file, &config.key_file).unwrap();
    drop(store);

    let restored_database = Database::open(backup_path).unwrap();
    let restored_store = recover(&restored_database, config).await.unwrap();
    assert_eq!(
        restored_store.read(&digest(b"backup payload")).unwrap(),
        b"backup payload"
    );
    assert_eq!(*restored_store.physical.lock().unwrap(), Some((42, 1)));
}

#[tokio::test]
async fn moved_root_keeps_its_database_binding() {
    let files = Files::new();
    let (database, store) = files.database().await;
    drop(store);
    let moved_root = files.path.join("moved-cas");
    fs::rename(&files.config.root, &moved_root).unwrap();
    recover(
        &database,
        CloudConfig {
            root: moved_root,
            key_file: files.config.key_file.clone(),
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn concurrent_copied_roots_have_one_database_owner() {
    let files = Files::new();
    let path = files.path.join("service.db");
    let first_database = Database::open(path.clone()).unwrap();
    let first_store = recover(&first_database, files.config.clone())
        .await
        .unwrap();
    let copied = CloudConfig {
        root: files.path.join("copied-cas"),
        key_file: files.config.key_file.clone(),
    };
    copy_store(&files.config.root, &copied.root);
    drop(first_store);
    let alias = files.path.join("alias");
    fs::create_dir(&alias).unwrap();
    let second_database = Database::open(alias.join("..").join("service.db")).unwrap();
    let start = Arc::new(tokio::sync::Barrier::new(3));
    let mut tasks = Vec::new();
    for (database, config) in [
        (first_database.clone(), files.config.clone()),
        (second_database, copied.clone()),
    ] {
        let start = start.clone();
        tasks.push(tokio::spawn(async move {
            start.wait().await;
            recover(&database, config).await
        }));
    }
    start.wait().await;
    let mut owners = Vec::new();
    let mut rejected = 0;
    for task in tasks {
        match task.await.unwrap() {
            Ok(owner) => owners.push(owner),
            Err(ServiceError::Capacity) => rejected += 1,
            Err(error) => panic!("unexpected recovery failure: {error}"),
        }
    }
    assert_eq!((owners.len(), rejected), (1, 1));
    drop(owners);
    recover(&first_database, copied).await.unwrap();
}

#[tokio::test]
async fn v4_adoption_validates_orphan_ciphertext_before_binding_a_key() {
    let files = Files::new();
    let (database, store) = files.database().await;
    database
        .execute(move |connection| {
            let owner = principal("owner");
            let (organization, project) = seed(connection, &owner);
            upload(
                connection,
                &owner,
                &store,
                &organization,
                &project,
                &digest(b"unpublished"),
                b"unpublished",
            )?;
            connection.execute_batch(
                "DROP TRIGGER maintenance_operation_reserved;
                 DROP TABLE cloud_maintenance_runs; DROP TABLE cloud_gc_pending; DROP TABLE cloud_upload_leases;
                 DROP TABLE cloud_snapshot_trash; DROP TABLE cloud_retention_policies;
                 DROP INDEX cloud_blobs_digest; DROP INDEX snapshot_blobs_digest;
                 DROP TABLE catalog_artifacts; DELETE FROM cloud_blobs;
                 DROP TABLE cloud_store_binding; PRAGMA user_version=4;",
            )?;
            Ok(())
        })
        .await
        .unwrap();
    fs::remove_file(files.config.root.join(".store-id")).unwrap();
    let migrated = Database::open(files.path.join("service.db")).unwrap();
    let bad_key = files.path.join("wrong.key");
    fs::write(&bad_key, [9u8; 32]).unwrap();
    assert!(matches!(
        recover(
            &migrated,
            CloudConfig {
                root: files.config.root.clone(),
                key_file: bad_key,
            },
        )
        .await,
        Err(ServiceError::Storage)
    ));
    assert!(!files.config.root.join(".store-id").exists());
    migrated
        .execute(|connection| {
            assert_eq!(
                connection.query_row("SELECT COUNT(*) FROM cloud_store_binding", [], |row| {
                    row.get::<_, u64>(0)
                })?,
                0
            );
            Ok(())
        })
        .await
        .unwrap();
    let adopted = recover(&migrated, files.config.clone()).await.unwrap();
    assert_eq!(
        adopted.read(&digest(b"unpublished")).unwrap(),
        b"unpublished"
    );
    assert!(files.config.root.join(".store-id").is_file());
}

#[tokio::test]
async fn store_identity_must_be_a_canonical_uuid_in_a_regular_file() {
    let files = Files::new();
    let (database, store) = files.database().await;
    drop(store);
    let path = files.config.root.join(".store-id");
    #[cfg(unix)]
    let valid_id = fs::read_to_string(&path).unwrap();
    fs::write(&path, "AAAAAAAA-AAAA-4AAA-AAAA-AAAAAAAAAAAA").unwrap();
    assert!(matches!(
        recover(&database, files.config.clone()).await,
        Err(ServiceError::Configuration)
    ));
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(matches!(
        recover(&database, files.config.clone()).await,
        Err(ServiceError::Configuration)
    ));

    #[cfg(unix)]
    {
        fs::remove_dir(&path).unwrap();
        let target = files.path.join("store-id-target");
        fs::write(&target, valid_id).unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(matches!(
            recover(&database, files.config.clone()).await,
            Err(ServiceError::Configuration)
        ));
    }
}
