use super::*;
use std::{fs, path::PathBuf};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "hub-regular-file-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn regular_file_read_enforces_exact_limit_and_rejects_directories() {
    let fixture = Fixture::new();
    let path = fixture.0.join("secret");
    fs::write(&path, b"key!").unwrap();
    assert_eq!(read_bounded_regular(&path, 4).unwrap(), b"key!");
    assert!(read_bounded_regular(&path, 3).is_err());
    assert!(read_bounded_regular(&fixture.0, 4).is_err());
}

#[cfg(windows)]
#[test]
fn windows_read_holds_checked_file_and_parent_against_replacement() {
    let fixture = Fixture::new();
    let parent = fixture.0.join("keys");
    fs::create_dir(&parent).unwrap();
    let path = parent.join("secret");
    fs::write(&path, b"key!").unwrap();
    let opened = open_regular(&path).unwrap();
    assert!(fs::rename(&path, parent.join("replacement")).is_err());
    assert!(fs::OpenOptions::new().write(true).open(&path).is_err());
    assert!(fs::rename(&parent, fixture.0.join("moved")).is_err());
    assert_eq!(read_bounded_regular(&path, 4).unwrap(), b"key!");
    drop(opened);
    fs::write(&path, b"next").unwrap();
}

#[cfg(unix)]
#[test]
fn unix_read_rejects_a_linked_parent() {
    let fixture = Fixture::new();
    let parent = fixture.0.join("keys");
    fs::create_dir(&parent).unwrap();
    fs::write(parent.join("secret"), b"key!").unwrap();
    let link = fixture.0.join("linked");
    std::os::unix::fs::symlink(&parent, &link).unwrap();
    assert!(read_bounded_regular(&link.join("secret"), 4).is_err());
}

#[cfg(unix)]
#[test]
fn unix_read_rejects_a_fifo_without_waiting_for_a_writer() {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let fixture = Fixture::new();
    let fifo = fixture.0.join("fifo");
    let name = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert!(read_bounded_regular(&fifo, 4).is_err());
}
