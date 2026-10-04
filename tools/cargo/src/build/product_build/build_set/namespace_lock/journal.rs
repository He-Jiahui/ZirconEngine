use std::cell::{Cell, RefCell};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem::size_of;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;

use serde::{Deserialize, Serialize};
use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
use windows_sys::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorW,
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SetSecurityInfo, SDDL_REVISION_1,
    SE_FILE_OBJECT,
};
use windows_sys::Win32::Security::{
    GetSecurityDescriptorControl, GetSecurityDescriptorDacl, ACL, DACL_SECURITY_INFORMATION,
    PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SE_DACL_PROTECTED,
    UNPROTECTED_DACL_SECURITY_INFORMATION,
};
use windows_sys::Win32::Storage::FileSystem::{
    FileDispositionInfo, FileIdInfo, GetFileInformationByHandleEx, SetFileInformationByHandle,
    DELETE, FILE_DISPOSITION_INFO, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_ID_INFO, FILE_SHARE_READ, FILE_SHARE_WRITE, WRITE_DAC,
};

use super::{status, LocalAllocation};

const JOURNAL_VERSION: u32 = 1;
const JOURNAL_BYTE_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectoryIdentity {
    volume: u64,
    file_id: [u8; 16],
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RecoveryRecord {
    version: u32,
    relative_path: String,
    identity: DirectoryIdentity,
    original_sddl: String,
}

pub(in crate::build::product_build::build_set) struct NamespaceJournal {
    file: RefCell<File>,
    root: PathBuf,
    _root_lease: File,
    recovery_required: Cell<bool>,
}

impl NamespaceJournal {
    pub(in crate::build::product_build::build_set) fn acquire(root: &Path) -> io::Result<Self> {
        let root_lease = open_directory(root, GENERIC_READ, FILE_SHARE_READ | FILE_SHARE_WRITE)?;
        let identity = directory_identity(&root_lease)?;
        let id = identity
            .file_id
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let parent = root
            .parent()
            .ok_or_else(|| io::Error::other("BuildSet root has no parent"))?;
        let path = parent.join(format!(
            ".build-set-namespace-{:016x}-{id}.jsonl",
            identity.volume
        ));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .access_mode(GENERIC_READ | GENERIC_WRITE | DELETE)
            .share_mode(0)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)?;
        let metadata = file.metadata()?;
        if super::super::is_reparse_or_symlink(&metadata) || !metadata.is_file() {
            return Err(io::Error::other(
                "BuildSet recovery journal must be a non-reparse file",
            ));
        }
        let journal = Self {
            file: RefCell::new(file),
            root: root.to_path_buf(),
            _root_lease: root_lease,
            recovery_required: Cell::new(true),
        };
        journal.recover()?;
        journal.recovery_required.set(false);
        Ok(journal)
    }

    pub(super) fn record(
        &self,
        path: &Path,
        directory: &File,
        descriptor: PSECURITY_DESCRIPTOR,
    ) -> io::Result<()> {
        let relative = path.strip_prefix(&self.root).map_err(|_| {
            io::Error::other("BuildSet recovery directory escaped its snapshot root")
        })?;
        let relative = relative
            .to_str()
            .ok_or_else(|| io::Error::other("BuildSet recovery directory path is not Unicode"))?
            .replace('\\', "/");
        let record = RecoveryRecord {
            version: JOURNAL_VERSION,
            relative_path: relative,
            identity: directory_identity(directory)?,
            original_sddl: descriptor_sddl(descriptor)?,
        };
        let mut bytes = serde_json::to_vec(&record).map_err(io::Error::other)?;
        bytes.push(b'\n');
        let mut file = self.file.borrow_mut();
        if file.metadata()?.len().saturating_add(bytes.len() as u64) > JOURNAL_BYTE_LIMIT {
            return Err(io::Error::other(
                "BuildSet recovery journal exceeded its byte limit",
            ));
        }
        // A complete, durable record must precede any persistent ACL mutation.
        let written = file.write_all(&bytes).and_then(|_| file.sync_all());
        if written.is_err() {
            self.recovery_required.set(true);
        }
        written
    }

    pub(super) fn retain_for_recovery(&self) {
        self.recovery_required.set(true);
    }

    fn recover(&self) -> io::Result<()> {
        let mut file = self.file.borrow_mut();
        if file.metadata()?.len() > JOURNAL_BYTE_LIMIT {
            return Err(io::Error::other(
                "BuildSet recovery journal exceeded its byte limit",
            ));
        }
        let mut bytes = Vec::new();
        Read::by_ref(&mut *file)
            .take(JOURNAL_BYTE_LIMIT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > JOURNAL_BYTE_LIMIT {
            return Err(io::Error::other(
                "BuildSet recovery journal grew beyond its byte limit",
            ));
        }
        let mut records = Vec::new();
        for line in bytes.split_inclusive(|byte| *byte == b'\n') {
            // An interrupted append cannot have applied an ACL: record() had not synced.
            if line.last() != Some(&b'\n') {
                break;
            }
            let record: RecoveryRecord = serde_json::from_slice(line).map_err(io::Error::other)?;
            if record.version != JOURNAL_VERSION {
                return Err(io::Error::other(
                    "Unsupported BuildSet recovery journal version",
                ));
            }
            if !record.relative_path.is_empty() {
                super::super::validate_relative_path(&record.relative_path)
                    .map_err(io::Error::other)?;
            }
            records.push(record);
        }
        // Validate every identity and DACL before the first persistent permission change.
        let mut opened = Vec::with_capacity(records.len());
        for record in &records {
            let directory = open_directory(
                &self.root.join(&record.relative_path),
                GENERIC_READ | WRITE_DAC,
                FILE_SHARE_READ,
            )?;
            if directory_identity(&directory)? != record.identity {
                return Err(io::Error::other(
                    "BuildSet recovery directory identity changed",
                ));
            }
            opened.push((directory, parse_recovery_dacl(&record.original_sddl)?));
        }
        for (directory, dacl) in opened.iter().rev() {
            dacl.restore(directory)?;
        }
        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.sync_all()?;
        Ok(())
    }
}

impl Drop for NamespaceJournal {
    fn drop(&mut self) {
        if self.recovery_required.get() {
            return;
        }
        let info = FILE_DISPOSITION_INFO { DeleteFile: true };
        // Delete on this exclusive handle, without a close/remove/reopen race.
        if unsafe {
            SetFileInformationByHandle(
                self.file.get_mut().as_raw_handle(),
                FileDispositionInfo,
                (&info as *const FILE_DISPOSITION_INFO).cast(),
                size_of::<FILE_DISPOSITION_INFO>() as u32,
            )
        } == 0
        {
            eprintln!(
                "could not retire BuildSet recovery journal: {}",
                io::Error::last_os_error()
            );
        }
    }
}

fn directory_identity(file: &File) -> io::Result<DirectoryIdentity> {
    let mut info = FILE_ID_INFO::default();
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle(),
            FileIdInfo,
            (&mut info as *mut FILE_ID_INFO).cast(),
            size_of::<FILE_ID_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(DirectoryIdentity {
        volume: info.VolumeSerialNumber,
        file_id: info.FileId.Identifier,
    })
}

fn open_directory(path: &Path, access: u32, share: u32) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .access_mode(access)
        .share_mode(share)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let metadata = file.metadata()?;
    if super::super::is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
        return Err(io::Error::other(
            "BuildSet recovery requires a non-reparse directory",
        ));
    }
    Ok(file)
}

pub(super) fn descriptor_sddl(descriptor: PSECURITY_DESCRIPTOR) -> io::Result<String> {
    let mut text = null_mut();
    let mut length = 0;
    if unsafe {
        ConvertSecurityDescriptorToStringSecurityDescriptorW(
            descriptor,
            SDDL_REVISION_1,
            DACL_SECURITY_INFORMATION,
            &mut text,
            &mut length,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let _allocation = LocalAllocation::new(text.cast())?;
    // Windows reports buffer capacity; only the prefix before the first NUL is SDDL.
    for used in 0..length as usize {
        if unsafe { *text.add(used) } == 0 {
            let text = unsafe { std::slice::from_raw_parts(text, used) };
            return String::from_utf16(text).map_err(io::Error::other);
        }
    }
    Err(io::Error::other(
        "Windows returned an unterminated security descriptor string",
    ))
}

struct RecoveryDacl {
    _descriptor: LocalAllocation,
    acl: *mut ACL,
    protection: u32,
}

impl RecoveryDacl {
    fn restore(&self, directory: &File) -> io::Result<()> {
        status(unsafe {
            SetSecurityInfo(
                directory.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | self.protection,
                null_mut(),
                null_mut(),
                self.acl,
                null_mut(),
            )
        })
    }
}

#[cfg(test)]
pub(super) fn restore_sddl(directory: &File, sddl: &str) -> io::Result<()> {
    parse_recovery_dacl(sddl)?.restore(directory)
}

fn parse_recovery_dacl(sddl: &str) -> io::Result<RecoveryDacl> {
    if sddl.contains('\0') {
        return Err(io::Error::other(
            "BuildSet recovery SDDL contains a null character",
        ));
    }
    let text = sddl.encode_utf16().chain([0]).collect::<Vec<_>>();
    let mut descriptor = null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let allocation = LocalAllocation::new(descriptor)?;
    let mut acl = null_mut();
    let mut present = 0;
    let mut defaulted = 0;
    let mut control = 0;
    let mut revision = 0;
    if unsafe { GetSecurityDescriptorDacl(descriptor, &mut present, &mut acl, &mut defaulted) } == 0
        || unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if present == 0 || acl.is_null() {
        return Err(io::Error::other(
            "BuildSet recovery record has a missing or null DACL",
        ));
    }
    let protection = if control & SE_DACL_PROTECTED != 0 {
        PROTECTED_DACL_SECURITY_INFORMATION
    } else {
        UNPROTECTED_DACL_SECURITY_INFORMATION
    };
    Ok(RecoveryDacl {
        _descriptor: allocation,
        acl,
        protection,
    })
}

#[cfg(test)]
#[path = "tests/journal.rs"]
mod tests;
