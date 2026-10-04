use std::{
    ffi::{c_void, OsStr},
    fs::File,
    io,
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Component, Path, Prefix},
};

mod identity;
pub(crate) use identity::same_file;

pub(crate) const FILE_OPEN: u32 = 1;
pub(crate) const FILE_CREATE: u32 = 2;
pub(crate) const FILE_OPEN_IF: u32 = 3;
const FILE_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x20;
const FILE_DIRECTORY_FILE: u32 = 0x1;
const FILE_NON_DIRECTORY_FILE: u32 = 0x40;
const OBJ_CASE_INSENSITIVE: u32 = 0x40;
const OBJ_DONT_REPARSE: u32 = 0x1000;
const DACL_SECURITY_INFORMATION: u32 = 4;
const PROTECTED_DACL_SECURITY_INFORMATION: u32 = 0x8000_0000;

struct LocalAllocation(*mut c_void);

impl Drop for LocalAllocation {
    fn drop(&mut self) {
        unsafe { LocalFree(self.0) };
    }
}

#[repr(C)]
struct UnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

#[repr(C)]
struct ObjectAttributes {
    length: u32,
    root: *mut c_void,
    name: *mut UnicodeString,
    attributes: u32,
    security: *mut c_void,
    security_quality: *mut c_void,
}

#[repr(C)]
struct IoStatus {
    status: usize,
    information: usize,
}

/// Opens the entire local path without following reparse points, including raced ancestors.
///
/// # Safety
/// `security` must be null or a valid security descriptor for this synchronous call.
pub(crate) unsafe fn open_no_reparse(
    path: &Path,
    access: u32,
    share: u32,
    disposition: u32,
    directory: bool,
    security: *mut c_void,
) -> io::Result<File> {
    let mut name = native_object_path(path)?;
    let length = name
        .len()
        .checked_mul(2)
        .and_then(|bytes| u16::try_from(bytes).ok())
        .ok_or_else(invalid_path)?;
    let mut unicode = UnicodeString {
        length,
        maximum_length: length,
        buffer: name.as_mut_ptr(),
    };
    let mut attributes = ObjectAttributes {
        length: std::mem::size_of::<ObjectAttributes>() as u32,
        root: std::ptr::null_mut(),
        name: &mut unicode,
        attributes: 0x40 | 0x1000, // OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE
        security,
        security_quality: std::ptr::null_mut(),
    };
    let mut status = IoStatus {
        status: 0,
        information: 0,
    };
    let mut handle = std::ptr::null_mut();
    let result = unsafe {
        NtCreateFile(
            &mut handle,
            access | 0x0010_0000, // SYNCHRONIZE for synchronous Rust File I/O
            &mut attributes,
            &mut status,
            std::ptr::null(),
            0x80,
            share,
            disposition,
            0x0020_0000 | 0x20 | if directory { 1 } else { 0x40 },
            std::ptr::null_mut(),
            0,
        )
    };
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(result) } as i32,
        ));
    }
    Ok(unsafe { File::from_raw_handle(handle) })
}

fn native_object_path(path: &Path) -> io::Result<Vec<u16>> {
    let path = std::path::absolute(path)?;
    let mut components = path.components();
    let prefix = match components.next() {
        Some(Component::Prefix(prefix)) => prefix,
        _ => return Err(invalid_path()),
    };
    if components.next() != Some(Component::RootDir)
        || components.any(|component| match component {
            Component::ParentDir => true,
            Component::Normal(name) => name.encode_wide().any(|unit| unit == b':' as u16),
            _ => false,
        })
    {
        return Err(invalid_path());
    }

    let encoded: Vec<_> = path.as_os_str().encode_wide().collect();
    if encoded.contains(&0) {
        return Err(invalid_path());
    }
    let (native_prefix, offset) = match prefix.kind() {
        Prefix::Disk(_) => ("\\??\\", 0),
        Prefix::VerbatimDisk(_) => ("\\??\\", 4),
        Prefix::UNC(_, _) => ("\\??\\UNC\\", 2),
        Prefix::VerbatimUNC(_, _) => ("\\??\\UNC\\", 8),
        Prefix::Verbatim(_) if is_volume_guid_prefix(prefix.as_os_str()) => ("\\??\\", 4),
        _ => return Err(invalid_path()),
    };
    let mut name: Vec<u16> = native_prefix.encode_utf16().collect();
    name.extend_from_slice(encoded.get(offset..).ok_or_else(invalid_path)?);
    Ok(name)
}

fn is_volume_guid_prefix(prefix: &OsStr) -> bool {
    let prefix = prefix.to_string_lossy().to_ascii_lowercase();
    let Some(guid) = prefix
        .strip_prefix("\\\\?\\volume{")
        .and_then(|value| value.strip_suffix('}'))
    else {
        return false;
    };
    guid.len() == 36
        && guid.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        })
}

/// Opens one child relative to a directory handle. No path prefix is re-resolved.
pub(crate) fn open_relative(
    parent: &File,
    name: &OsStr,
    access: u32,
    directory: Option<bool>,
    share: u32,
) -> io::Result<File> {
    let path = Path::new(name);
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
        || name.encode_wide().any(|unit| {
            unit == 0 || unit == b':' as u16 || unit == b'/' as u16 || unit == b'\\' as u16
        })
    {
        return Err(invalid_path());
    }
    let mut encoded: Vec<u16> = name.encode_wide().collect();
    let length = encoded
        .len()
        .checked_mul(2)
        .and_then(|bytes| u16::try_from(bytes).ok())
        .ok_or_else(invalid_path)?;
    let mut unicode = UnicodeString {
        length,
        maximum_length: length,
        buffer: encoded.as_mut_ptr(),
    };
    let mut attributes = ObjectAttributes {
        length: std::mem::size_of::<ObjectAttributes>() as u32,
        root: parent.as_raw_handle() as *mut c_void,
        name: &mut unicode,
        attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        security: std::ptr::null_mut(),
        security_quality: std::ptr::null_mut(),
    };
    let mut status = IoStatus {
        status: 0,
        information: 0,
    };
    let mut handle = std::ptr::null_mut();
    let options = FILE_OPEN_REPARSE_POINT
        | FILE_SYNCHRONOUS_IO_NONALERT
        | match directory {
            Some(true) => FILE_DIRECTORY_FILE,
            Some(false) => FILE_NON_DIRECTORY_FILE,
            None => 0,
        };
    let result = unsafe {
        NtCreateFile(
            &mut handle,
            access | 0x0010_0000, // SYNCHRONIZE for synchronous Rust File I/O
            &mut attributes,
            &mut status,
            std::ptr::null(),
            0x80,
            share,
            FILE_OPEN,
            options,
            std::ptr::null_mut(),
            0,
        )
    };
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(result) } as i32,
        ));
    }
    Ok(unsafe { File::from_raw_handle(handle) })
}

pub(crate) fn create_relative_file(parent: &File, name: &OsStr) -> io::Result<File> {
    create_relative_entry(
        parent,
        name,
        0x0002 | 0x0080, // FILE_WRITE_DATA | FILE_READ_ATTRIBUTES
        false,
        FILE_CREATE,
    )
}

pub(crate) fn create_relative_private_file(parent: &File, name: &OsStr) -> io::Result<File> {
    let descriptor = private_security_descriptor()?;
    create_relative_entry_with_security(
        parent,
        name,
        0x0002 | 0x0080, // FILE_WRITE_DATA | FILE_READ_ATTRIBUTES
        false,
        FILE_CREATE,
        descriptor.0,
    )
}

pub(crate) fn create_relative_directory(parent: &File, name: &OsStr) -> io::Result<File> {
    create_relative_entry(
        parent,
        name,
        0x0080, // FILE_READ_ATTRIBUTES
        true,
        FILE_OPEN_IF,
    )
}

pub(crate) fn create_relative_private_directory(parent: &File, name: &OsStr) -> io::Result<File> {
    let descriptor = private_security_descriptor()?;
    create_relative_entry_with_security(
        parent,
        name,
        0x0080 | 0x0004_0000, // FILE_READ_ATTRIBUTES | WRITE_DAC
        true,
        FILE_OPEN_IF,
        descriptor.0,
    )
}

pub(crate) fn protect_private_object(file: &File) -> io::Result<()> {
    let descriptor = private_security_descriptor()?;
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
    Ok(())
}

fn private_security_descriptor() -> io::Result<LocalAllocation> {
    // Keep Cloud artifacts private even when the project folder has inherited grants.
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
        let error = io::Error::last_os_error();
        if !descriptor.is_null() {
            unsafe { LocalFree(descriptor) };
        }
        return Err(error);
    }
    if descriptor.is_null() {
        return Err(io::Error::other("private security descriptor missing"));
    }
    Ok(LocalAllocation(descriptor))
}

fn create_relative_entry(
    parent: &File,
    name: &OsStr,
    access: u32,
    directory: bool,
    disposition: u32,
) -> io::Result<File> {
    create_relative_entry_with_security(
        parent,
        name,
        access,
        directory,
        disposition,
        std::ptr::null_mut(),
    )
}

fn create_relative_entry_with_security(
    parent: &File,
    name: &OsStr,
    access: u32,
    directory: bool,
    disposition: u32,
    security: *mut c_void,
) -> io::Result<File> {
    let path = Path::new(name);
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
        || name.encode_wide().any(|unit| {
            unit == 0 || unit == b':' as u16 || unit == b'/' as u16 || unit == b'\\' as u16
        })
    {
        return Err(invalid_path());
    }
    let mut encoded: Vec<u16> = name.encode_wide().collect();
    let length = encoded
        .len()
        .checked_mul(2)
        .and_then(|bytes| u16::try_from(bytes).ok())
        .ok_or_else(invalid_path)?;
    let mut unicode = UnicodeString {
        length,
        maximum_length: length,
        buffer: encoded.as_mut_ptr(),
    };
    let mut attributes = ObjectAttributes {
        length: std::mem::size_of::<ObjectAttributes>() as u32,
        root: parent.as_raw_handle() as *mut c_void,
        name: &mut unicode,
        attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        security,
        security_quality: std::ptr::null_mut(),
    };
    let mut status = IoStatus {
        status: 0,
        information: 0,
    };
    let mut handle = std::ptr::null_mut();
    let options = FILE_OPEN_REPARSE_POINT
        | FILE_SYNCHRONOUS_IO_NONALERT
        | if directory {
            FILE_DIRECTORY_FILE
        } else {
            FILE_NON_DIRECTORY_FILE
        };
    let result = unsafe {
        NtCreateFile(
            &mut handle,
            access | 0x0010_0000, // SYNCHRONIZE for synchronous Rust File I/O
            &mut attributes,
            &mut status,
            std::ptr::null(),
            0x80,
            1, // FILE_SHARE_READ; prevent a second writer from replacing the opened entry
            disposition,
            options,
            std::ptr::null_mut(),
            0,
        )
    };
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(result) } as i32,
        ));
    }
    Ok(unsafe { File::from_raw_handle(handle) })
}

/// Creates a no-clobber sibling hard link without resolving a path prefix.
pub(crate) fn hard_link_relative(file: &File, name: &OsStr) -> io::Result<()> {
    #[repr(C)]
    struct FileLinkInformation {
        replace_if_exists: u8,
        root_directory: *mut c_void,
        file_name_length: u32,
        file_name: [u16; 1],
    }

    let path = Path::new(name);
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
        || name.encode_wide().any(|unit| {
            unit == 0 || unit == b':' as u16 || unit == b'/' as u16 || unit == b'\\' as u16
        })
    {
        return Err(invalid_path());
    }
    let encoded: Vec<u16> = name.encode_wide().collect();
    let name_length = encoded
        .len()
        .checked_mul(2)
        .and_then(|bytes| u32::try_from(bytes).ok())
        .ok_or_else(invalid_path)?;
    let name_offset = std::mem::offset_of!(FileLinkInformation, file_name);
    let buffer_length = name_offset
        .checked_add(encoded.len().checked_mul(2).ok_or_else(invalid_path)?)
        .ok_or_else(invalid_path)?;
    let buffer_length = buffer_length.max(std::mem::size_of::<FileLinkInformation>());
    let word_count = buffer_length.div_ceil(std::mem::size_of::<usize>());
    let mut buffer = vec![0usize; word_count];
    let information = buffer.as_mut_ptr().cast::<FileLinkInformation>();
    unsafe {
        (*information).replace_if_exists = 0;
        // The source and new link are siblings, so FileLinkInformation's
        // same-directory form uses a null RootDirectory.
        (*information).root_directory = std::ptr::null_mut();
        (*information).file_name_length = name_length;
        std::ptr::copy_nonoverlapping(
            encoded.as_ptr(),
            std::ptr::addr_of_mut!((*information).file_name).cast::<u16>(),
            encoded.len(),
        );
    }
    let mut status = IoStatus {
        status: 0,
        information: 0,
    };
    let result = unsafe {
        NtSetInformationFile(
            file.as_raw_handle() as *mut c_void,
            &mut status,
            information.cast::<c_void>(),
            u32::try_from(buffer_length).map_err(|_| invalid_path())?,
            11, // FileLinkInformation
        )
    };
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(result) } as i32,
        ));
    }
    Ok(())
}

/// Marks an already-opened file or directory for deletion without resolving its name again.
pub(crate) fn mark_delete(file: &File) -> io::Result<()> {
    #[repr(C)]
    struct FileDispositionInformation {
        delete_file: u8,
    }
    let mut information = FileDispositionInformation { delete_file: 1 };
    let mut status = IoStatus {
        status: 0,
        information: 0,
    };
    let result = unsafe {
        NtSetInformationFile(
            file.as_raw_handle() as *mut c_void,
            &mut status,
            &mut information as *mut _ as *mut c_void,
            std::mem::size_of::<FileDispositionInformation>() as u32,
            13, // FileDispositionInformation
        )
    };
    if result < 0 {
        return Err(io::Error::from_raw_os_error(
            unsafe { RtlNtStatusToDosError(result) } as i32,
        ));
    }
    Ok(())
}

fn invalid_path() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "expected a local path without reparse points",
    )
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtCreateFile(
        handle: *mut *mut c_void,
        access: u32,
        attributes: *mut ObjectAttributes,
        status: *mut IoStatus,
        allocation: *const i64,
        file_attributes: u32,
        share: u32,
        disposition: u32,
        options: u32,
        ea: *mut c_void,
        ea_length: u32,
    ) -> i32;
    fn NtSetInformationFile(
        handle: *mut c_void,
        status: *mut IoStatus,
        information: *mut c_void,
        length: u32,
        information_class: u32,
    ) -> i32;
    fn RtlNtStatusToDosError(status: i32) -> u32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
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

#[cfg(test)]
#[path = "windows/tests/cases.rs"]
mod tests;
