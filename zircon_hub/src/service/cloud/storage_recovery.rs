use super::{
    config::reject_link,
    manifest, quota,
    store::{acquire_owner, ENCRYPTION_OVERHEAD},
    BlobStore,
};
use crate::service::{config::read_bounded_regular, error::ServiceError};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
};

const STORE_ID_FILE: &str = ".store-id";

impl BlobStore {
    pub fn recover(&self, connection: &mut Connection) -> Result<(), ServiceError> {
        let mut database_owner = self
            .database_owner
            .lock()
            .map_err(|_| ServiceError::Storage)?;
        if database_owner.is_some() {
            return Err(ServiceError::Storage);
        }
        let owner = acquire_database_owner(connection)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut physical = self.physical.lock().map_err(|_| ServiceError::Storage)?;
        if physical.is_some() {
            return Err(ServiceError::Storage);
        }
        let binding: Option<(String, Vec<u8>)> = transaction
            .query_row(
                "SELECT store_id,key_fingerprint FROM cloud_store_binding WHERE id=1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let stored_id = read_store_id(&self.root)?;
        let mut binding_id = binding.as_ref().map(|(id, _)| id.clone());
        if let Some((id, key)) = &binding {
            if stored_id.as_ref() != Some(id) || key.as_slice() != self.key_fingerprint.as_slice() {
                return Err(ServiceError::Configuration);
            }
        }
        let mut bytes = 0u64;
        let mut objects = 0u64;
        for entry in fs::read_dir(&self.root).map_err(|_| ServiceError::Storage)? {
            let entry = entry.map_err(|_| ServiceError::Storage)?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| ServiceError::Configuration)?;
            reject_link(&entry.path())?;
            if name == ".store.lock" || name == STORE_ID_FILE {
                continue;
            }
            objects = objects.checked_add(1).ok_or(ServiceError::Capacity)?;
            if objects > quota::MAX_GLOBAL_OBJECTS {
                return Err(ServiceError::Capacity);
            }
            if name.strip_suffix(".partial").is_some_and(|id| {
                uuid::Uuid::parse_str(id).is_ok_and(|value| value.to_string() == id)
            }) {
                fs::remove_file(entry.path()).map_err(|_| ServiceError::Storage)?;
                objects -= 1;
                continue;
            }
            manifest::digest(&name).map_err(|_| ServiceError::Configuration)?;
            let metadata = entry.metadata().map_err(|_| ServiceError::Storage)?;
            if !metadata.is_file()
                || metadata.len() < ENCRYPTION_OVERHEAD
                || metadata.len() > manifest::MAX_BLOB_BYTES as u64 + ENCRYPTION_OVERHEAD
            {
                return Err(ServiceError::Storage);
            }
            bytes = bytes
                .checked_add(metadata.len())
                .ok_or(ServiceError::Capacity)?;
            if bytes > quota::MAX_GLOBAL_BYTES {
                return Err(ServiceError::Capacity);
            }
            // Validate every ciphertext, including unpublished blobs, before exposing the store.
            self.read(&name)?;
            // Keep unpublished blobs accounted. An external database restore must not erase them.
        }
        let mut statement = transaction.prepare("SELECT digest,bytes FROM cloud_blobs")?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let digest: String = row.get(0)?;
            let size: u64 = row.get(1)?;
            if self.read(&digest)?.len() as u64 != size {
                return Err(ServiceError::Storage);
            }
        }
        drop(rows);
        drop(statement);
        if binding.is_none() {
            let id = match stored_id {
                Some(id) => id,
                None => publish_store_id(&self.root)?,
            };
            transaction.execute(
                "INSERT INTO cloud_store_binding VALUES (1,?1,?2)",
                params![id, self.key_fingerprint.as_slice()],
            )?;
            binding_id = Some(id);
        }
        transaction.commit()?;
        if read_store_id(&self.root)? != binding_id {
            return Err(ServiceError::Configuration);
        }
        *database_owner = Some(owner);
        *physical = Some((bytes, objects));
        drop(database_owner);
        drop(physical);
        super::retention::gc::recover(connection, self)
    }
}

fn acquire_database_owner(connection: &Connection) -> Result<File, ServiceError> {
    // Use SQLite's actual main database path, not a separately supplied configuration string.
    let reported: String = connection.query_row(
        "SELECT file FROM pragma_database_list WHERE name='main'",
        [],
        |row| row.get(0),
    )?;
    let path = Path::new(&reported);
    if !path.is_absolute() {
        return Err(ServiceError::Configuration);
    }
    for ancestor in path.ancestors() {
        reject_link(ancestor)?;
    }
    let canonical = fs::canonicalize(path).map_err(|_| ServiceError::Configuration)?;
    let metadata = fs::metadata(&canonical).map_err(|_| ServiceError::Configuration)?;
    if !metadata.is_file() {
        return Err(ServiceError::Configuration);
    }
    let parent = canonical.parent().ok_or(ServiceError::Configuration)?;
    let mut lock_name = canonical
        .file_name()
        .ok_or(ServiceError::Configuration)?
        .to_os_string();
    lock_name.push(".cloud-owner.lock");
    // A cloned CAS preserves its store ID, but still competes for this database-wide owner.
    acquire_owner(&parent.join(lock_name))
}

fn read_store_id(root: &Path) -> Result<Option<String>, ServiceError> {
    let path = root.join(STORE_ID_FILE);
    match fs::symlink_metadata(&path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(ServiceError::Storage),
    }
    let bytes = read_bounded_regular(&path, 36)?;
    let id = String::from_utf8(bytes).map_err(|_| ServiceError::Configuration)?;
    let parsed = uuid::Uuid::parse_str(&id).map_err(|_| ServiceError::Configuration)?;
    if parsed.to_string() != id {
        return Err(ServiceError::Configuration);
    }
    Ok(Some(id))
}

fn publish_store_id(root: &Path) -> Result<String, ServiceError> {
    let id = uuid::Uuid::new_v4().to_string();
    let temporary = root.join(format!("{}.partial", uuid::Uuid::new_v4()));
    let outcome = (|| {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .map_err(|_| ServiceError::Storage)?;
        file.write_all(id.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|_| ServiceError::Storage)?;
        drop(file);
        if read_bounded_regular(&temporary, 36)? != id.as_bytes() {
            return Err(ServiceError::Storage);
        }
        fs::rename(&temporary, root.join(STORE_ID_FILE)).map_err(|_| ServiceError::Storage)?;
        #[cfg(unix)]
        File::open(root)
            .and_then(|file| file.sync_all())
            .map_err(|_| ServiceError::Storage)?;
        if read_store_id(root)?.as_deref() != Some(id.as_str()) {
            return Err(ServiceError::Configuration);
        }
        Ok(id)
    })();
    if outcome.is_err() {
        let _ = fs::remove_file(temporary);
    }
    outcome
}
