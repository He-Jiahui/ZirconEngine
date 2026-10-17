use super::{digest, AccountError};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::Read,
    os::windows::{fs::MetadataExt, io::AsRawHandle},
    path::Path,
};

pub(super) struct VerifiedInput {
    _file: File,
    _parents: Vec<File>,
}

fn parents(path: &Path) -> Result<Vec<File>, AccountError> {
    let mut files = Vec::new();
    for parent in path
        .parent()
        .ok_or(AccountError::PackageUnavailable)?
        .ancestors()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        files.push(open_directory(parent)?);
    }
    Ok(files)
}

fn open_directory(path: &Path) -> Result<File, AccountError> {
    unsafe {
        crate::file_io::windows::open_no_reparse(
            path,
            0x0012_0081,
            3,
            crate::file_io::windows::FILE_OPEN,
            true,
            std::ptr::null_mut(),
        )
    }
    .map_err(|_| AccountError::PackageUnavailable)
}

pub(super) fn root_identity(path: &Path) -> Result<(VerifiedInput, String), AccountError> {
    let parents = parents(path)?;
    let file = open_directory(path)?;
    let mut information = [0u32; 13];
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) } == 0 {
        return Err(AccountError::PackageUnavailable);
    }
    let identity = serde_json::to_vec(&(
        information[7],
        information[11],
        information[12],
        path.to_string_lossy().to_ascii_lowercase(),
    ))
    .map_err(|_| AccountError::PackageUnavailable)?;
    Ok((
        VerifiedInput {
            _file: file,
            _parents: parents,
        },
        digest(&identity),
    ))
}

pub(super) fn verified_input(
    path: &Path,
    expected: &str,
    limit: usize,
) -> Result<VerifiedInput, AccountError> {
    let parents = parents(path)?;
    let mut file = unsafe {
        crate::file_io::windows::open_no_reparse(
            path,
            0x8000_0000,
            1,
            crate::file_io::windows::FILE_OPEN,
            false,
            std::ptr::null_mut(),
        )
    }
    .map_err(|_| AccountError::PackageUnavailable)?;
    let metadata = file
        .metadata()
        .map_err(|_| AccountError::PackageUnavailable)?;
    if !metadata.is_file()
        || metadata.file_attributes() & 0x400 != 0
        || metadata.len() > limit as u64
    {
        return Err(AccountError::PackageUnavailable);
    }
    let mut information = [0u32; 13];
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) } == 0
        || information[10] != 1
    {
        return Err(AccountError::PackageUnavailable);
    }
    let mut hasher = Sha256::new();
    let mut size = 0usize;
    let mut buffer = [0u8; 65536];
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|_| AccountError::PackageUnavailable)?;
        if count == 0 {
            break;
        }
        size = size
            .checked_add(count)
            .filter(|size| *size <= limit)
            .ok_or(AccountError::PackageUnavailable)?;
        hasher.update(&buffer[..count]);
    }
    if format!("{:x}", hasher.finalize()) != expected {
        return Err(AccountError::PackageTrust);
    }
    Ok(VerifiedInput {
        _file: file,
        _parents: parents,
    })
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetFileInformationByHandle(file: *mut std::ffi::c_void, information: *mut u32) -> i32;
}
