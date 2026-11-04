use std::fs::{self, File, Metadata, OpenOptions};
use std::io;
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

fn open(path: &Path, directory: bool) -> io::Result<File> {
    // No delete sharing pins every directory in the chain for the operation's lifetime.
    let file = OpenOptions::new()
        .read(true)
        .share_mode(0x0001 | 0x0002)
        .custom_flags(0x0020_0000 | if directory { 0x0200_0000 } else { 0 })
        .open(path)?;
    let metadata = file.metadata()?;
    if super::super::safe_project_path::is_link_or_reparse(&metadata)
        || metadata.is_dir() != directory
        || (!directory && !metadata.is_file())
    {
        return Err(super::unsafe_path(path));
    }
    Ok(file)
}

pub(super) fn open_directory(_: Option<&File>, path: &Path) -> io::Result<File> {
    open(path, true)
}
pub(super) fn open_file(_: &File, path: &Path) -> io::Result<File> {
    open(path, false)
}
pub(super) fn create_directory(_: &File, path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}
pub(super) fn remove_directory(_: &File, path: &Path) -> io::Result<()> {
    fs::remove_dir(path)
}
pub(super) fn directory_path(_: &File, path: &Path) -> PathBuf {
    path.to_path_buf()
}
pub(super) fn same_identity(a: &Metadata, b: &Metadata) -> bool {
    // Existing no-delete-sharing handles prevent replacement while the chain is retained.
    a.is_dir() == b.is_dir() && a.created().ok() == b.created().ok()
}
pub(super) fn same_file(a: &Metadata, b: &Metadata) -> bool {
    same_identity(a, b) && a.len() == b.len() && a.modified().ok() == b.modified().ok()
}

pub(super) fn sync_directory(_: &File) -> io::Result<()> {
    // Windows does not expose POSIX directory fsync through these directory handles.
    // Match the existing durable file transaction's parent-sync behavior on Windows.
    Ok(())
}
