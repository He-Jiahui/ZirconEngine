use std::fs::{self, File, OpenOptions};
use std::io;
use std::mem::{offset_of, size_of};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use windows_sys::Win32::Foundation::{ERROR_LOCK_VIOLATION, ERROR_SHARING_VIOLATION};
use windows_sys::Win32::Storage::FileSystem::{
    FileDispositionInfo, FileRenameInfo, SetFileInformationByHandle, DELETE,
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_DISPOSITION_INFO, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_READ_ATTRIBUTES, FILE_RENAME_INFO, FILE_SHARE_READ,
};

const TEMPORARY_RECOVERY_ENTRY_LIMIT: usize = 1_024;

pub(super) fn retire(file: &File) -> io::Result<()> {
    let info = FILE_DISPOSITION_INFO { DeleteFile: true };
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FileDispositionInfo,
            (&info as *const FILE_DISPOSITION_INFO).cast(),
            size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(super) fn recover_temporary_receipts(parent: &Path, file_name: &str) -> io::Result<()> {
    let prefix = format!(".{file_name}.receipt-");
    for entry in fs::read_dir(parent)?.take(TEMPORARY_RECOVERY_ENTRY_LIMIT) {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(identity) = name
            .strip_prefix(&prefix)
            .and_then(|name| name.strip_suffix(".tmp"))
        else {
            continue;
        };
        let Some((process, sequence)) = identity.split_once('-') else {
            continue;
        };
        if process.is_empty()
            || sequence.is_empty()
            || !process.bytes().all(|byte| byte.is_ascii_digit())
            || !sequence.bytes().all(|byte| byte.is_ascii_digit())
            || process.parse::<u32>().is_err()
            || sequence.parse::<u64>().is_err()
        {
            continue;
        }
        recover_temporary_receipt(&path)?;
    }
    Ok(())
}

pub(super) fn recover_temporary_receipt(path: &Path) -> io::Result<bool> {
    let opened = OpenOptions::new()
        .read(true)
        .access_mode(FILE_READ_ATTRIBUTES | DELETE)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path);
    let file = match opened {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
        Err(error)
            if error.raw_os_error().is_some_and(|code| {
                [ERROR_SHARING_VIOLATION, ERROR_LOCK_VIOLATION].contains(&(code as u32))
            }) =>
        {
            // Every active publisher retains a handle that denies this delete access.
            return Ok(false);
        }
        Err(error) => return Err(error),
    };
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(io::Error::other(
            "temporary receipt recovery requires a non-reparse file",
        ));
    }
    retire(&file).map_err(|error| {
        io::Error::other(format!(
            "temporary receipt cleanup failed for `{}`: {error}",
            path.display()
        ))
    })?;
    Ok(true)
}

pub(super) fn publish(file: &File, output_path: &Path) -> io::Result<()> {
    let output_path = std::path::absolute(output_path)?;
    let name = output_path.as_os_str().encode_wide().collect::<Vec<_>>();
    if name.contains(&0) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "receipt publication path contains a null character",
        ));
    }
    let name_bytes = name.len().checked_mul(size_of::<u16>()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "receipt publication path is too long",
        )
    })?;
    let header_bytes = offset_of!(FILE_RENAME_INFO, FileName);
    let bytes = header_bytes.checked_add(name_bytes).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "receipt publication path is too long",
        )
    })?;
    let length = u32::try_from(bytes).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "receipt publication path is too long",
        )
    })?;
    // usize storage provides the alignment of FILE_RENAME_INFO, including its HANDLE.
    let mut buffer = vec![
        0_usize;
        bytes
            .max(size_of::<FILE_RENAME_INFO>())
            .div_ceil(size_of::<usize>())
    ];
    let info = buffer.as_mut_ptr().cast::<FILE_RENAME_INFO>();
    unsafe {
        (*info).FileNameLength = name_bytes as u32;
        std::ptr::copy_nonoverlapping(
            name.as_ptr(),
            buffer
                .as_mut_ptr()
                .cast::<u8>()
                .add(header_bytes)
                .cast::<u16>(),
            name.len(),
        );
        // ReplaceIfExists remains false. Rename preserves the original locked file identity.
        if SetFileInformationByHandle(file.as_raw_handle(), FileRenameInfo, info.cast(), length)
            == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}
