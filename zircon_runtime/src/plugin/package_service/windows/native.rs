use super::{File, PackageError, Path, Result};
use std::{
    ffi::c_void,
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    path::{Component, Prefix},
};

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
    quality: *mut c_void,
}
#[repr(C)]
struct IoStatus {
    status: usize,
    information: usize,
}

pub(super) fn open(
    path: &Path,
    parent: Option<&File>,
    access: u32,
    share: u32,
    disposition: u32,
    directory: bool,
) -> Result<File> {
    let mut name: Vec<u16> = if parent.is_some() {
        let name = path.file_name().ok_or(PackageError::Invalid)?;
        let encoded: Vec<_> = name.encode_wide().collect();
        if encoded
            .iter()
            .any(|unit| [0, b':' as u16, b'/' as u16, b'\\' as u16].contains(unit))
        {
            return Err(PackageError::Invalid);
        }
        encoded
    } else {
        let verbatim = match path.components().next() {
            Some(Component::Prefix(prefix)) => match prefix.kind() {
                Prefix::Disk(_) => false,
                Prefix::VerbatimDisk(_) => true,
                _ => return Err(PackageError::Invalid),
            },
            _ => return Err(PackageError::Invalid),
        };
        let encoded: Vec<_> = path.as_os_str().encode_wide().collect();
        if encoded.contains(&0) {
            return Err(PackageError::Invalid);
        }
        let mut native: Vec<_> = "\\??\\".encode_utf16().collect();
        native.extend_from_slice(if verbatim { &encoded[4..] } else { &encoded });
        native
    };
    let length = name
        .len()
        .checked_mul(2)
        .and_then(|length| u16::try_from(length).ok())
        .ok_or(PackageError::Invalid)?;
    let mut unicode = UnicodeString {
        length,
        maximum_length: length,
        buffer: name.as_mut_ptr(),
    };
    let mut attributes = ObjectAttributes {
        length: std::mem::size_of::<ObjectAttributes>() as u32,
        root: parent.map_or(std::ptr::null_mut(), AsRawHandle::as_raw_handle),
        name: &mut unicode,
        attributes: 0x40 | 0x1000,
        security: std::ptr::null_mut(),
        quality: std::ptr::null_mut(),
    };
    let mut io = IoStatus {
        status: 0,
        information: 0,
    };
    let mut handle = std::ptr::null_mut();
    let status = unsafe {
        NtCreateFile(
            &mut handle,
            access | 0x0010_0000,
            &mut attributes,
            &mut io,
            std::ptr::null(),
            0x80,
            share,
            disposition,
            0x0020_0000 | 0x20 | if directory { 1 } else { 0x40 },
            std::ptr::null(),
            0,
        )
    };
    if status < 0 {
        return Err(PackageError::Storage);
    }
    Ok(unsafe { File::from_raw_handle(handle) })
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtCreateFile(
        handle: *mut *mut c_void,
        access: u32,
        attributes: *mut ObjectAttributes,
        status: *mut IoStatus,
        allocation: *const i64,
        attributes_file: u32,
        share: u32,
        disposition: u32,
        options: u32,
        ea: *const c_void,
        ea_length: u32,
    ) -> i32;
}
