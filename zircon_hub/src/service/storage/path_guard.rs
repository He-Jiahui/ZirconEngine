use std::{
    fs::File,
    io::{self, ErrorKind},
    os::windows::{ffi::OsStrExt, fs::MetadataExt},
    path::{Component, Path, PathBuf, Prefix},
};

mod private_access;
mod windows;

const FILE_READ_ATTRIBUTES: u32 = 0x80;
const FILE_SHARE_READ: u32 = 1;
const FILE_SHARE_READ_WRITE: u32 = 3;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

pub(super) struct PathGuard {
    pub(super) path: PathBuf,
    _file: File,
    _parents: Vec<File>,
}

impl PathGuard {
    pub(super) fn open(path: &Path) -> io::Result<Self> {
        let path = std::path::absolute(path)?;
        if !matches!(
            path.components().next(),
            Some(Component::Prefix(prefix))
                if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
        ) || path.components().any(|component| match component {
            Component::ParentDir => true,
            Component::Normal(name) => name.encode_wide().any(|unit| unit == b':' as u16),
            _ => false,
        }) {
            return Err(invalid_path());
        }
        let mut parents = Vec::new();
        for parent in path
            .ancestors()
            .skip(1)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let held = unsafe {
                crate::file_io::windows::open_no_reparse(
                    parent,
                    FILE_READ_ATTRIBUTES | private_access::READ_CONTROL,
                    FILE_SHARE_READ,
                    crate::file_io::windows::FILE_OPEN,
                    true,
                    std::ptr::null_mut(),
                )
            }?;
            let metadata = held.metadata()?;
            if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err(invalid_path());
            }
            parents.push(held);
        }
        let identity = private_access::Identity::current()?;
        let mut protected = false;
        for parent in parents.iter().rev() {
            if identity.require_private(parent)? {
                protected = true;
                break;
            }
        }
        if !protected {
            return Err(invalid_path());
        }
        // SQLite opens this same object after admission. Retain it to prevent path replacement
        // until the connection and every detached database job have finished.
        let file = unsafe {
            crate::file_io::windows::open_no_reparse(
                &path,
                0xC000_0000,
                FILE_SHARE_READ_WRITE,
                crate::file_io::windows::FILE_OPEN_IF,
                false,
                std::ptr::null_mut(),
            )
        }?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || windows::link_count(&file)? != 1
        {
            return Err(invalid_path());
        }
        identity.require_private(&file)?;
        for suffix in ["-wal", "-shm", "-journal"] {
            let mut name = path.as_os_str().to_owned();
            name.push(suffix);
            let sidecar = PathBuf::from(name);
            match unsafe {
                crate::file_io::windows::open_no_reparse(
                    &sidecar,
                    FILE_READ_ATTRIBUTES | private_access::READ_CONTROL,
                    FILE_SHARE_READ_WRITE,
                    crate::file_io::windows::FILE_OPEN,
                    false,
                    std::ptr::null_mut(),
                )
            } {
                Ok(held) => {
                    let metadata = held.metadata()?;
                    if !metadata.is_file()
                        || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
                        || windows::link_count(&held)? != 1
                    {
                        return Err(invalid_path());
                    }
                    identity.require_private(&held)?;
                    // The private directory excludes other users. SQLite must be able to
                    // delete/recreate its journals during recovery and checkpointing.
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
        Ok(Self {
            path,
            _file: file,
            _parents: parents,
        })
    }
}

fn invalid_path() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "expected an unaliased local database",
    )
}

#[cfg(test)]
#[path = "path_guard/tests/cases.rs"]
mod tests;
