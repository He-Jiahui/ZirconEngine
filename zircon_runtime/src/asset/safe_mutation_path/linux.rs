use std::ffi::CString;
use std::fs::{File, Metadata};
use std::io;
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

fn name(path: &Path, relative: bool) -> io::Result<CString> {
    let name = if relative {
        path.file_name().ok_or_else(|| super::unsafe_path(path))?
    } else {
        path.as_os_str()
    };
    CString::new(name.as_bytes()).map_err(|_| super::unsafe_path(path))
}

pub(super) fn open_directory(parent: Option<&File>, path: &Path) -> io::Result<File> {
    open(parent, path, libc::O_RDONLY | libc::O_DIRECTORY)
}

pub(super) fn open_file(parent: &File, path: &Path) -> io::Result<File> {
    open(Some(parent), path, libc::O_RDONLY | libc::O_NONBLOCK)
}

fn open(parent: Option<&File>, path: &Path, flags: i32) -> io::Result<File> {
    let name = name(path, parent.is_some())?;
    // SAFETY: the C string and optional owned parent remain alive. The result is a new descriptor.
    let fd = unsafe {
        libc::openat(
            parent.map_or(libc::AT_FDCWD, AsRawFd::as_raw_fd),
            name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: openat returned an owned descriptor which is transferred to File exactly once.
    Ok(unsafe { File::from_raw_fd(fd) })
}

pub(super) fn create_directory(parent: &File, path: &Path) -> io::Result<()> {
    let name = name(path, true)?;
    // SAFETY: the leased parent and NUL-terminated leaf remain alive during mkdirat.
    if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o755) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(super) fn remove_directory(parent: &File, path: &Path) -> io::Result<()> {
    let name = name(path, true)?;
    // SAFETY: unlinkat addresses the entry in the leased parent, without following a symlink.
    if unsafe { libc::unlinkat(parent.as_raw_fd(), name.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(super) fn directory_path(file: &File, _: &Path) -> PathBuf {
    PathBuf::from(format!("/proc/self/fd/{}", file.as_raw_fd()))
}

pub(super) fn same_identity(a: &Metadata, b: &Metadata) -> bool {
    a.dev() == b.dev() && a.ino() == b.ino()
}

pub(super) fn same_file(a: &Metadata, b: &Metadata) -> bool {
    same_identity(a, b)
        && a.len() == b.len()
        && a.mtime() == b.mtime()
        && a.mtime_nsec() == b.mtime_nsec()
        && a.ctime() == b.ctime()
        && a.ctime_nsec() == b.ctime_nsec()
}

pub(super) fn sync_directory(file: &File) -> io::Result<()> {
    file.sync_all()
}
