use super::store::OpenMode;
use std::{
    ffi::c_void,
    fs::File,
    io,
    os::windows::{ffi::OsStrExt, fs::MetadataExt, io::AsRawHandle},
    path::Path,
};

const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_SHARE_READ: u32 = 1;
const FILE_SHARE_READ_WRITE: u32 = 3;
const FILE_SHARE_DELETE: u32 = 4;
const DACL_SECURITY_INFORMATION: u32 = 4;
const PROTECTED_DACL_SECURITY_INFORMATION: u32 = 0x80000000;
const CRYPTPROTECT_UI_FORBIDDEN: u32 = 1;

#[repr(C)]
struct Blob {
    length: u32,
    bytes: *mut u8,
}

struct LocalAllocation(*mut c_void);
impl Drop for LocalAllocation {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}

fn wide(path: &Path) -> io::Result<Vec<u16>> {
    let mut value: Vec<u16> = path.as_os_str().encode_wide().collect();
    if value.contains(&0) {
        return Err(io::Error::other("account journal path invalid"));
    }
    value.push(0);
    Ok(value)
}

pub(super) fn open_directory(path: &Path) -> io::Result<File> {
    open_directory_mode(path, crate::file_io::windows::FILE_OPEN)
}

pub(super) fn create_directory(path: &Path) -> io::Result<File> {
    open_directory_mode(path, crate::file_io::windows::FILE_OPEN_IF)
}

fn open_directory_mode(path: &Path, disposition: u32) -> io::Result<File> {
    let file = unsafe {
        crate::file_io::windows::open_no_reparse(
            path,
            FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ,
            disposition,
            true,
            std::ptr::null_mut(),
        )
    }?;
    let metadata = file.metadata()?;
    if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(io::Error::other("account journal directory invalid"));
    }
    Ok(file)
}

pub(super) fn open_private(path: &Path, mode: OpenMode) -> io::Result<File> {
    // The protected ACL grants file-owner and SYSTEM access; DPAPI additionally binds
    // contents to the current Windows user, including when an elevated owner is a group.
    let sddl: Vec<u16> = "D:P(A;;FA;;;OW)(A;;FA;;;SY)\0".encode_utf16().collect();
    let mut descriptor = std::ptr::null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let descriptor = LocalAllocation(descriptor);
    let disposition = match mode {
        OpenMode::Read => crate::file_io::windows::FILE_OPEN,
        OpenMode::Lock => crate::file_io::windows::FILE_OPEN_IF,
        OpenMode::Create => crate::file_io::windows::FILE_CREATE,
    };
    let share = FILE_SHARE_READ_WRITE
        | if matches!(mode, OpenMode::Lock) {
            0
        } else {
            FILE_SHARE_DELETE
        };
    // SAFETY: the protected security descriptor remains allocated through the open.
    let file = unsafe {
        crate::file_io::windows::open_no_reparse(
            path,
            0xC0040000,
            share,
            disposition,
            false,
            descriptor.0,
        )
    }?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(io::Error::other("account journal file invalid"));
    }
    if unsafe {
        SetKernelObjectSecurity(
            file.as_raw_handle(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            descriptor.0,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(file)
}

pub(super) fn protect(bytes: &[u8], decrypt: bool) -> io::Result<Vec<u8>> {
    let mut input = Blob {
        length: u32::try_from(bytes.len()).map_err(io::Error::other)?,
        bytes: bytes.as_ptr() as *mut u8,
    };
    let entropy_bytes = b"ZirconHub.PendingOperations.V1";
    let mut entropy = Blob {
        length: entropy_bytes.len() as u32,
        bytes: entropy_bytes.as_ptr() as *mut u8,
    };
    let mut output = Blob {
        length: 0,
        bytes: std::ptr::null_mut(),
    };
    // No LOCAL_MACHINE flag: plaintext can only be recovered under this Windows user.
    let ok = unsafe {
        if decrypt {
            CryptUnprotectData(
                &mut input,
                std::ptr::null_mut(),
                &mut entropy,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptProtectData(
                &mut input,
                std::ptr::null(),
                &mut entropy,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    let allocation = LocalAllocation(output.bytes.cast());
    if output.bytes.is_null() || output.length as usize > super::MAX_JOURNAL_BYTES {
        return Err(io::Error::other("protected account journal invalid"));
    }
    let result =
        unsafe { std::slice::from_raw_parts(output.bytes, output.length as usize) }.to_vec();
    if decrypt {
        unsafe {
            std::ptr::write_bytes(output.bytes, 0, output.length as usize);
        }
    }
    drop(allocation);
    Ok(result)
}

pub(super) fn replace(source: &Path, target: &Path) -> io::Result<()> {
    let source = wide(source)?;
    let target = wide(target)?;
    // Same-directory rename replaces atomically; WRITE_THROUGH covers first creation too.
    if unsafe { MoveFileExW(source.as_ptr(), target.as_ptr(), 1 | 8) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
    fn MoveFileExW(source: *const u16, target: *const u16, flags: u32) -> i32;
}
#[link(name = "advapi32")]
unsafe extern "system" {
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16,
        revision: u32,
        descriptor: *mut *mut c_void,
        size: *mut u32,
    ) -> i32;
    fn SetKernelObjectSecurity(
        handle: *mut c_void,
        information: u32,
        descriptor: *mut c_void,
    ) -> i32;
}
#[link(name = "crypt32")]
unsafe extern "system" {
    fn CryptProtectData(
        input: *mut Blob,
        description: *const u16,
        entropy: *mut Blob,
        reserved: *mut c_void,
        prompt: *mut c_void,
        flags: u32,
        output: *mut Blob,
    ) -> i32;
    fn CryptUnprotectData(
        input: *mut Blob,
        description: *mut *mut u16,
        entropy: *mut Blob,
        reserved: *mut c_void,
        prompt: *mut c_void,
        flags: u32,
        output: *mut Blob,
    ) -> i32;
}
