use std::ffi::c_void;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::mem::{size_of, size_of_val};
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::ptr::{null_mut, NonNull};
use std::rc::Rc;

mod journal;
pub(super) use journal::NamespaceJournal;

use windows_sys::Win32::Foundation::{LocalFree, GENERIC_READ};
use windows_sys::Win32::Security::Authorization::{
    GetSecurityInfo, SetEntriesInAclW, SetSecurityInfo, DENY_ACCESS, EXPLICIT_ACCESS_W,
    NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_W,
};
use windows_sys::Win32::Security::{
    CreateWellKnownSid, GetSecurityDescriptorControl, WinWorldSid, ACL, DACL_SECURITY_INFORMATION,
    NO_INHERITANCE, PROTECTED_DACL_SECURITY_INFORMATION, SECURITY_MAX_SID_SIZE, SE_DACL_PROTECTED,
    UNPROTECTED_DACL_SECURITY_INFORMATION,
};
use windows_sys::Win32::Storage::FileSystem::{
    DELETE, FILE_ADD_FILE, FILE_ADD_SUBDIRECTORY, FILE_DELETE_CHILD, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ, FILE_WRITE_ATTRIBUTES, FILE_WRITE_EA, WRITE_DAC,
};

const NAMESPACE_MUTATION_ACCESS: u32 = FILE_ADD_FILE
    | FILE_ADD_SUBDIRECTORY
    | FILE_DELETE_CHILD
    | FILE_WRITE_ATTRIBUTES
    | FILE_WRITE_EA
    | DELETE;

pub(super) struct DirectoryLease {
    directory: File,
    _original_descriptor: LocalAllocation,
    original_acl: *mut ACL,
    original_protection: u32,
    frozen: bool,
    journal: Rc<NamespaceJournal>,
}

impl DirectoryLease {
    pub(super) fn open(
        path: &Path,
        journal: Rc<NamespaceJournal>,
    ) -> io::Result<(Self, fs::Metadata)> {
        // The write access also makes another namespace guard fail its share check.
        let directory = OpenOptions::new()
            .read(true)
            .access_mode(GENERIC_READ | FILE_ADD_FILE | WRITE_DAC)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        let metadata = directory.metadata()?;
        if super::is_reparse_or_symlink(&metadata) || !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "BuildSet directory lease requires a non-reparse directory",
            ));
        }

        let mut descriptor = null_mut();
        let mut original_acl = null_mut();
        // The returned ACL points into the descriptor, which remains owned by this lease.
        status(unsafe {
            GetSecurityInfo(
                directory.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                &mut original_acl,
                null_mut(),
                &mut descriptor,
            )
        })?;
        let original_descriptor = LocalAllocation::new(descriptor)?;
        if original_acl.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "BuildSet namespace protection does not support null DACL directories",
            ));
        }
        let mut control = 0;
        let mut revision = 0;
        if unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let original_protection = if control & SE_DACL_PROTECTED != 0 {
            PROTECTED_DACL_SECURITY_INFORMATION
        } else {
            UNPROTECTED_DACL_SECURITY_INFORMATION
        };
        let mut lease = Self {
            directory,
            _original_descriptor: original_descriptor,
            original_acl,
            original_protection,
            frozen: false,
            journal,
        };
        lease.journal.record(path, &lease.directory, descriptor)?;
        lease.freeze()?;
        Ok((lease, metadata))
    }

    fn freeze(&mut self) -> io::Result<()> {
        let mut sid = [0_u32; SECURITY_MAX_SID_SIZE as usize / size_of::<u32>()];
        let mut sid_bytes = size_of_val(&sid) as u32;
        if unsafe {
            CreateWellKnownSid(
                WinWorldSid,
                null_mut(),
                sid.as_mut_ptr().cast(),
                &mut sid_bytes,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let deny = EXPLICIT_ACCESS_W {
            grfAccessPermissions: NAMESPACE_MUTATION_ACCESS,
            grfAccessMode: DENY_ACCESS,
            grfInheritance: NO_INHERITANCE,
            Trustee: TRUSTEE_W {
                pMultipleTrustee: null_mut(),
                MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
                TrusteeForm: TRUSTEE_IS_SID,
                TrusteeType: TRUSTEE_IS_WELL_KNOWN_GROUP,
                ptstrName: sid.as_mut_ptr().cast(),
            },
        };
        let mut frozen_acl = null_mut();
        status(unsafe { SetEntriesInAclW(1, &deny, self.original_acl, &mut frozen_acl) })?;
        let _frozen_allocation = LocalAllocation::new(frozen_acl.cast())?;
        self.frozen = true;
        self.set_acl(frozen_acl, PROTECTED_DACL_SECURITY_INFORMATION)?;
        Ok(())
    }

    pub(super) fn restore(&mut self) -> io::Result<()> {
        if self.frozen {
            if let Err(error) = self.set_acl(self.original_acl, self.original_protection) {
                self.journal.retain_for_recovery();
                return Err(error);
            }
            self.frozen = false;
        }
        Ok(())
    }

    fn set_acl(&self, acl: *mut ACL, protection: u32) -> io::Result<()> {
        // Restore through the same open directory identity, never through a mutable path.
        status(unsafe {
            SetSecurityInfo(
                self.directory.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | protection,
                null_mut(),
                null_mut(),
                acl,
                null_mut(),
            )
        })
    }
}

impl Drop for DirectoryLease {
    fn drop(&mut self) {
        if let Err(error) = self.restore() {
            eprintln!("could not restore BuildSet directory access: {error}");
        }
    }
}

struct LocalAllocation(NonNull<c_void>);

impl LocalAllocation {
    fn new(pointer: *mut c_void) -> io::Result<Self> {
        NonNull::new(pointer).map(Self).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Windows returned no security descriptor",
            )
        })
    }
}

impl Drop for LocalAllocation {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0.as_ptr()) };
    }
}

fn status(code: u32) -> io::Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(code as i32))
    }
}

#[cfg(test)]
pub(super) fn directory_sddl(path: &Path) -> io::Result<String> {
    let descriptor = directory_descriptor(path)?;
    journal::descriptor_sddl(descriptor.0.as_ptr())
}

#[cfg(test)]
pub(super) fn directory_permissions(path: &Path) -> io::Result<(Vec<Vec<u8>>, bool)> {
    use windows_sys::Win32::Security::{GetAce, GetSecurityDescriptorDacl, ACE_HEADER};

    let descriptor = directory_descriptor(path)?;
    let mut acl = null_mut();
    let mut present = 0;
    let mut defaulted = 0;
    let mut control = 0;
    let mut revision = 0;
    if unsafe {
        GetSecurityDescriptorDacl(
            descriptor.0.as_ptr(),
            &mut present,
            &mut acl,
            &mut defaulted,
        )
    } == 0
        || unsafe {
            GetSecurityDescriptorControl(descriptor.0.as_ptr(), &mut control, &mut revision)
        } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if present == 0 || acl.is_null() {
        return Err(io::Error::other("permission comparison requires a DACL"));
    }
    let mut entries = Vec::new();
    for index in 0..u32::from(unsafe { (*acl).AceCount }) {
        let mut entry = null_mut();
        if unsafe { GetAce(acl, index, &mut entry) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let header = unsafe { &*entry.cast::<ACE_HEADER>() };
        entries.push(unsafe {
            std::slice::from_raw_parts(entry.cast::<u8>(), header.AceSize as usize).to_vec()
        });
    }
    // SetSecurityInfo normalizes AUTO_INHERITED; ACE bytes and protection define access.
    Ok((entries, control & SE_DACL_PROTECTED != 0))
}

#[cfg(test)]
fn directory_descriptor(path: &Path) -> io::Result<LocalAllocation> {
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let mut descriptor = null_mut();
    status(unsafe {
        GetSecurityInfo(
            directory.as_raw_handle(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            null_mut(),
            null_mut(),
            &mut descriptor,
        )
    })?;
    LocalAllocation::new(descriptor)
}

#[cfg(test)]
#[path = "tests/namespace_lock.rs"]
mod tests;
