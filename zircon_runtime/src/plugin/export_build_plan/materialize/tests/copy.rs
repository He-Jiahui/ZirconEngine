use std::fs::{self, File};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    copy_file_if_changed, copy_native_dynamic_package_files, open_native_source_file_with_hook,
    validate_native_dynamic_package_file_entries, NativeDynamicPackageFileEntry,
};
use crate::plugin::native::NativePluginArtifactDigest;

#[test]
#[cfg(any(windows, target_os = "linux"))]
fn native_inventory_rejects_parent_replacement_between_classification_and_open() {
    let root = temporary_test_root();
    let package = root.join("package");
    let member = package.join("assets/member");
    let outside = root.join("outside");
    fs::create_dir_all(&member).expect("create admitted package resource");
    fs::create_dir_all(&outside).expect("create outside resource");
    fs::write(member.join("data.bin"), b"admitted bytes").unwrap();
    fs::write(outside.join("data.bin"), b"outside bytes").unwrap();
    let source = member.join("data.bin");
    assert!(fs::symlink_metadata(&source).unwrap().is_file());

    let opened = open_native_source_file_with_hook(&source, &package, || {
        fs::rename(&member, package.join("assets/original")).unwrap();
        #[cfg(windows)]
        {
            let output = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(&member)
                .arg(&outside)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        #[cfg(target_os = "linux")]
        std::os::unix::fs::symlink(&outside, &member).unwrap();
    });
    assert!(
        opened.is_err(),
        "an out-of-package handle must not enter inventory"
    );

    #[cfg(windows)]
    fs::remove_dir(&member).unwrap();
    #[cfg(target_os = "linux")]
    fs::remove_file(&member).unwrap();
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}

#[test]
fn native_inventory_validation_rejects_changed_open_source() {
    let root = temporary_test_root();
    fs::create_dir_all(&root).expect("test root should be created");
    let source = root.join("source.dll");
    fs::write(&source, "before").expect("source fixture should be written");
    let source_file = File::open(&source).expect("source fixture should be opened");
    let source_digest = NativePluginArtifactDigest::capture_file(&source_file, &source)
        .expect("source digest should be captured");
    let entries = [NativeDynamicPackageFileEntry {
        source_path: source.clone(),
        relative_path: "native/source.dll".to_string(),
        source_file,
        source_digest,
    }];

    fs::write(&source, "after!").expect("source fixture should be changed in place");
    let error = validate_native_dynamic_package_file_entries(&entries)
        .expect_err("changed source generation must fail validation");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);

    drop(entries);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn changed_inventory_source_is_rejected_before_existing_destination_is_replaced() {
    let root = temporary_test_root();
    let destination_root = root.join("output");
    let destination = destination_root.join("native").join("source.dll");
    fs::create_dir_all(
        destination
            .parent()
            .expect("destination fixture should have a parent"),
    )
    .expect("destination parent should be created");
    let source = root.join("source.dll");
    fs::write(&source, "trusted").expect("source fixture should be written");
    fs::write(&destination, "previous").expect("destination fixture should be written");
    let source_file = File::open(&source).expect("source fixture should be opened");
    let source_digest = NativePluginArtifactDigest::capture_file(&source_file, &source)
        .expect("source digest should be captured");
    let entries = [NativeDynamicPackageFileEntry {
        source_path: source.clone(),
        relative_path: "native/source.dll".to_string(),
        source_file,
        source_digest,
    }];

    fs::write(&source, "tamper!").expect("source fixture should change after capture");
    let error = copy_native_dynamic_package_files(&entries, &destination_root)
        .expect_err("changed source bytes must not replace the existing destination");

    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        fs::read_to_string(&destination).expect("destination should remain readable"),
        "previous"
    );

    drop(entries);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn later_source_mismatch_preserves_every_existing_destination() {
    let root = temporary_test_root();
    let destination_root = root.join("output");
    fs::create_dir_all(&destination_root).expect("destination root should be created");
    let first_source = root.join("first.dll");
    let second_source = root.join("second.dll");
    fs::write(&first_source, "first-new").expect("first source should be written");
    fs::write(&second_source, "second-ok").expect("second source should be written");
    fs::write(destination_root.join("first.dll"), "first-old")
        .expect("first destination should be written");
    fs::write(destination_root.join("second.dll"), "second-old")
        .expect("second destination should be written");

    let first_file = File::open(&first_source).expect("first source should be opened");
    let second_file = File::open(&second_source).expect("second source should be opened");
    let entries = [
        NativeDynamicPackageFileEntry {
            source_path: first_source.clone(),
            relative_path: "first.dll".to_string(),
            source_digest: NativePluginArtifactDigest::capture_file(&first_file, &first_source)
                .expect("first source digest should be captured"),
            source_file: first_file,
        },
        NativeDynamicPackageFileEntry {
            source_path: second_source.clone(),
            relative_path: "second.dll".to_string(),
            source_digest: NativePluginArtifactDigest::capture_file(&second_file, &second_source)
                .expect("second source digest should be captured"),
            source_file: second_file,
        },
    ];
    fs::write(&second_source, "tampered!").expect("second source should change after capture");

    let error = copy_native_dynamic_package_files(&entries, &destination_root)
        .expect_err("one changed source must reject the complete copy batch");

    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(
        fs::read_to_string(destination_root.join("first.dll"))
            .expect("first destination should remain readable"),
        "first-old"
    );
    assert_eq!(
        fs::read_to_string(destination_root.join("second.dll"))
            .expect("second destination should remain readable"),
        "second-old"
    );

    drop(entries);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn native_file_copy_skips_equal_contents_and_replaces_changed_contents() {
    let root = temporary_test_root();
    fs::create_dir_all(&root).expect("test root should be created");
    let source = root.join("source.dll");
    let destination = root.join("destination.dll");
    fs::write(&source, "stable").expect("source fixture should be written");
    fs::write(&destination, "stable").expect("destination fixture should be written");
    let source_file = File::open(&source).expect("source fixture should be opened");
    let source_digest = NativePluginArtifactDigest::capture_file(&source_file, &source)
        .expect("source digest should be captured");

    let original_permissions = fs::metadata(&destination)
        .expect("destination metadata should be readable")
        .permissions();
    let mut read_only_permissions = original_permissions.clone();
    read_only_permissions.set_readonly(true);
    fs::set_permissions(&destination, read_only_permissions)
        .expect("destination should become read-only");

    assert!(
        !copy_file_if_changed(&source_file, &source_digest, &source, &destination,)
            .expect("equal native contents should not rewrite the destination")
    );

    fs::set_permissions(&destination, original_permissions)
        .expect("destination should become writable again");
    fs::write(&destination, "stale!").expect("destination fixture should be changed");

    assert!(
        copy_file_if_changed(&source_file, &source_digest, &source, &destination,)
            .expect("changed native contents should replace the destination")
    );
    assert_eq!(
        fs::read_to_string(&destination).expect("destination should be readable"),
        "stable"
    );

    drop(source_file);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn native_file_copy_compares_every_bounded_buffer_chunk() {
    let root = temporary_test_root();
    fs::create_dir_all(&root).expect("test root should be created");
    let source = root.join("source.pdb");
    let destination = root.join("destination.pdb");
    let payload = vec![7_u8; 64 * 1024 + 3];
    fs::write(&source, &payload).expect("source fixture should be written");
    fs::write(&destination, &payload).expect("destination fixture should be written");
    let source_file = File::open(&source).expect("source fixture should be opened");
    let source_digest = NativePluginArtifactDigest::capture_file(&source_file, &source)
        .expect("source digest should be captured");

    assert!(
        !copy_file_if_changed(&source_file, &source_digest, &source, &destination,)
            .expect("equal multi-chunk native contents should be skipped")
    );

    let mut changed_destination = payload.clone();
    *changed_destination
        .last_mut()
        .expect("multi-chunk fixture should have a trailing byte") = 9;
    fs::write(&destination, changed_destination).expect("destination fixture should be changed");
    assert!(
        copy_file_if_changed(&source_file, &source_digest, &source, &destination,)
            .expect("a trailing chunk change should replace the destination")
    );
    assert_eq!(
        fs::read(&destination).expect("destination should be readable"),
        payload
    );

    drop(source_file);
    fs::remove_dir_all(root).expect("test root should be removable");
}

fn temporary_test_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-export-native-incremental-{}-{nonce}",
        std::process::id()
    ))
}
