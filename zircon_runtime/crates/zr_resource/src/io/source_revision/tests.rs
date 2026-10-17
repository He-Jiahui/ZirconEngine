#![cfg(windows)]

use super::*;

pub(crate) struct Fixture(pub(crate) std::path::PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        let result = fs::remove_dir_all(&self.0);
        if !std::thread::panicking() {
            result.expect("remove the owned revision fixture");
        }
    }
}

pub(crate) fn fixture(label: &str) -> Fixture {
    let base = Path::new(r"E:\cargo-targets\zircon-local");
    // Verify the physical root before creating any test output.
    for ancestor in base.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).expect("physical E output root must exist");
        assert!(metadata.is_dir() && !platform::is_reparse(&metadata));
    }
    let canonical = fs::canonicalize(base).unwrap();
    assert!(canonical
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .eq_ignore_ascii_case(r"E:\cargo-targets\zircon-local"));
    let root = base.join(format!(
        "editor182-observed-{label}-{}-{}-{}",
        std::process::id(),
        crate::io::next_test_output_id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).expect("create a new owned fixture, never reuse an existing path");
    Fixture(root)
}

#[test]
fn same_size_same_mtime_change_is_a_distinct_revision() {
    let fixture = fixture("same-size");
    let path = fixture.0.join("source");
    fs::write(&path, b"AAAA").unwrap();
    let before = SourceSnapshot::read(&path).unwrap();
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    fs::write(&path, b"BBBB").unwrap();
    let file = File::options().write(true).open(&path).unwrap();
    file.set_modified(modified).unwrap();
    drop(file);
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert_eq!(before.bytes(), b"AAAA");
    assert_ne!(before.revision(), &SourceRevision::observe(&path).unwrap());
}

#[test]
fn delete_recreate_with_same_bytes_is_a_distinct_revision() {
    let fixture = fixture("recreated");
    let path = fixture.0.join("source");
    fs::write(&path, b"same source").unwrap();
    let before = SourceRevision::observe(&path).unwrap();
    fs::remove_file(&path).unwrap();
    fs::write(&path, b"same source").unwrap();
    assert_ne!(before, SourceRevision::observe(&path).unwrap());
}

#[test]
fn reparse_source_is_rejected_without_reading_its_target() {
    let fixture = fixture("reparse");
    let target = fixture.0.join("target");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("scene"), b"outside source").unwrap();
    let file_link = fixture.0.join("file-link");
    std::os::windows::fs::symlink_file(target.join("scene"), &file_link)
        .expect("real native file symlink capability is required, never skip this regression");
    assert_eq!(
        SourceSnapshot::read(&file_link).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    let directory_link = fixture.0.join("directory-link");
    std::os::windows::fs::symlink_dir(&target, &directory_link)
        .expect("real native directory symlink capability is required, never skip this regression");
    assert_eq!(
        SourceSnapshot::read(&directory_link.join("scene"))
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(
        SourceRevision::observe(&directory_link.join("missing/scene"))
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(fs::read(target.join("scene")).unwrap(), b"outside source");
    fs::remove_file(file_link).unwrap();
    fs::remove_dir(directory_link).unwrap();
}
