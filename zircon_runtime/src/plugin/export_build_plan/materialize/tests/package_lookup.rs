use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::NativePackageInventory;

#[test]
fn inventory_prefers_direct_packages_and_reuses_nested_manifest_scan() {
    let root = temporary_test_root();
    let direct_rendering = root.join("rendering");
    let nested_rendering = root.join("aliases").join("rendering");
    let nested_audio = root.join("third_party").join("audio");
    let lexical_fallback_audio = root.join("vendor").join("audio");
    write_plugin_manifest(&direct_rendering, "rendering");
    write_plugin_manifest(&nested_rendering, "rendering");
    write_plugin_manifest(&nested_audio, "audio");
    write_plugin_manifest(&lexical_fallback_audio, "audio");

    let selected_package_ids = vec!["rendering".to_owned(), "audio".to_owned()];
    let inventory = NativePackageInventory::build(&root, &selected_package_ids)
        .expect("inventory should scan once");

    assert_eq!(
        inventory.package_dir("rendering"),
        Some(direct_rendering.as_path())
    );
    assert_eq!(inventory.package_dir("audio"), Some(nested_audio.as_path()));
    assert_eq!(inventory.package_dir("missing"), None);

    write_plugin_manifest(&root.join("z_late").join("audio"), "audio");
    assert_eq!(inventory.package_dir("audio"), Some(nested_audio.as_path()));

    drop(inventory);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn nested_ineligible_duplicate_does_not_reject_already_resolved_direct_package() {
    let root = temporary_test_root();
    let direct = root.join("foo");
    let duplicate = root.join("aliases/foo");
    let nested_bar = root.join("vendor/deep/bar");
    write_plugin_manifest(&direct, "foo");
    write_plugin_manifest(&nested_bar, "bar");
    fs::create_dir_all(&duplicate).expect("create duplicate package directory");
    fs::write(
        duplicate.join("plugin.toml"),
        "id = \"foo\"\nversion = \"0.1.0\"\ndisplay_name = \"Foo fixture\"\npackage_role = \"test_fixture\"\n",
    )
    .expect("write ineligible duplicate");

    let inventory = NativePackageInventory::build(&root, &["foo".into(), "bar".into()])
        .expect("only still-unresolved selections need role admission");
    assert_eq!(inventory.package_dir("foo"), Some(direct.as_path()));
    assert_eq!(inventory.package_dir("bar"), Some(nested_bar.as_path()));

    drop(inventory);
    fs::remove_dir_all(root).expect("remove owned fixture directory");
}

#[test]
fn inventory_skips_unrelated_manifest_errors_when_direct_selection_resolves() {
    let root = temporary_test_root();
    let direct_rendering = root.join("rendering");
    write_plugin_manifest(&direct_rendering, "rendering");
    let unrelated_manifest = root.join("third_party").join("plugin.toml");
    fs::create_dir_all(
        unrelated_manifest
            .parent()
            .expect("fixture parent should exist"),
    )
    .expect("fixture directory should be created");
    fs::write(&unrelated_manifest, [0xff]).expect("fixture invalid manifest should be written");

    let selected_package_ids = vec!["rendering".to_owned()];
    let inventory = NativePackageInventory::build(&root, &selected_package_ids)
        .expect("direct selection should not read unrelated manifests");

    assert_eq!(
        inventory.package_dir("rendering"),
        Some(direct_rendering.as_path())
    );

    drop(inventory);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn selected_carriers_cannot_enter_product_native_export_inventory() {
    for (role, nested, eligible) in [
        ("production", false, true),
        ("developer_tool", true, true),
        ("sample", false, false),
        ("test_fixture", true, false),
    ] {
        let root = temporary_test_root();
        let package = if nested {
            root.join("third_party").join("carrier")
        } else {
            root.join("carrier")
        };
        fs::create_dir_all(&package).expect("create package directory");
        fs::write(
            package.join("plugin.toml"),
            format!(
                "id = \"carrier\"\nversion = \"0.1.0\"\ndisplay_name = \"Carrier\"\npackage_role = \"{role}\"\n"
            ),
        )
        .expect("write role-bearing package manifest");

        let result = NativePackageInventory::build(&root, &["carrier".to_owned()]);
        if eligible {
            assert_eq!(
                result.unwrap().package_dir("carrier"),
                Some(package.as_path()),
                "selected {role} must remain exportable"
            );
        } else {
            let error = result.err().expect("selected carrier must be rejected");
            assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
            assert!(error
                .to_string()
                .contains("not eligible for product export"));
        }
        fs::remove_dir_all(root).expect("remove owned fixture directory");
    }
}

#[test]
fn inventory_stops_after_all_nested_selections_resolve() {
    let root = temporary_test_root();
    let nested_audio = root.join("a").join("audio");
    let nested_rendering = root.join("b").join("rendering");
    write_plugin_manifest(&nested_audio, "audio");
    write_plugin_manifest(&nested_rendering, "rendering");
    let unrelated_manifest = root.join("z_unrelated").join("plugin.toml");
    fs::create_dir_all(
        unrelated_manifest
            .parent()
            .expect("fixture parent should exist"),
    )
    .expect("fixture directory should be created");
    fs::write(&unrelated_manifest, [0xff]).expect("fixture invalid manifest should be written");

    let selected_package_ids = vec!["audio".to_owned(), "rendering".to_owned()];
    let inventory = NativePackageInventory::build(&root, &selected_package_ids)
        .expect("resolved selections should stop the remaining tree scan");

    assert_eq!(inventory.package_dir("audio"), Some(nested_audio.as_path()));
    assert_eq!(
        inventory.package_dir("rendering"),
        Some(nested_rendering.as_path())
    );

    drop(inventory);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn inventory_searches_inside_direct_selected_packages_for_nested_selections() {
    let root = temporary_test_root();
    let direct_rendering = root.join("rendering");
    let nested_audio = direct_rendering.join("vendor").join("audio");
    write_plugin_manifest(&direct_rendering, "rendering");
    write_plugin_manifest(&nested_audio, "audio");

    let selected_package_ids = vec!["rendering".to_owned(), "audio".to_owned()];
    let inventory = NativePackageInventory::build(&root, &selected_package_ids)
        .expect("nested selections inside direct packages should remain searchable");

    assert_eq!(
        inventory.package_dir("rendering"),
        Some(direct_rendering.as_path())
    );
    assert_eq!(inventory.package_dir("audio"), Some(nested_audio.as_path()));

    drop(inventory);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn inventory_snapshots_selected_native_payload_entries() {
    let root = temporary_test_root();
    let package_dir = root.join("audio");
    write_plugin_manifest(&package_dir, "audio");
    let native_artifact = package_dir.join("native").join("audio.dll");
    let resource = package_dir.join("assets").join("settings.json");
    fs::create_dir_all(
        native_artifact
            .parent()
            .expect("native fixture parent should exist"),
    )
    .expect("native fixture directory should be created");
    fs::create_dir_all(
        resource
            .parent()
            .expect("resource fixture parent should exist"),
    )
    .expect("resource fixture directory should be created");
    fs::write(&native_artifact, "native payload").expect("native fixture should be written");
    fs::write(&resource, "resource payload").expect("resource fixture should be written");

    let selected_package_ids = vec!["audio".to_owned()];
    let inventory = NativePackageInventory::build(&root, &selected_package_ids)
        .expect("inventory should snapshot the selected package payload");

    fs::write(package_dir.join("native").join("late.dll"), "late payload")
        .expect("late native fixture should be written");
    let payload = inventory
        .file_inventory("audio")
        .expect("selected package should retain its frozen payload inventory");
    let relative_paths = payload
        .entries
        .iter()
        .map(|entry| entry.relative_path.as_str())
        .collect::<Vec<_>>();

    assert_eq!(relative_paths, ["assets/settings.json", "native/audio.dll"]);

    drop(inventory);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn hash_resolution_sets_preserve_lexical_fallback_order() {
    let root = temporary_test_root();
    let lexical_first = root.join("a").join("audio");
    let lexical_last = root.join("z").join("audio");
    write_plugin_manifest(&lexical_first, "audio");
    write_plugin_manifest(&lexical_last, "audio");

    let inventory = NativePackageInventory::build(&root, &["audio".to_owned()])
        .expect("nested selection should resolve");

    assert_eq!(
        inventory.package_dir("audio"),
        Some(lexical_first.as_path())
    );

    drop(inventory);
    fs::remove_dir_all(root).expect("test root should be removable");
}

#[test]
fn hash_resolution_sets_deduplicate_selected_package_ids() {
    let root = temporary_test_root();
    let direct_audio = root.join("audio");
    write_plugin_manifest(&direct_audio, "audio");

    let inventory = NativePackageInventory::build(
        &root,
        &["audio".to_owned(), "audio".to_owned(), "audio".to_owned()],
    )
    .expect("duplicate selections should resolve once");

    assert_eq!(inventory.package_dirs.len(), 1);
    assert_eq!(inventory.package_dir("audio"), Some(direct_audio.as_path()));

    drop(inventory);
    fs::remove_dir_all(root).expect("test root should be removable");
}

fn temporary_test_root() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-export-native-package-inventory-{}-{nonce}",
        std::process::id()
    ))
}

fn write_plugin_manifest(directory: &Path, id: &str) {
    fs::create_dir_all(directory).expect("fixture directory should be created");
    fs::write(
        directory.join("plugin.toml"),
        format!("id = {id:?}\nversion = \"0.1.0\"\ndisplay_name = {id:?}\n"),
    )
    .expect("fixture plugin manifest should be written");
}
