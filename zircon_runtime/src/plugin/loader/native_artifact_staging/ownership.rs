//! Crash recovery is authorized by an exclusive file lock and a versioned marker.
//! A parent lock serializes creation, sweeping and removal, including the interval
//! between creating a directory and installing its ownership marker.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const PARENT: &str = "zircon-native-staging-v1";
const OWNER: &str = "owner.lock";
const LIFECYCLE: &str = "lifecycle.lock";
const MARKER: &str = "zircon-native-staging-v1\n";

#[derive(Debug)]
pub(super) struct StagingDirectory {
    root: PathBuf,
    owner: Option<File>,
}

impl StagingDirectory {
    pub(super) fn create(temporary: &Path) -> io::Result<Self> {
        check_ancestors(temporary)?;
        let parent = temporary.join(PARENT);
        match fs::create_dir(&parent) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        check_ancestors(&parent)?;
        let lifecycle = lifecycle_lock(&parent)?;
        lifecycle.lock()?;
        sweep(&parent)?;
        let name = uuid::Uuid::new_v4().to_string();
        let root = parent.join(&name);
        fs::create_dir(&root)?;
        let mut owner = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(root.join(OWNER))?;
        owner.lock()?;
        owner.write_all(format!("{MARKER}{name}\n").as_bytes())?;
        owner.sync_all()?;
        Ok(Self {
            root,
            owner: Some(owner),
        })
    }

    pub(super) fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for StagingDirectory {
    fn drop(&mut self) {
        // If another operation owns the parent, release our lease and let its
        // next sweep recover this directory. Never wait while unloading a DLL.
        if self.owner.is_none() {
            return;
        }
        let Some(parent) = self.root.parent() else {
            return;
        };
        let Ok(lifecycle) = lifecycle_lock(parent) else {
            return;
        };
        if lifecycle.try_lock().is_err() || !owned_files_only(&self.root) {
            return;
        }
        self.owner.take();
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn lifecycle_lock(parent: &Path) -> io::Result<File> {
    check_ancestors(parent)?;
    let path = parent.join(LIFECYCLE);
    if path.try_exists()? {
        check_plain_file(&path)?;
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)?;
    check_plain_file(&path)?;
    Ok(file)
}

/// Only called while holding the parent lifecycle lock. Missing, malformed or
/// locked markers are retained; legacy zircon-native-* roots are never scanned.
fn sweep(parent: &Path) -> io::Result<()> {
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let root = entry.path();
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if uuid::Uuid::parse_str(&name).is_err() || !owned_files_only(&root) {
            continue;
        }
        let Ok(mut owner) = OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join(OWNER))
        else {
            continue;
        };
        if owner.try_lock().is_err() {
            continue;
        }
        let mut marker = String::new();
        if Read::by_ref(&mut owner)
            .take(128)
            .read_to_string(&mut marker)
            .is_err()
            || marker != format!("{MARKER}{name}\n")
            || !owned_files_only(&root)
        {
            continue;
        }
        // The parent excludes new owners until removal finishes. Close the
        // marker for Windows; no live generation could have yielded this lock.
        drop(owner);
        let _ = fs::remove_dir_all(&root);
    }
    Ok(())
}

fn owned_files_only(root: &Path) -> bool {
    if check_ancestors(root).is_err() || check_plain_file(&root.join(OWNER)).is_err() {
        return false;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return false;
    };
    entries.into_iter().all(|entry| {
        let Ok(entry) = entry else {
            return false;
        };
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return false;
        };
        (name == OWNER || name.to_ascii_lowercase().ends_with(".dll"))
            && check_plain_file(&entry.path()).is_ok()
    })
}

fn check_ancestors(path: &Path) -> io::Result<()> {
    for ancestor in path.ancestors() {
        let metadata = fs::symlink_metadata(ancestor)?;
        if !metadata.is_dir() || reparse(&metadata) {
            return Err(io::Error::other(
                "native staging directory contains a reparse path",
            ));
        }
    }
    Ok(())
}

fn check_plain_file(path: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || reparse(&metadata) {
        return Err(io::Error::other("native staging file is not a plain owned file"));
    }
    Ok(())
}

fn reparse(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

#[cfg(test)]
#[path = "tests/ownership.rs"]
mod tests;
