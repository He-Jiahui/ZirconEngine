use super::{PackageError, Result};
mod native;
use std::{
    fs::File,
    io::Read,
    os::windows::{fs::MetadataExt, io::AsRawHandle},
    path::{Component, Path, PathBuf, Prefix},
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, LocalFree},
    Security::{
        Authorization::{ConvertStringSidToSidW, GetSecurityInfo, SE_FILE_OBJECT},
        EqualSid, GetAce, GetSecurityDescriptorControl, GetTokenInformation, TokenUser,
        ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, DACL_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
        SE_DACL_PROTECTED, TOKEN_QUERY, TOKEN_USER,
    },
    Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

pub(super) struct DirectoryPins {
    _files: Vec<File>,
    path: PathBuf,
}

pub(super) struct PrivateFileLock {
    _parents: DirectoryPins,
    _file: File,
}

fn no_link(file: &File, directory: bool) -> Result<()> {
    let metadata = file.metadata().map_err(|_| PackageError::Storage)?;
    if metadata.file_attributes() & 0x400 != 0
        || if directory {
            !metadata.is_dir()
        } else {
            !metadata.is_file()
        }
    {
        return Err(PackageError::Storage);
    }
    if !directory {
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0
            || info.nNumberOfLinks != 1
        {
            return Err(PackageError::Storage);
        }
    }
    Ok(())
}

impl DirectoryPins {
    pub(super) fn open(path: &Path, create: bool) -> Result<Self> {
        if !path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
        {
            return Err(PackageError::Invalid);
        }
        if !matches!(path.components().next(), Some(Component::Prefix(prefix)) if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        {
            return Err(PackageError::Invalid);
        }
        let mut files = Vec::new();
        let mut current = PathBuf::new();
        for part in path.components() {
            current.push(part);
            if matches!(part, Component::Prefix(_)) {
                continue;
            }
            let file = native::open(
                &current,
                files.last(),
                0x0012_0081,
                1,
                if create && matches!(part, Component::Normal(_)) {
                    3
                } else {
                    1
                },
                true,
            )?;
            no_link(&file, true)?;
            files.push(file);
        }
        if files.is_empty() {
            return Err(PackageError::Invalid);
        }
        let path = std::fs::canonicalize(path).map_err(|_| PackageError::Storage)?;
        Ok(Self {
            _files: files,
            path,
        })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn require_private_owner(&self) -> Result<()> {
        self.require_owner_acl(true)
    }

    pub(super) fn require_private_access(&self) -> Result<()> {
        self.require_owner_acl(false)
    }

    fn require_owner_acl(&self, protected: bool) -> Result<()> {
        let file = self._files.last().ok_or(PackageError::Storage)?;
        require_private_acl(file, protected)
    }
}

fn require_private_acl(file: &File, protected: bool) -> Result<()> {
    let mut descriptor = std::ptr::null_mut();
    let mut owner = std::ptr::null_mut();
    let mut dacl: *mut ACL = std::ptr::null_mut();
    if unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner,
            std::ptr::null_mut(),
            &mut dacl,
            std::ptr::null_mut(),
            &mut descriptor,
        )
    } != 0
    {
        return Err(PackageError::Storage);
    }
    let result = unsafe { private_descriptor(descriptor, owner, dacl, protected) };
    unsafe {
        LocalFree(descriptor);
    }
    result
}

unsafe fn private_descriptor(
    descriptor: *mut std::ffi::c_void,
    owner: *mut std::ffi::c_void,
    dacl: *mut ACL,
    protected: bool,
) -> Result<()> {
    let mut control = 0;
    let mut revision = 0;
    if descriptor.is_null()
        || owner.is_null()
        || dacl.is_null()
        || unsafe { GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) } == 0
        || (protected && control & SE_DACL_PROTECTED == 0)
    {
        return Err(PackageError::Storage);
    }
    let mut token = std::ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return Err(PackageError::Storage);
    }
    let mut length = 0;
    unsafe {
        GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut length);
    }
    if length < std::mem::size_of::<TOKEN_USER>() as u32 || length > 65536 {
        unsafe {
            CloseHandle(token);
        }
        return Err(PackageError::Storage);
    }
    let mut buffer = vec![0usize; (length as usize).div_ceil(std::mem::size_of::<usize>())];
    let valid = unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            length,
            &mut length,
        )
    } != 0
        && unsafe { EqualSid(owner, (*(buffer.as_ptr().cast::<TOKEN_USER>())).User.Sid) } != 0;
    unsafe {
        CloseHandle(token);
    }
    if !valid {
        return Err(PackageError::Storage);
    }
    let mut system = std::ptr::null_mut();
    let sid: Vec<u16> = "S-1-5-18\0".encode_utf16().collect();
    if unsafe { ConvertStringSidToSidW(sid.as_ptr(), &mut system) } == 0 {
        return Err(PackageError::Storage);
    }
    let result = (|| {
        for index in 0..unsafe { (*dacl).AceCount } {
            let mut ace = std::ptr::null_mut();
            if unsafe { GetAce(dacl, index.into(), &mut ace) } == 0 || ace.is_null() {
                return Err(PackageError::Storage);
            }
            let header = unsafe { &*(ace.cast::<ACE_HEADER>()) };
            const ACCESS_ALLOWED_ACE_TYPE: u8 = 0;
            if header.AceType != ACCESS_ALLOWED_ACE_TYPE {
                return Err(PackageError::Storage);
            }
            let allowed = unsafe { &*(ace.cast::<ACCESS_ALLOWED_ACE>()) };
            let sid = std::ptr::addr_of!(allowed.SidStart).cast_mut().cast();
            if unsafe { EqualSid(sid, owner) } == 0 && unsafe { EqualSid(sid, system) } == 0 {
                return Err(PackageError::Storage);
            }
        }
        Ok(())
    })();
    unsafe {
        LocalFree(system);
    }
    result
}

pub(super) fn lock(path: &Path) -> Result<File> {
    let parents = DirectoryPins::open(path.parent().ok_or(PackageError::Invalid)?, false)?;
    let file = native::open(path, parents._files.last(), 0xC000_0000, 3, 3, false)?;
    no_link(&file, false)?;
    file.try_lock().map_err(|_| PackageError::Busy)?;
    Ok(file)
}

pub(super) fn lock_private_file(path: &Path) -> Result<PrivateFileLock> {
    let parents = DirectoryPins::open(path.parent().ok_or(PackageError::Invalid)?, false)?;
    parents.require_private_owner()?;
    let file = native::open(path, parents._files.last(), 0xC000_0000, 3, 3, false)?;
    no_link(&file, false)?;
    require_private_acl(&file, false)?;
    file.try_lock().map_err(|_| PackageError::Busy)?;
    Ok(PrivateFileLock {
        _parents: parents,
        _file: file,
    })
}

pub fn read_regular(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let parents = DirectoryPins::open(path.parent().ok_or(PackageError::Invalid)?, false)?;
    let file = native::open(path, parents._files.last(), 0x8000_0000, 1, 1, false)?;
    no_link(&file, false)?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| PackageError::Storage)?;
    if bytes.len() > limit {
        return Err(PackageError::Capacity);
    }
    Ok(bytes)
}

/// Reads host configuration through pinned, owner-private Windows handles.
pub fn read_private_regular(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let parents = DirectoryPins::open(path.parent().ok_or(PackageError::Invalid)?, false)?;
    parents.require_private_owner()?;
    let file = native::open(path, parents._files.last(), 0x8002_0000, 1, 1, false)?;
    no_link(&file, false)?;
    require_private_acl(&file, false)?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| PackageError::Storage)?;
    if bytes.len() > limit {
        return Err(PackageError::Capacity);
    }
    Ok(bytes)
}

/// Atomically publishes owner-private host configuration after pinning and checking its parent.
///
/// The package service never accepts a path supplied by a project or package here. Callers derive
/// the path from an already authenticated host policy index. The parent handle and post-publish
/// read keep the existing private ACL and no-link checks on both sides of the replacement.
pub(super) fn write_private_regular_atomic(path: &Path, bytes: &[u8], limit: usize) -> Result<()> {
    if bytes.len() > limit {
        return Err(PackageError::Capacity);
    }
    let parents = DirectoryPins::open(path.parent().ok_or(PackageError::Invalid)?, false)?;
    parents.require_private_owner()?;
    crate::core::resource::io::atomic_write(path, bytes).map_err(|_| PackageError::Storage)?;
    let published = read_private_regular(path, limit)?;
    if published != bytes {
        return Err(PackageError::Trust);
    }
    Ok(())
}
