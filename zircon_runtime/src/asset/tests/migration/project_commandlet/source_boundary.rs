use super::*;

#[test]
fn paired_source_link_is_not_followed_into_sidecar_preflight() {
    let root = fixture_root("migration-sidecar-source-link");
    let outside = fixture_root("migration-sidecar-source-link-outside");
    write_manifest(&root, &["assets"]);
    let assets = root.join("assets/data");
    fs::create_dir_all(&assets).unwrap();
    let outside_source = outside.join("escaped.asset");
    fs::write(&outside_source, "outside source").unwrap();
    let linked_source = assets.join("linked.asset");
    if !create_file_link(&outside_source, &linked_source) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
        return;
    }
    let sidecar = assets.join("linked.asset.zmeta");
    let source = "format_version = 7\nuuid = \"ff111111-2222-4333-8444-555555555555\"\nurl = \"res://data/linked.asset\"\nasset_kind = \"Data\"\nsource_digest = \"digest\"\n";
    fs::write(&sidecar, source).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();

    assert!(report.succeeded());
    assert!(report.changed_files().is_empty());
    assert_eq!(fs::read_to_string(&sidecar).unwrap(), source);
    assert_eq!(
        fs::read_to_string(&outside_source).unwrap(),
        "outside source"
    );
    remove_file_link(&linked_source);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn linked_journal_directory_cannot_escape_project_owner() {
    let root = fixture_root("migration-journal-directory-link");
    let outside = fixture_root("migration-journal-directory-link-outside");
    write_manifest(&root, &["assets"]);
    fs::create_dir_all(root.join("assets")).unwrap();
    let victim = outside.join("victim.toml");
    fs::write(&victim, "outside = true\n").unwrap();
    fs::create_dir_all(root.join(".zircon")).unwrap();
    let linked = root.join(".zircon/asset-migration");
    if !create_directory_link(&outside, &linked) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
        return;
    }

    let error =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap_err();

    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::InvalidJournal { .. }
    ));
    assert_eq!(fs::read_to_string(&victim).unwrap(), "outside = true\n");
    remove_directory_link(&linked);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn linked_zircon_owner_is_rejected_even_without_migration_subdirectory() {
    let root = fixture_root("migration-owner-link");
    let outside = fixture_root("migration-owner-link-outside");
    write_manifest(&root, &["assets"]);
    fs::create_dir_all(root.join("assets")).unwrap();
    let linked = root.join(".zircon");
    if !create_directory_link(&outside, &linked) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
        return;
    }
    let error =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap_err();
    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::InvalidJournal { .. }
    ));
    assert!(!outside.join("asset-migration").exists());
    remove_directory_link(&linked);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn linked_journal_file_is_rejected_without_touching_external_target() {
    let root = fixture_root("migration-journal-file-link");
    let outside = fixture_root("migration-journal-file-link-outside");
    write_manifest(&root, &["assets"]);
    fs::create_dir_all(root.join("assets")).unwrap();
    let journal_directory = root.join(".zircon/asset-migration");
    fs::create_dir_all(&journal_directory).unwrap();
    let victim = outside.join("victim.toml");
    fs::write(&victim, "outside = true\n").unwrap();
    let linked = journal_directory.join("linked.toml");
    if !create_file_link(&victim, &linked) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
        return;
    }

    let error =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap_err();

    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::InvalidJournal { .. }
    ));
    assert_eq!(fs::read_to_string(&victim).unwrap(), "outside = true\n");
    remove_file_link(&linked);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[cfg(unix)]
fn create_file_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
fn create_file_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    match std::os::windows::fs::symlink_file(target, link) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => false,
        Err(error) => panic!("create file reparse fixture failed: {error}"),
    }
}

#[cfg(unix)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
fn create_directory_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    match std::os::windows::fs::symlink_dir(target, link) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => false,
        Err(error) => panic!("create directory reparse fixture failed: {error}"),
    }
}

fn remove_file_link(link: &std::path::Path) {
    fs::remove_file(link).unwrap();
}

#[cfg(unix)]
fn remove_directory_link(link: &std::path::Path) {
    fs::remove_file(link).unwrap();
}

#[cfg(windows)]
fn remove_directory_link(link: &std::path::Path) {
    fs::remove_dir(link).unwrap();
}
