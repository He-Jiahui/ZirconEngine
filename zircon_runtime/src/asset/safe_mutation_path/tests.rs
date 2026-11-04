use super::*;
use std::os::unix::fs::symlink;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon-u09-path-{}-{}",
            std::process::id(),
            crate::asset::AssetUuid::new()
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn folder_create_and_delete_use_regular_parents_and_preserve_nonempty_contents() {
    let fixture = Fixture::new();
    let folder = fixture.0.join("folder");
    create_directory(&fixture.0, &folder)
        .unwrap()
        .ensure_durable()
        .unwrap();
    fs::write(folder.join("asset.json"), b"asset").unwrap();
    assert!(remove_empty_directory(&fixture.0, &folder).is_err());
    assert_eq!(
        read_regular_file(&fixture.0, &folder.join("asset.json")).unwrap(),
        b"asset"
    );
    fs::remove_file(folder.join("asset.json")).unwrap();
    remove_empty_directory(&fixture.0, &folder)
        .unwrap()
        .ensure_durable()
        .unwrap();
    assert!(!folder.exists());
}

#[test]
fn folder_operations_refuse_directory_file_and_dangling_symlinks() {
    let fixture = Fixture::new();
    let outside = Fixture::new();
    fs::write(outside.0.join("source"), b"outside").unwrap();
    symlink(&outside.0, fixture.0.join("linked")).unwrap();
    symlink(outside.0.join("source"), fixture.0.join("file")).unwrap();
    symlink(outside.0.join("absent"), fixture.0.join("dangling")).unwrap();
    assert!(create_directory(&fixture.0, &fixture.0.join("linked/new")).is_err());
    assert!(remove_empty_directory(&fixture.0, &fixture.0.join("linked")).is_err());
    assert!(read_regular_file(&fixture.0, &fixture.0.join("file")).is_err());
    assert!(validate_no_follow_path(&fixture.0, &fixture.0.join("dangling"), true).is_err());
    assert!(read_directory(&fixture.0, &fixture.0).is_err());
    assert_eq!(
        read_directory_entries(&fixture.0, &fixture.0)
            .unwrap()
            .len(),
        3
    );
    assert!(!outside.0.join("new").exists());
    assert_eq!(fs::read(outside.0.join("source")).unwrap(), b"outside");
}

#[test]
fn folder_lease_detects_parent_detachment_and_replacement() {
    let fixture = Fixture::new();
    let folder = fixture.0.join("parent");
    fs::create_dir(&folder).unwrap();
    let lease = DirectoryLease::open(&fixture.0, &folder).unwrap();
    fs::rename(&folder, fixture.0.join("detached")).unwrap();
    fs::create_dir(&folder).unwrap();
    assert!(lease.verify().is_err());
    assert!(fs::read_dir(&folder).unwrap().next().is_none());
}

#[test]
fn asset_read_respects_sidecar_limit_without_following_links() {
    let fixture = Fixture::new();
    let source = fixture.0.join("source.zmeta");
    fs::write(&source, b"12345").unwrap();
    assert!(read_regular_file_limited(&fixture.0, &source, 4).is_err());
    assert_eq!(
        read_regular_file_limited(&fixture.0, &source, 5).unwrap(),
        b"12345"
    );
}

#[test]
fn folder_inventory_refuses_excess_entries_without_mutation() {
    let fixture = Fixture::new();
    for name in ["one", "two", "three"] {
        fs::create_dir(fixture.0.join(name)).unwrap();
    }
    assert!(read_directory_with_limit(&fixture.0, &fixture.0, 2).is_err());
    assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 3);
}

#[test]
fn folder_mutation_rejects_nonportable_target_components() {
    let fixture = Fixture::new();
    for name in [
        "NUL.txt",
        "CONIN$",
        "COM¹",
        "unsafe:C",
        "tail.",
        "tail ",
        "a\\b",
        "part#label",
    ] {
        assert!(
            create_directory(&fixture.0, &fixture.0.join(name)).is_err(),
            "{name}"
        );
    }
    assert!(fs::read_dir(&fixture.0).unwrap().next().is_none());
}
