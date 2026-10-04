use super::{authorization::authorize, config::reject_link, manifest, quota, CloudConfig};
use crate::service::{
    config::read_bounded_regular,
    error::ServiceError,
    identity::{now_seconds, Principal},
    organization,
};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    Aes256Gcm, Nonce,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub(super) const ENCRYPTION_OVERHEAD: u64 = 28;

pub struct BlobStore {
    pub(super) root: PathBuf,
    cipher: Aes256Gcm,
    _owner: File,
    pub(super) key_fingerprint: [u8; 32],
    pub(super) database_owner: Mutex<Option<File>>,
    pub(super) physical: Mutex<Option<(u64, u64)>>,
}

impl BlobStore {
    pub fn load(config: &CloudConfig) -> Result<Self, ServiceError> {
        config.validate()?;
        for ancestor in config.key_file.ancestors() {
            reject_link(ancestor)?;
        }
        let key = read_bounded_regular(&config.key_file, 32)?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| ServiceError::Configuration)?;
        let key_fingerprint = Sha256::digest(&key).into();
        for ancestor in config.root.ancestors() {
            match fs::symlink_metadata(ancestor) {
                Ok(_) => reject_link(ancestor)?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(ServiceError::Storage),
            }
        }
        fs::create_dir_all(&config.root).map_err(|_| ServiceError::Storage)?;
        for ancestor in config.root.ancestors() {
            reject_link(ancestor)?;
        }
        let root = fs::canonicalize(&config.root).map_err(|_| ServiceError::Configuration)?;
        let key_file =
            fs::canonicalize(&config.key_file).map_err(|_| ServiceError::Configuration)?;
        if key_file.starts_with(&root) {
            return Err(ServiceError::Configuration);
        }
        let owner = acquire_owner(&root.join(".store.lock"))?;
        Ok(Self {
            root,
            cipher,
            _owner: owner,
            key_fingerprint,
            database_owner: Mutex::new(None),
            physical: Mutex::new(None),
        })
    }

    pub(super) fn read(&self, digest: &str) -> Result<Vec<u8>, ServiceError> {
        manifest::digest(digest)?;
        self.read_file(&self.root.join(digest), digest)
    }

    fn read_file(&self, path: &Path, digest: &str) -> Result<Vec<u8>, ServiceError> {
        let bytes = read_bounded_regular(
            path,
            manifest::MAX_BLOB_BYTES + ENCRYPTION_OVERHEAD as usize,
        )
        .map_err(|_| ServiceError::Storage)?;
        if bytes.len() < ENCRYPTION_OVERHEAD as usize {
            return Err(ServiceError::Storage);
        }
        let plaintext = self
            .cipher
            .decrypt(
                Nonce::from_slice(&bytes[..12]),
                Payload {
                    msg: &bytes[12..],
                    aad: digest.as_bytes(),
                },
            )
            .map_err(|_| ServiceError::Storage)?;
        if format!("{:x}", Sha256::digest(&plaintext)) != digest {
            return Err(ServiceError::Storage);
        }
        Ok(plaintext)
    }

    fn write(&self, digest: &str, plaintext: &[u8]) -> Result<(), ServiceError> {
        let mut physical = self.physical.lock().map_err(|_| ServiceError::Storage)?;
        let (used, objects) = physical.as_mut().ok_or(ServiceError::Storage)?;
        let target = self.root.join(digest);
        if target.exists() {
            self.read(digest)?;
            return Ok(());
        }
        let stored_bytes = plaintext.len() as u64 + ENCRYPTION_OVERHEAD;
        if used
            .checked_add(stored_bytes)
            .is_none_or(|value| value > quota::MAX_GLOBAL_BYTES)
            || *objects >= quota::MAX_GLOBAL_OBJECTS
        {
            return Err(ServiceError::Capacity);
        }
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: digest.as_bytes(),
                },
            )
            .map_err(|_| ServiceError::Storage)?;
        let temporary = self.root.join(format!("{}.partial", uuid::Uuid::new_v4()));
        // Reserve even failed staging bytes until recovery proves they are absent.
        *used += stored_bytes;
        *objects += 1;
        let outcome = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary)
                .map_err(|_| ServiceError::Storage)?;
            file.write_all(&nonce)
                .and_then(|_| file.write_all(&ciphertext))
                .and_then(|_| file.sync_all())
                .map_err(|_| ServiceError::Storage)?;
            drop(file);
            self.read_file(&temporary, digest)?;
            fs::rename(&temporary, &target).map_err(|_| ServiceError::Storage)?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&target)
                .and_then(|file| file.sync_all())
                .map_err(|_| ServiceError::Storage)?;
            #[cfg(unix)]
            File::open(&self.root)
                .and_then(|file| file.sync_all())
                .map_err(|_| ServiceError::Storage)?;
            Ok(())
        })();
        if outcome.is_err() && fs::remove_file(&temporary).is_ok() {
            *used -= stored_bytes;
            *objects -= 1;
        }
        outcome
    }

    pub(super) fn remove_unreferenced(&self, digest: &str, bytes: u64) -> Result<(), ServiceError> {
        manifest::digest(digest)?;
        let mut physical = self.physical.lock().map_err(|_| ServiceError::Storage)?;
        let (used, objects) = physical.as_mut().ok_or(ServiceError::Storage)?;
        let target = self.root.join(digest);
        match fs::symlink_metadata(&target) {
            Ok(metadata) => {
                reject_link(&target)?;
                if !metadata.is_file()
                    || metadata.len() != bytes + ENCRYPTION_OVERHEAD
                    || self.read(digest)?.len() as u64 != bytes
                {
                    return Err(ServiceError::Storage);
                }
                let remaining_bytes = used
                    .checked_sub(metadata.len())
                    .ok_or(ServiceError::Storage)?;
                let remaining_objects = objects.checked_sub(1).ok_or(ServiceError::Storage)?;
                fs::remove_file(&target).map_err(|_| ServiceError::Storage)?;
                *used = remaining_bytes;
                *objects = remaining_objects;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(ServiceError::Storage),
        }
        #[cfg(unix)]
        File::open(&self.root)
            .and_then(|file| file.sync_all())
            .map_err(|_| ServiceError::Storage)?;
        Ok(())
    }
}

pub(super) fn acquire_owner(path: &Path) -> Result<File, ServiceError> {
    match fs::symlink_metadata(path) {
        Ok(_) => reject_link(path)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(ServiceError::Storage),
    }
    let mut options = OpenOptions::new();
    options.create(true).truncate(false).read(true).write(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_SHARE_READ_WRITE: u32 = 0x0000_0003;
        options
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .share_mode(FILE_SHARE_READ_WRITE);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let owner = options.open(path).map_err(|_| ServiceError::Storage)?;
    let metadata = owner.metadata().map_err(|_| ServiceError::Storage)?;
    if !metadata.is_file() {
        return Err(ServiceError::Configuration);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(ServiceError::Configuration);
        }
    }
    // These locks own runtime data and are released by the OS after process exit.
    File::try_lock(&owner).map_err(|_| ServiceError::Capacity)?;
    Ok(owner)
}

pub fn upload(
    connection: &mut Connection,
    principal: &Principal,
    store: &BlobStore,
    organization: &str,
    project: &str,
    digest: &str,
    bytes: &[u8],
) -> Result<(), ServiceError> {
    manifest::digest(digest)?;
    if bytes.len() > manifest::MAX_BLOB_BYTES || format!("{:x}", Sha256::digest(bytes)) != digest {
        return Err(ServiceError::InvalidRequest);
    }
    super::retention::collect_before_write(connection, principal, store, organization, project)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let revision = authorize(&transaction, principal, organization, project, true)?;
    let existing: Option<u64> = transaction.query_row("SELECT bytes FROM cloud_blobs WHERE organization_id=?1 AND project_id=?2 AND digest=?3", params![organization, project, digest], |row| row.get(0)).optional()?;
    if let Some(size) = existing {
        if size != bytes.len() as u64 {
            return Err(ServiceError::Storage);
        }
    } else {
        let (used, count): (u64, u64) = transaction.query_row("SELECT COALESCE(SUM(bytes),0),COUNT(*) FROM cloud_blobs WHERE organization_id=?1 AND project_id=?2", params![organization, project], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let globally_existing: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM cloud_blobs WHERE digest=?1)",
            [digest],
            |row| row.get(0),
        )?;
        let global_over_limit = if globally_existing {
            false
        } else {
            let (global, global_count) = global_unique_usage(&transaction)?;
            global
                .checked_add(bytes.len() as u64)
                .is_none_or(|value| value > quota::MAX_GLOBAL_BYTES)
                || global_count >= quota::MAX_GLOBAL_OBJECTS
        };
        if used
            .checked_add(bytes.len() as u64)
            .is_none_or(|value| value > manifest::MAX_PROJECT_BYTES)
            || global_over_limit
            || count >= quota::MAX_PROJECT_OBJECTS
        {
            return Err(ServiceError::Capacity);
        }
    }
    store.write(digest, bytes)?;
    if existing.is_none() {
        transaction.execute(
            "INSERT INTO cloud_blobs VALUES (?1,?2,?3,?4)",
            params![organization, project, digest, bytes.len()],
        )?;
        organization::audit(
            &transaction,
            organization,
            principal,
            "cloud.upload",
            "allowed",
            revision,
        )?;
    }
    transaction.execute(
        "INSERT INTO cloud_upload_leases VALUES (?1,?2,?3,?4)
         ON CONFLICT(organization_id,project_id,digest) DO UPDATE SET expires_at=excluded.expires_at",
        params![organization, project, digest, now_seconds() + super::retention::UPLOAD_LEASE_SECONDS],
    )?;
    transaction.execute("DELETE FROM cloud_gc_pending WHERE digest=?1", [digest])?;
    transaction.commit()?;
    Ok(())
}

fn global_unique_usage(connection: &Connection) -> Result<(u64, u64), ServiceError> {
    // A digest names one physical CAS object even when several tenants retain
    // rows for it. Count each digest once; MAX keeps malformed cross-tenant
    // metadata from understating admission until recovery can reject it.
    Ok(connection.query_row(
        "SELECT COALESCE(SUM(bytes),0),COUNT(*)
         FROM (SELECT digest,MAX(bytes) AS bytes FROM cloud_blobs GROUP BY digest)",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?)
}

pub fn read_blob(
    connection: &mut Connection,
    principal: &Principal,
    store: &BlobStore,
    organization: &str,
    project: &str,
    digest: &str,
) -> Result<Vec<u8>, ServiceError> {
    manifest::digest(digest)?;
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let revision = authorize(&transaction, principal, organization, project, false)?;
    let referenced: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM snapshot_blobs WHERE organization_id=?1 AND project_id=?2 AND digest=?3)", params![organization, project, digest], |row| row.get(0))?;
    if !referenced {
        return Err(ServiceError::Forbidden);
    }
    let bytes = store.read(digest)?;
    organization::audit(
        &transaction,
        organization,
        principal,
        "cloud.download",
        "allowed",
        revision,
    )?;
    transaction.commit()?;
    Ok(bytes)
}
