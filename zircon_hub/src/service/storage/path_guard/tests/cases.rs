use super::*;
use crate::service::{error::ServiceError, storage::Database};
use std::sync::Arc;

struct Files(PathBuf);

impl Files {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("zircon-db-path-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&path).unwrap();
        crate::service::test_support::make_private_directory(&path);
        Self(path)
    }
}

impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn existing_hardlink_alias_is_rejected_before_migration() {
    let files = Files::new();
    let database = files.0.join("original.db");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute_batch("CREATE TABLE marker(value TEXT); PRAGMA user_version=0;")
        .unwrap();
    drop(connection);
    let alias = files.0.join("alias.db");
    std::fs::hard_link(&database, &alias).unwrap();
    let original = std::fs::read(&database).unwrap();
    assert!(matches!(
        Database::open(alias),
        Err(ServiceError::Configuration)
    ));
    assert_eq!(std::fs::read(&database).unwrap(), original);
    assert!(!files.0.join("alias.db-wal").exists());
}

#[test]
fn hostile_wal_hardlink_is_rejected_before_migration() {
    let files = Files::new();
    let database = files.0.join("service.db");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute_batch("CREATE TABLE marker(value TEXT); PRAGMA user_version=0;")
        .unwrap();
    drop(connection);

    let target = files.0.join("wal-target");
    std::fs::write(&target, b"preserve").unwrap();
    let wal = files.0.join("service.db-wal");
    std::fs::hard_link(&target, &wal).unwrap();
    let original = std::fs::read(&database).unwrap();

    assert!(matches!(
        Database::open(database.clone()),
        Err(ServiceError::Configuration)
    ));
    assert_eq!(std::fs::read(&database).unwrap(), original);
    assert_eq!(std::fs::read(&wal).unwrap(), b"preserve");
}

#[test]
fn hostile_shm_reparse_is_rejected_before_migration() {
    let files = Files::new();
    let database = files.0.join("service.db");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute_batch("CREATE TABLE marker(value TEXT); PRAGMA user_version=0;")
        .unwrap();
    drop(connection);

    let target = files.0.join("shm-target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("marker"), b"preserve").unwrap();
    let shm = files.0.join("service.db-shm");
    let junction = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg("$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:ZIRCON_DB_TEST_LINK -Target $env:ZIRCON_DB_TEST_TARGET | Out-Null")
        .env("ZIRCON_DB_TEST_LINK", &shm)
        .env("ZIRCON_DB_TEST_TARGET", &target)
        .output()
        .unwrap();
    assert!(
        junction.status.success(),
        "junction fixture must be created"
    );
    let original = std::fs::read(&database).unwrap();

    assert!(matches!(
        Database::open(database.clone()),
        Err(ServiceError::Configuration)
    ));
    assert_eq!(std::fs::read(&database).unwrap(), original);
    assert_eq!(std::fs::read(target.join("marker")).unwrap(), b"preserve");
    std::fs::remove_dir(shm).unwrap();
}

fn grant_everyone_access(path: &Path) {
    let result = std::process::Command::new("icacls.exe")
        .arg(path)
        .args(["/grant", "*S-1-1-0:(F)"])
        .output()
        .unwrap();
    assert!(result.status.success(), "ACL fixture must be created");
}

#[test]
fn public_directory_is_rejected_before_database_creation() {
    let files = Files::new();
    grant_everyone_access(&files.0);
    let path = files.0.join("service.db");
    assert!(matches!(
        Database::open(path.clone()),
        Err(ServiceError::Configuration)
    ));
    assert!(!path.exists());
}

#[test]
fn public_database_or_sidecar_is_rejected_without_migration() {
    for suffix in ["", "-wal", "-shm", "-journal"] {
        let files = Files::new();
        let path = files.0.join("service.db");
        let connection = rusqlite::Connection::open(&path).unwrap();
        connection
            .execute_batch("CREATE TABLE marker(value TEXT);")
            .unwrap();
        drop(connection);
        let target = files.0.join(format!("service.db{suffix}"));
        if !suffix.is_empty() {
            std::fs::write(&target, b"preserve").unwrap();
        }
        grant_everyone_access(&target);
        let before = std::fs::read(&path).unwrap();
        assert!(matches!(
            Database::open(path.clone()),
            Err(ServiceError::Configuration)
        ));
        assert_eq!(std::fs::read(path).unwrap(), before);
    }
}

#[tokio::test]
async fn hot_rollback_journal_recovers_and_normal_wal_cleanup_remains_available() {
    let files = Files::new();
    let source = files.0.join("source.db");
    let connection = rusqlite::Connection::open(&source).unwrap();
    connection.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA cache_size=2; CREATE TABLE marker(value BLOB); INSERT INTO marker VALUES(zeroblob(131072)); BEGIN IMMEDIATE; UPDATE marker SET value=randomblob(131072);").unwrap();
    let path = files.0.join("service.db");
    std::fs::copy(&source, &path).unwrap();
    std::fs::copy(
        files.0.join("source.db-journal"),
        files.0.join("service.db-journal"),
    )
    .unwrap();
    // While the source writer owns its journal, Windows may expose a zeroed
    // header to a second reader. Restore SQLite's fixed hot-journal marker on
    // the detached copy so this fixture models a crash after the journal was
    // fully written.
    {
        use std::io::{Seek, SeekFrom, Write};
        let mut journal = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(files.0.join("service.db-journal"))
            .unwrap();
        journal.seek(SeekFrom::Start(0)).unwrap();
        journal
            .write_all(&[0xd9, 0xd5, 0x05, 0xf9, 0x20, 0xa1, 0x63, 0xd7])
            .unwrap();
        journal.sync_all().unwrap();
    }
    connection.execute_batch("ROLLBACK").unwrap();
    drop(connection);
    let journal = std::fs::read(files.0.join("service.db-journal")).unwrap();
    assert_eq!(
        &journal[..8],
        &[0xd9, 0xd5, 0x05, 0xf9, 0x20, 0xa1, 0x63, 0xd7]
    );
    let database = Database::open(path.clone()).unwrap();
    database
        .execute(|connection| {
            let value: Vec<u8> =
                connection.query_row("SELECT value FROM marker", [], |row| row.get(0))?;
            assert_eq!(value, vec![0u8; 131072]);
            connection.execute_batch("INSERT INTO marker VALUES(x'01');")?;
            Ok(())
        })
        .await
        .unwrap();
    drop(database);
    assert!(!files.0.join("service.db-journal").exists());
    assert!(!files.0.join("service.db-wal").exists());
    assert!(!files.0.join("service.db-shm").exists());
    drop(Database::open(path).unwrap());
}

#[test]
fn held_database_path_rejects_parent_and_file_replacement() {
    let files = Files::new();
    let path = files.0.join("service.db");
    let database = Database::open(path.clone()).unwrap();
    assert!(std::fs::rename(&files.0, files.0.with_extension("moved")).is_err());
    let replacement_denied = std::fs::rename(&path, files.0.join("moved.db")).is_err();
    assert!(replacement_denied);
    drop(database);
    std::fs::rename(path, files.0.join("moved.db")).unwrap();
}

#[test]
fn hardlink_created_while_database_is_open_cannot_admit_another_connection() {
    let files = Files::new();
    let path = files.0.join("service.db");
    let database = Database::open(path.clone()).unwrap();
    let alias = files.0.join("alias.db");
    // Windows may deny link creation itself. When allowed, the new alias must still fail admission.
    if std::fs::hard_link(&path, &alias).is_ok() {
        assert!(matches!(
            Database::open(alias),
            Err(ServiceError::Configuration)
        ));
    }
    assert!(std::fs::remove_file(path).is_err());
    drop(database);
}

#[tokio::test]
async fn cancelled_database_caller_keeps_the_path_guard_until_its_job_exits() {
    let files = Files::new();
    let path = files.0.join("service.db");
    let database = Database::open(path.clone()).unwrap();
    let (entered, started) = tokio::sync::oneshot::channel();
    let release = Arc::new((std::sync::Mutex::new(false), std::sync::Condvar::new()));
    let worker_release = release.clone();
    let worker = database.clone();
    let caller = tokio::spawn(async move {
        worker
            .execute(move |_| {
                let _ = entered.send(());
                let (lock, ready) = &*worker_release;
                let released = lock.lock().unwrap();
                drop(ready.wait_while(released, |released| !*released).unwrap());
                Ok(())
            })
            .await
    });
    started.await.unwrap();
    let jobs = database.jobs.clone();
    drop(database);
    caller.abort();
    let _ = caller.await;
    let rename_denied_while_owner_alive = std::fs::rename(&path, files.0.join("moved.db")).is_err();
    let (lock, ready) = &*release;
    *lock.lock().unwrap() = true;
    ready.notify_all();
    jobs.shutdown(tokio::time::Instant::now() + std::time::Duration::from_secs(2))
        .await
        .unwrap();
    assert!(rename_denied_while_owner_alive);
    std::fs::rename(path, files.0.join("moved.db")).unwrap();
}
