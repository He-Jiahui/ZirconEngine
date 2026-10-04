use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

struct OpenedFile {
    file: File,
    _parents: Vec<File>,
}

pub(crate) fn read_bounded_regular(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let path = std::path::absolute(path)?;
    let opened = open_regular(&path)?;
    let metadata = opened.file.metadata()?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err(invalid_file());
    }
    let bound = (limit as u64).checked_add(1).ok_or_else(invalid_file)?;
    let mut bytes = Vec::new();
    opened.file.take(bound).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(invalid_file());
    }
    Ok(bytes)
}

fn invalid_file() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "expected a local regular file")
}

#[cfg(windows)]
fn open_regular(path: &Path) -> io::Result<OpenedFile> {
    use std::{
        os::windows::{ffi::OsStrExt, fs::MetadataExt},
        path::{Component, Prefix},
    };
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const FILE_SHARE_READ: u32 = 1;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

    if !matches!(
        path.components().next(),
        Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
    ) || path.components().any(|component| match component {
        Component::ParentDir => true,
        Component::Normal(name) => name.encode_wide().any(|unit| unit == b':' as u16),
        _ => false,
    }) {
        return Err(invalid_file());
    }

    // Keep each checked ancestor open against rename until the read ends.
    let ancestors: Vec<_> = path.ancestors().skip(1).collect();
    let mut parents = Vec::with_capacity(ancestors.len());
    for ancestor in ancestors.into_iter().rev() {
        let parent = unsafe {
            super::windows::open_no_reparse(
                ancestor,
                FILE_READ_ATTRIBUTES,
                FILE_SHARE_READ,
                super::windows::FILE_OPEN,
                true,
                std::ptr::null_mut(),
            )
        }?;
        let metadata = parent.metadata()?;
        if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(invalid_file());
        }
        parents.push(parent);
    }
    let file = unsafe {
        super::windows::open_no_reparse(
            path,
            0x8000_0000,
            FILE_SHARE_READ,
            super::windows::FILE_OPEN,
            false,
            std::ptr::null_mut(),
        )
    }?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(invalid_file());
    }
    Ok(OpenedFile {
        file,
        _parents: parents,
    })
}

#[cfg(unix)]
fn open_regular(path: &Path) -> io::Result<OpenedFile> {
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::ffi::OsStrExt,
        },
        path::Component,
    };
    let mut components = path.components().peekable();
    if components.next() != Some(Component::RootDir) {
        return Err(invalid_file());
    }
    let mut current = File::open("/")?;
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err(invalid_file());
        };
        let name = CString::new(name.as_bytes()).map_err(|_| invalid_file())?;
        let mut flags = libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK;
        if components.peek().is_some() {
            flags |= libc::O_DIRECTORY;
        }
        // Resolve one name through its opened parent; O_NONBLOCK also rejects FIFOs promptly.
        let descriptor = unsafe { libc::openat(current.as_raw_fd(), name.as_ptr(), flags) };
        if descriptor < 0 {
            return Err(io::Error::last_os_error());
        }
        current = unsafe { File::from_raw_fd(descriptor) };
    }
    if !current.metadata()?.is_file() {
        return Err(invalid_file());
    }
    Ok(OpenedFile {
        file: current,
        _parents: Vec::new(),
    })
}

#[cfg(not(any(windows, unix)))]
fn open_regular(_: &Path) -> io::Result<OpenedFile> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "secure file reads are unavailable",
    ))
}

#[cfg(test)]
#[path = "regular_file/tests/cases.rs"]
mod tests;
