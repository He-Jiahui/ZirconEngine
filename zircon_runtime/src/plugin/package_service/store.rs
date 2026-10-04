use super::*;
use crate::core::resource::io::transaction::{
    commit_prepared_files, recover_pending_transactions, DurableCommitDisposition,
    DurableCommitReport, JournalDocument, PreparedFileWrite, RecoveryPolicy, TransactionFault,
};
use std::path::{Component, Path};

const TAG: &str = "package";
const MAX_PACKAGES: usize = 128;
const MAX_IDENTITY_PARTITIONS: usize = 128;
const MAX_RECEIPTS: usize = 256;
const MAX_STORED_FILES: usize = 4096;
const MAX_STORED_BYTES: u64 = 256 * 1024 * 1024;

pub struct PackageStore {
    root: PathBuf,
    identity_digest: String,
    #[cfg(windows)]
    _pins: super::windows::DirectoryPins,
    #[cfg(windows)]
    _lock: std::fs::File,
}

impl PackageStore {
    pub fn open(root: &Path, identity_digest: &str) -> Result<Self> {
        Self::open_inner(root, identity_digest, true)?.ok_or(PackageError::Storage)
    }

    pub fn open_existing(root: &Path, identity_digest: &str) -> Result<Option<Self>> {
        Self::open_inner(root, identity_digest, false)
    }

    pub(super) fn open_all_existing(root: &Path) -> Result<Vec<Self>> {
        #[cfg(windows)]
        {
            let owner = super::windows::DirectoryPins::open(root, false)?;
            owner.require_private_owner()?;
            let mut identities = Vec::new();
            for entry in std::fs::read_dir(owner.path()).map_err(|_| PackageError::Storage)? {
                let entry = entry.map_err(|_| PackageError::Storage)?;
                let metadata =
                    std::fs::symlink_metadata(entry.path()).map_err(|_| PackageError::Storage)?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if !metadata.is_dir() || metadata.file_type().is_symlink() || !is_digest(&name) {
                    return Err(PackageError::Storage);
                }
                identities.push(name);
                if identities.len() > MAX_IDENTITY_PARTITIONS {
                    return Err(PackageError::Capacity);
                }
            }
            identities.sort();
            identities
                .into_iter()
                .map(|identity| {
                    Self::open_existing(owner.path(), &identity)?.ok_or(PackageError::Storage)
                })
                .collect()
        }
        #[cfg(not(windows))]
        {
            let _ = root;
            Err(PackageError::Storage)
        }
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn identity_digest(&self) -> &str {
        &self.identity_digest
    }

    pub(super) fn installed_file(
        &self,
        package: &InstalledPackage,
        member: &str,
    ) -> Result<Vec<u8>> {
        let expected = package.files.get(member).ok_or(PackageError::Storage)?;
        if package.slot != format!("slots/{}/{}", package.package_id, package.artifact_digest)
            || !valid_member(member)
            || !is_digest(expected)
        {
            return Err(PackageError::Storage);
        }
        let bytes = read_regular(
            &self.root.join(&package.slot).join(member),
            MAX_PACKAGE_BYTES,
        )?;
        if digest(&bytes) != *expected {
            return Err(PackageError::Trust);
        }
        Ok(bytes)
    }

    fn open_inner(root: &Path, identity_digest: &str, create: bool) -> Result<Option<Self>> {
        if !is_digest(identity_digest) {
            return Err(PackageError::Invalid);
        }
        #[cfg(windows)]
        {
            let owner = super::windows::DirectoryPins::open(root, false)?;
            owner.require_private_owner()?;
            if !create
                && !owner
                    .path()
                    .join(identity_digest)
                    .try_exists()
                    .map_err(|_| PackageError::Storage)?
            {
                return Ok(None);
            }
            let pins =
                super::windows::DirectoryPins::open(&owner.path().join(identity_digest), create)?;
            pins.require_private_access()?;
            let lock = super::windows::lock(&pins.path().join(".package-owner"))?;
            let store = Self {
                root: pins.path().to_owned(),
                identity_digest: identity_digest.to_owned(),
                _pins: pins,
                _lock: lock,
            };
            store.recover()?;
            Ok(Some(store))
        }
        #[cfg(not(windows))]
        {
            let _ = (root, create);
            Err(PackageError::Storage)
        }
    }

    fn recover(&self) -> Result<()> {
        let journal = self.root.join("journal");
        if !journal.try_exists().map_err(|_| PackageError::Storage)? {
            return Ok(());
        }
        #[cfg(windows)]
        let _journal = private_directory(&journal, false)?;
        let mut policy = StoreRecovery {
            root: &self.root,
            #[cfg(windows)]
            pins: std::cell::RefCell::new(Vec::new()),
        };
        recover_pending_transactions(&journal, TAG, &mut policy)
            .map_err(|_| PackageError::OutcomeUnknown)?;
        Ok(())
    }

    pub fn inventory(&self) -> Result<PackageInventory> {
        let path = self.root.join("inventory.json");
        if !path.try_exists().map_err(|_| PackageError::Storage)? {
            return Ok(PackageInventory::default());
        }
        let inventory: PackageInventory =
            serde_json::from_slice(&read_regular(&path, MAX_CONTROL_BYTES)?)
                .map_err(|_| PackageError::Storage)?;
        if inventory.schema_version != 1 || inventory.packages.len() > MAX_PACKAGES {
            return Err(PackageError::Storage);
        }
        revision(&inventory.revision).map_err(|_| PackageError::Storage)?;
        let mut ids = std::collections::HashSet::new();
        for package in &inventory.packages {
            if !ids.insert(&package.package_id) {
                return Err(PackageError::Storage);
            }
            self.verify_installed(package)?;
        }
        Ok(inventory)
    }

    pub fn receipt(&self, operation_id: &str) -> Result<Option<InstallReceipt>> {
        if !canonical_uuid(operation_id) {
            return Err(PackageError::Invalid);
        }
        let path = self
            .root
            .join("receipts")
            .join(format!("{operation_id}.json"));
        if !path.try_exists().map_err(|_| PackageError::Storage)? {
            return Ok(None);
        }
        let receipt: InstallReceipt =
            serde_json::from_slice(&read_regular(&path, MAX_CONTROL_BYTES)?)
                .map_err(|_| PackageError::Storage)?;
        if !matches!(receipt.schema_version, 1 | INSTALL_RECEIPT_SCHEMA_V2)
            || receipt.operation_id != operation_id
            || receipt.package.operation_id != operation_id
            || !is_digest(&receipt.request_digest)
            || revision(&receipt.inventory_revision)
                .ok()
                .filter(|value| *value > 0)
                .is_none()
        {
            return Err(PackageError::Storage);
        }
        match receipt.schema_version {
            1 if receipt.plugin_id.is_none() && receipt.target.is_none() => {}
            INSTALL_RECEIPT_SCHEMA_V2
                if receipt.plugin_id.as_deref().is_some_and(valid_plugin_id)
                    && receipt.target.as_ref().is_some_and(|target| {
                        matches!(
                            target.runtime_mode,
                            crate::core::framework::platform::RuntimeTargetMode::EditorHost
                                | crate::core::framework::platform::RuntimeTargetMode::ClientRuntime
                        ) && target.platform.supports_native_dynamic()
                    }) => {}
            _ => return Err(PackageError::Storage),
        }
        self.verify_installed(&receipt.package)?;
        Ok(Some(receipt))
    }

    fn verify_installed(&self, package: &InstalledPackage) -> Result<()> {
        if !canonical_uuid(&package.package_id)
            || !canonical_uuid(&package.operation_id)
            || !is_digest(&package.artifact_digest)
            || package.version.is_empty()
            || package.version.len() > 64
            || package.version.chars().any(char::is_control)
            || revision(&package.release_revision)
                .ok()
                .filter(|value| *value > 0)
                .is_none()
            || package.slot != format!("slots/{}/{}", package.package_id, package.artifact_digest)
            || package.files.is_empty()
            || package.files.len() > MAX_FILES
            || !distinct_members(package.files.keys().map(String::as_str))
        {
            return Err(PackageError::Storage);
        }
        let mut total = 0usize;
        for (member, expected) in &package.files {
            if !is_digest(expected) {
                return Err(PackageError::Storage);
            }
            let bytes = read_regular(
                &self.root.join(&package.slot).join(member),
                MAX_PACKAGE_BYTES - total,
            )?;
            total += bytes.len();
            if digest(&bytes) != *expected {
                return Err(PackageError::Storage);
            }
        }
        Ok(())
    }

    pub fn commit(&self, prepared: PreparedPackage) -> Result<InstallReceipt> {
        self.commit_with_fault(prepared, TransactionFault::None)
    }

    pub(super) fn commit_with_fault(
        &self,
        prepared: PreparedPackage,
        fault: TransactionFault,
    ) -> Result<InstallReceipt> {
        let request = &prepared.request;
        if request.identity_digest != self.identity_digest {
            return Err(PackageError::Invalid);
        }
        self.recover()?;
        let fingerprint = request.fingerprint()?;
        if let Some(receipt) = self.receipt(&request.operation_id)? {
            if receipt.request_digest != fingerprint {
                return Err(PackageError::Conflict);
            }
            return Ok(receipt);
        }
        if Utc::now() >= prepared.valid_until {
            return Err(PackageError::Trust);
        }
        let mut inventory = self.inventory()?;
        if inventory.revision != request.expected_inventory_revision {
            return Err(PackageError::Conflict);
        }
        if inventory.packages.iter().any(|item| {
            item.package_id == request.package_id
                && revision(&item.release_revision).unwrap_or(u64::MAX)
                    >= revision(&request.release_revision).unwrap_or(0)
        }) {
            return Err(PackageError::Conflict);
        }
        if inventory.packages.len() >= MAX_PACKAGES
            && !inventory
                .packages
                .iter()
                .any(|item| item.package_id == request.package_id)
        {
            return Err(PackageError::Capacity);
        }
        let slot = format!("slots/{}/{}", request.package_id, request.artifact_digest);
        let installed = InstalledPackage {
            operation_id: request.operation_id.clone(),
            package_id: request.package_id.clone(),
            version: request.version.clone(),
            release_revision: request.release_revision.clone(),
            artifact_digest: request.artifact_digest.clone(),
            slot: slot.clone(),
            files: prepared
                .files
                .iter()
                .map(|(path, bytes)| (path.clone(), digest(bytes)))
                .collect(),
        };
        let slot_path = self.root.join(&slot);
        if slot_path.try_exists().map_err(|_| PackageError::Storage)?
            && !empty_directories(&slot_path, &mut 0)?
        {
            return Err(PackageError::Conflict);
        }
        inventory
            .packages
            .retain(|item| item.package_id != request.package_id);
        inventory.packages.push(installed.clone());
        inventory
            .packages
            .sort_by(|a, b| a.package_id.cmp(&b.package_id));
        inventory.revision = revision(&inventory.revision)?
            .checked_add(1)
            .ok_or(PackageError::Capacity)?
            .to_string();
        let receipt = InstallReceipt {
            schema_version: INSTALL_RECEIPT_SCHEMA_V2,
            operation_id: request.operation_id.clone(),
            request_digest: fingerprint,
            inventory_revision: inventory.revision.clone(),
            package: installed,
            plugin_id: Some(prepared.plugin_id),
            target: Some(request.target.clone()),
        };
        let inventory_bytes = serde_json::to_vec(&inventory).map_err(|_| PackageError::Storage)?;
        let receipt_bytes = serde_json::to_vec(&receipt).map_err(|_| PackageError::Storage)?;
        if inventory_bytes.len() > MAX_CONTROL_BYTES || receipt_bytes.len() > MAX_CONTROL_BYTES {
            return Err(PackageError::Capacity);
        }
        let mut usage = StoreUsage::default();
        usage.scan(&self.root, 0)?;
        let incoming = prepared
            .files
            .values()
            .map(|bytes| bytes.len() as u64)
            .sum::<u64>()
            + inventory_bytes.len() as u64
            + receipt_bytes.len() as u64;
        if usage.receipts >= MAX_RECEIPTS
            || usage.files + prepared.files.len() + 2 > MAX_STORED_FILES
            || usage
                .bytes
                .checked_add(incoming)
                .filter(|total| *total <= MAX_STORED_BYTES)
                .is_none()
        {
            return Err(PackageError::Capacity);
        }
        let mut writes = Vec::new();
        for (member, bytes) in prepared.files {
            writes.push(PreparedFileWrite::new(slot_path.join(member), bytes));
        }
        writes.push(PreparedFileWrite::new(
            self.root.join("inventory.json"),
            inventory_bytes,
        ));
        writes.push(PreparedFileWrite::new(
            self.root
                .join("receipts")
                .join(format!("{}.json", request.operation_id)),
            receipt_bytes,
        ));
        #[cfg(windows)]
        let _parents = writes
            .iter()
            .map(|write| {
                private_directory(write.path().parent().ok_or(PackageError::Storage)?, true)
            })
            .collect::<Result<Vec<_>>>()?;
        #[cfg(windows)]
        let _journal = private_directory(&self.root.join("journal"), true)?;
        let outcome = commit_prepared_files(
            &self.root.join("journal"),
            TAG,
            writes,
            fault,
            &mut DurableCommitReport::default(),
        )
        .map_err(|_| PackageError::OutcomeUnknown)?;
        if matches!(outcome, DurableCommitDisposition::CommitRecoveryDeferred) {
            return Err(PackageError::OutcomeUnknown);
        }
        Ok(receipt)
    }
}

struct StoreRecovery<'a> {
    root: &'a Path,
    #[cfg(windows)]
    pins: std::cell::RefCell<Vec<super::windows::DirectoryPins>>,
}
impl RecoveryPolicy for StoreRecovery<'_> {
    fn validate_document(
        &self,
        _journal: &Path,
        document: &JournalDocument,
    ) -> std::result::Result<(), String> {
        let relative = document
            .target()
            .strip_prefix(self.root)
            .map_err(|_| "package transaction target is outside store")?;
        if relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
            || document.retired_paths().next().is_some()
        {
            return Err("invalid package recovery target".into());
        }
        let parts = relative
            .iter()
            .filter_map(|part| part.to_str())
            .collect::<Vec<_>>();
        let allowed = parts == ["inventory.json"]
            || (parts.len() == 2
                && parts[0] == "receipts"
                && parts[1].strip_suffix(".json").is_some_and(canonical_uuid))
            || (parts.len() >= 4
                && parts[0] == "slots"
                && canonical_uuid(parts[1])
                && is_digest(parts[2])
                && valid_member(&parts[3..].join("/")));
        if !allowed {
            return Err("unexpected package recovery target".into());
        }
        #[cfg(windows)]
        self.pins.borrow_mut().push(
            private_directory(
                document.target().parent().ok_or("missing package parent")?,
                false,
            )
            .map_err(|_| "package parent unavailable")?,
        );
        Ok(())
    }

    fn digest_file(&mut self, path: &Path) -> std::io::Result<String> {
        read_regular(path, MAX_PACKAGE_BYTES)
            .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
            .map_err(std::io::Error::other)
    }
}

#[derive(Default)]
struct StoreUsage {
    bytes: u64,
    files: usize,
    receipts: usize,
}

impl StoreUsage {
    fn scan(&mut self, path: &Path, depth: usize) -> Result<()> {
        if depth > 32 {
            return Err(PackageError::Capacity);
        }
        #[cfg(windows)]
        let _pins = private_directory(path, false)?;
        for entry in std::fs::read_dir(path).map_err(|_| PackageError::Storage)? {
            self.files += 1;
            if self.files > MAX_STORED_FILES {
                return Err(PackageError::Capacity);
            }
            let entry = entry.map_err(|_| PackageError::Storage)?;
            let metadata =
                std::fs::symlink_metadata(entry.path()).map_err(|_| PackageError::Storage)?;
            if metadata.file_type().is_symlink() {
                return Err(PackageError::Storage);
            }
            if metadata.is_dir() {
                self.scan(&entry.path(), depth + 1)?;
            } else if metadata.is_file() {
                if depth == 1 && path.file_name().is_some_and(|name| name == "receipts") {
                    self.receipts += 1;
                }
                self.bytes = self
                    .bytes
                    .checked_add(metadata.len())
                    .filter(|total| *total <= MAX_STORED_BYTES)
                    .ok_or(PackageError::Capacity)?;
            } else {
                return Err(PackageError::Storage);
            }
        }
        Ok(())
    }
}

#[cfg(windows)]
fn private_directory(path: &Path, create: bool) -> Result<super::windows::DirectoryPins> {
    let pins = super::windows::DirectoryPins::open(path, create)?;
    pins.require_private_access()?;
    Ok(pins)
}

fn empty_directories(path: &Path, visited: &mut usize) -> Result<bool> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| PackageError::Storage)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(PackageError::Storage);
    }
    #[cfg(windows)]
    let _guard = super::windows::DirectoryPins::open(path, false)?;
    for entry in std::fs::read_dir(path).map_err(|_| PackageError::Storage)? {
        *visited += 1;
        if *visited > MAX_FILES {
            return Err(PackageError::Capacity);
        }
        let entry = entry.map_err(|_| PackageError::Storage)?;
        if !entry
            .file_type()
            .map_err(|_| PackageError::Storage)?
            .is_dir()
            || !empty_directories(&entry.path(), visited)?
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn read_regular(path: &Path, limit: usize) -> Result<Vec<u8>> {
    #[cfg(windows)]
    {
        super::windows::read_regular(path, limit)
    }
    #[cfg(not(windows))]
    {
        let _ = (path, limit);
        Err(PackageError::Storage)
    }
}
