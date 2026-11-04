//! No-follow authoring access before asset transactions select their durable paths.
//!
//! Directory operations use the leased parent on Linux. File transactions still use their
//! durable absolute names; callers must revalidate those names immediately before commit.

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

pub(crate) const MAX_DIRECTORY_ENTRIES: usize = 65_536;

/// A directory entry may already be visible when syncing its parent fails. Owners publish
/// their new generation before reporting this durability error, just as file transactions do.
pub(crate) struct DirectoryMutationOutcome {
    synchronization: io::Result<()>,
}

impl DirectoryMutationOutcome {
    pub(crate) fn ensure_durable(self) -> io::Result<()> {
        self.synchronization
    }
}

#[cfg(target_os = "linux")]
#[path = "safe_mutation_path/linux.rs"]
mod platform;
#[cfg(windows)]
#[path = "safe_mutation_path/windows.rs"]
mod platform;

pub(crate) fn validate_no_follow_path(
    root: &Path,
    path: &Path,
    allow_missing: bool,
) -> io::Result<()> {
    validate_lexical(root, path)?;
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if super::safe_project_path::is_link_or_reparse(&metadata) => {
                return Err(unsafe_path(&current));
            }
            Ok(metadata) if current != path && !metadata.is_dir() => {
                return Err(unsafe_path(&current));
            }
            Ok(_) => {}
            Err(error) if allow_missing && error.kind() == io::ErrorKind::NotFound => {
                return Ok(());
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

pub(crate) fn read_regular_file(root: &Path, path: &Path) -> io::Result<Vec<u8>> {
    read_regular_file_limited(root, path, usize::MAX)
}

pub(crate) fn read_regular_file_limited(
    root: &Path,
    path: &Path,
    limit: usize,
) -> io::Result<Vec<u8>> {
    let parent = DirectoryLease::open(root, parent_of(path)?)?;
    let mut file = platform::open_file(&parent.file, path)?;
    let before = file.metadata()?;
    if !before.is_file() || before.len() > limit as u64 {
        return Err(unsafe_path(path));
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(unsafe_path(path));
    }
    parent.verify()?;
    let current = platform::open_file(&parent.file, path)?;
    if !platform::same_file(&before, &current.metadata()?) {
        return Err(unsafe_path(path));
    }
    Ok(bytes)
}

pub(crate) fn read_directory(
    root: &Path,
    path: &Path,
) -> io::Result<Vec<(OsString, fs::Metadata)>> {
    let entries = read_directory_entries(root, path)?;
    for (name, metadata) in &entries {
        if super::safe_project_path::is_link_or_reparse(metadata) {
            return Err(unsafe_path(&path.join(name)));
        }
    }
    Ok(entries)
}

/// Enumerates without following entries, allowing read-only inventories to skip unsafe leaves.
pub(crate) fn read_directory_entries(
    root: &Path,
    path: &Path,
) -> io::Result<Vec<(OsString, fs::Metadata)>> {
    read_directory_with_limit(root, path, MAX_DIRECTORY_ENTRIES)
}

fn read_directory_with_limit(
    root: &Path,
    path: &Path,
    limit: usize,
) -> io::Result<Vec<(OsString, fs::Metadata)>> {
    let directory = DirectoryLease::open(root, path)?;
    let mut entries = Vec::new();
    for entry in fs::read_dir(platform::directory_path(&directory.file, path))? {
        if entries.len() == limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "asset folder inventory exceeds its entry limit",
            ));
        }
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        entries.push((entry.file_name(), metadata));
    }
    directory.verify()?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(entries)
}

#[cfg(all(test, target_os = "linux"))]
#[path = "safe_mutation_path/tests.rs"]
mod tests;

pub(crate) fn create_directory(root: &Path, path: &Path) -> io::Result<DirectoryMutationOutcome> {
    validate_lexical(root, path)?;
    if path == root {
        return Err(unsafe_path(path));
    }
    let parent = DirectoryLease::open(root, parent_of(path)?)?;
    parent.verify()?;
    platform::create_directory(&parent.file, path)?;
    if let Err(error) = parent.verify() {
        // Only the just-created empty entry in our original directory may be retired.
        let _ = platform::remove_directory(&parent.file, path);
        return Err(error);
    }
    Ok(DirectoryMutationOutcome {
        synchronization: platform::sync_directory(&parent.file),
    })
}

pub(crate) fn remove_empty_directory(
    root: &Path,
    path: &Path,
) -> io::Result<DirectoryMutationOutcome> {
    validate_lexical(root, path)?;
    if path == root {
        return Err(unsafe_path(path));
    }
    let parent = DirectoryLease::open(root, parent_of(path)?)?;
    let directory = platform::open_directory(Some(&parent.file), path)?;
    parent.verify()?;
    drop(directory);
    platform::remove_directory(&parent.file, path)?;
    Ok(DirectoryMutationOutcome {
        synchronization: platform::sync_directory(&parent.file),
    })
}

struct DirectoryLease {
    file: File,
    chain: Vec<(PathBuf, File)>,
}

impl DirectoryLease {
    fn open(root: &Path, path: &Path) -> io::Result<Self> {
        validate_lexical(root, path)?;
        let mut current = PathBuf::new();
        let mut chain: Vec<(PathBuf, File)> = Vec::new();
        for component in path.components() {
            current.push(component.as_os_str());
            if matches!(component, Component::Prefix(_)) {
                continue;
            }
            let parent = chain.last().map(|(_, file)| file);
            let file = platform::open_directory(parent, &current)?;
            chain.push((current.clone(), file));
        }
        let file = chain
            .last()
            .ok_or_else(|| unsafe_path(path))?
            .1
            .try_clone()?;
        let lease = Self { file, chain };
        lease.verify()?;
        Ok(lease)
    }

    fn verify(&self) -> io::Result<()> {
        let mut previous: Option<File> = None;
        for (path, pinned) in &self.chain {
            let current = platform::open_directory(previous.as_ref(), path)?;
            if !platform::same_identity(&pinned.metadata()?, &current.metadata()?) {
                return Err(unsafe_path(path));
            }
            previous = Some(current);
        }
        Ok(())
    }
}

fn validate_lexical(root: &Path, path: &Path) -> io::Result<()> {
    if !root.is_absolute()
        || !path.is_absolute()
        || !path.starts_with(root)
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(unsafe_path(path));
    }
    let relative = path.strip_prefix(root).map_err(|_| unsafe_path(path))?;
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(unsafe_path(path));
        };
        let name = name.to_str().ok_or_else(|| unsafe_path(path))?;
        if name.ends_with(['.', ' '])
            || name.chars().any(|ch| {
                ch.is_control()
                    || matches!(ch, '\\' | ':' | '#' | '<' | '>' | '"' | '|' | '?' | '*')
            })
            || zircon_runtime_interface::project::RelPath::parse(name).is_err()
        {
            return Err(unsafe_path(path));
        }
    }
    Ok(())
}

fn parent_of(path: &Path) -> io::Result<&Path> {
    path.parent().ok_or_else(|| unsafe_path(path))
}

fn unsafe_path(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "asset mutation requires a regular no-follow path: {}",
            path.display()
        ),
    )
}

#[cfg(not(any(target_os = "linux", windows)))]
mod platform {
    use super::*;
    fn unsupported() -> io::Error {
        io::Error::new(
            io::ErrorKind::Unsupported,
            "asset mutation requires platform path leases",
        )
    }
    pub(super) fn open_directory(_: Option<&File>, _: &Path) -> io::Result<File> {
        Err(unsupported())
    }
    pub(super) fn open_file(_: &File, _: &Path) -> io::Result<File> {
        Err(unsupported())
    }
    pub(super) fn create_directory(_: &File, _: &Path) -> io::Result<()> {
        Err(unsupported())
    }
    pub(super) fn remove_directory(_: &File, _: &Path) -> io::Result<()> {
        Err(unsupported())
    }
    pub(super) fn directory_path(_: &File, path: &Path) -> PathBuf {
        path.to_path_buf()
    }
    pub(super) fn same_identity(_: &fs::Metadata, _: &fs::Metadata) -> bool {
        false
    }
    pub(super) fn same_file(_: &fs::Metadata, _: &fs::Metadata) -> bool {
        false
    }
    pub(super) fn sync_directory(_: &File) -> io::Result<()> {
        Err(unsupported())
    }
}
