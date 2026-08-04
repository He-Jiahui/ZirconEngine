use super::*;
use crate::asset::migration::scan_migration_inventory_for_test;
use std::path::PathBuf;
use zircon_runtime_interface::project::RelPath;

#[test]
fn migration_run_has_one_inventory_walk_owner() {
    const RUN_SOURCE: &str = include_str!("../../../migration/run.rs");
    const SCAN_SOURCE: &str = include_str!("../../../migration/scan.rs");
    const SIDECAR_SOURCE: &str = include_str!("../../../migration/sidecar.rs");

    assert_eq!(RUN_SOURCE.matches("MigrationInventory::build(").count(), 2);
    for retired_call in [
        "recognized_sources(&root_paths)",
        "supported_authoring_files(&root_paths)",
        "supported_transaction_targets(&root_paths)",
        "prospective_sidecar_targets(&recognized)",
    ] {
        assert!(!RUN_SOURCE.contains(retired_call));
    }
    assert_eq!(SCAN_SOURCE.matches("fs::read_dir(").count(), 1);
    assert!(SCAN_SOURCE.contains(".binary_search_by(|identity| identity.path.as_path().cmp(path))"));
    assert!(!SIDECAR_SOURCE.contains("fs::read_dir("));
    assert!(!SIDECAR_SOURCE.contains(".exists()"));
    assert!(SIDECAR_SOURCE.contains("inventory.is_rejected_path"));
    assert!(SCAN_SOURCE.contains("resolver_projections"));
    assert!(SIDECAR_SOURCE.contains("compound_bindings"));
    assert_eq!(
        RUN_SOURCE.matches("MigrationResolverIndex::build(").count(),
        1
    );
    assert!(RUN_SOURCE.contains("MigrationResolver::new(&sidecars.index, &resolver_index)"));
}

#[test]
fn migration_run_republishes_the_inventory_after_apply_mode_recovery() {
    const RUN_SOURCE: &str = include_str!("../../../migration/run.rs");

    let recovery = RUN_SOURCE
        .find("recover_pending_transactions(paths.root(), &root_paths, &recovery_targets)?;")
        .expect("apply-mode pending recovery must complete before publishing the final inventory");
    let republish = RUN_SOURCE
        .find("let inventory = if pending_recovery.is_empty() || options.mode == AssetMigrationMode::DryRun {")
        .expect("migration run must choose a post-recovery inventory generation");
    let preflight = RUN_SOURCE
        .find("preflight_sidecars(&root_paths, &inventory)")
        .expect("sidecar preflight must consume the selected inventory generation");

    assert!(recovery < republish);
    assert!(republish < preflight);
    assert!(RUN_SOURCE[republish..preflight].contains("MigrationInventory::build(&roots)"));
}

#[test]
fn migration_run_keeps_pending_recovery_available_after_dry_run_reporting() {
    const RUN_SOURCE: &str = include_str!("../../../migration/run.rs");

    assert!(
        RUN_SOURCE.contains("for journal in &pending_recovery {"),
        "dry-run recovery reporting must borrow journals because generation selection reads them later"
    );
    assert!(RUN_SOURCE.contains("let inventory = if pending_recovery.is_empty()"));
}

#[test]
fn migration_inventory_walks_overlapping_roots_once_and_classifies_files() {
    let root = fixture_root("migration-single-inventory");
    let assets = root.join("assets");
    let assets_alias = assets.join(".");
    let nested = assets.join("nested");
    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(assets.join(".zircon")).unwrap();

    let authoring = assets.join("level.scene.toml");
    let texture_source = nested.join("texture.png");
    let standalone_shader = nested.join("standalone.wgsl");
    let current_sidecar = nested.join("texture.png.zmeta");
    let retired_sidecar = nested.join("texture.png.meta.toml");
    let orphan_sidecar = nested.join("orphan.asset.zmeta");
    let orphan_counterpart = nested.join("orphan.asset.meta.toml");
    let prospective_sidecar = nested.join("standalone.wgsl.zmeta");
    fs::write(&authoring, "scene = true\n").unwrap();
    fs::write(&texture_source, b"png fixture").unwrap();
    fs::write(
        &standalone_shader,
        "@compute @workgroup_size(1) fn main() {}\n",
    )
    .unwrap();
    fs::write(&current_sidecar, "format_version = 7\n").unwrap();
    fs::write(&retired_sidecar, "version = 6\n").unwrap();
    fs::write(&orphan_sidecar, "format_version = 7\n").unwrap();
    fs::write(
        assets.join(".zircon/ignored.scene.toml"),
        "ignored = true\n",
    )
    .unwrap();

    let nested_root = nested_root_with_distinct_lexical_case(&root, &nested);
    let snapshot =
        scan_migration_inventory_for_test(&[assets.clone(), assets_alias, nested_root]).unwrap();

    assert_eq!(snapshot.directory_visits, 2);
    assert_eq!(snapshot.directory_sorts, 2);
    assert_eq!(snapshot.file_visits, 6);
    assert_eq!(snapshot.authoring_files, vec![authoring.clone()]);
    let mut expected_sidecar_candidates = vec![
        current_sidecar.clone(),
        retired_sidecar.clone(),
        orphan_sidecar.clone(),
    ];
    expected_sidecar_candidates.sort();
    assert_eq!(snapshot.sidecar_candidates, expected_sidecar_candidates);
    let standalone_relative_paths = snapshot
        .physical_relative_paths
        .iter()
        .find_map(|(path, relative_paths)| (path == &standalone_shader).then_some(relative_paths))
        .expect("standalone source must publish root-relative physical identities");
    assert_eq!(
        standalone_relative_paths,
        &vec![
            PathBuf::from("nested/standalone.wgsl"),
            PathBuf::from("nested/standalone.wgsl"),
            PathBuf::from("standalone.wgsl"),
        ]
    );
    let standalone_logical_roots = snapshot
        .logical_root_identities
        .iter()
        .find_map(|(path, logical_roots)| (path == &standalone_shader).then_some(logical_roots))
        .expect("standalone source must retain every logical root identity");
    assert_eq!(
        standalone_logical_roots,
        &vec![
            RelPath::parse("test-root-0").unwrap(),
            RelPath::parse("test-root-1").unwrap(),
            RelPath::parse("test-root-2").unwrap(),
        ]
    );
    assert!(snapshot.transaction_targets.contains(&authoring));
    assert!(snapshot.transaction_targets.contains(&current_sidecar));
    assert!(snapshot.transaction_targets.contains(&retired_sidecar));
    assert!(snapshot.transaction_targets.contains(&orphan_sidecar));
    assert!(snapshot.transaction_targets.contains(&orphan_counterpart));
    assert!(snapshot.transaction_targets.contains(&prospective_sidecar));
    assert!(!snapshot
        .transaction_targets
        .contains(&assets.join(".zircon/ignored.scene.toml")));

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn migration_inventory_skips_missing_roots_without_rescanning_existing_roots() {
    let root = fixture_root("migration-missing-asset-root");
    let assets = root.join("assets");
    let missing = root.join("optional-assets");
    fs::create_dir_all(&assets).unwrap();
    let authoring = assets.join("level.scene.toml");
    fs::write(&authoring, "scene = true\n").unwrap();

    let snapshot = scan_migration_inventory_for_test(&[missing, assets.clone()]).unwrap();

    assert_eq!(snapshot.directory_visits, 1);
    assert_eq!(snapshot.directory_sorts, 1);
    assert_eq!(snapshot.file_visits, 1);
    assert_eq!(snapshot.authoring_files, vec![authoring]);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn migration_inventory_rejects_reparse_asset_roots_without_visiting_their_target() {
    let root = fixture_root("migration-reparse-asset-root");
    let outside = fixture_root("migration-reparse-asset-root-outside");
    let linked_assets = root.join("assets");
    let outside_source = outside.join("must-not-scan.scene.toml");
    fs::write(&outside_source, "outside = true\n").unwrap();
    if !create_directory_link(&outside, &linked_assets) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
        return;
    }

    let error = scan_migration_inventory_for_test(&[linked_assets.clone()]).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert_eq!(
        fs::read_to_string(&outside_source).unwrap(),
        "outside = true\n"
    );
    remove_directory_link(&linked_assets);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn migration_commandlet_reports_reparse_asset_roots_as_scan_errors() {
    let root = fixture_root("migration-commandlet-reparse-asset-root");
    let outside = fixture_root("migration-commandlet-reparse-asset-root-outside");
    write_manifest(&root, &["assets"]);
    let linked_assets = root.join("assets");
    let shader_guid: AssetUuid = "aa111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &outside,
        "",
        "shaders/pbr.zshader",
        shader_guid,
        AssetKind::Shader,
    );
    let outside_source = outside.join("materials/must-not-migrate.zmaterial");
    fs::create_dir_all(outside_source.parent().unwrap()).unwrap();
    let original = format!(
        "version = 2\n\n[shader]\nuuid = \"{shader_guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
    );
    fs::write(&outside_source, &original).unwrap();
    if !create_directory_link(&outside, &linked_assets) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
        return;
    }

    let error =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap_err();

    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::Scan { source, .. }
            if source.kind() == std::io::ErrorKind::InvalidInput
    ));
    assert_eq!(fs::read_to_string(&outside_source).unwrap(), original);
    remove_directory_link(&linked_assets);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[cfg(windows)]
fn nested_root_with_distinct_lexical_case(
    root: &std::path::Path,
    _nested: &std::path::Path,
) -> PathBuf {
    root.join("ASSETS").join("NESTED")
}

#[cfg(not(windows))]
fn nested_root_with_distinct_lexical_case(
    _root: &std::path::Path,
    nested: &std::path::Path,
) -> PathBuf {
    nested.to_path_buf()
}

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
fn linked_current_sidecar_cannot_become_retired_preflight_authority() {
    let root = fixture_root("migration-current-sidecar-link");
    let outside = fixture_root("migration-current-sidecar-link-outside");
    write_manifest(&root, &["assets"]);
    let assets = root.join("assets/textures");
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("hero.png"), b"png fixture").unwrap();
    fs::write(
        assets.join("hero.png.meta.toml"),
        "format_version = 6\nuuid = \"aa111111-2222-4333-8444-555555555555\"\nurl = \"res://textures/hero.png\"\nasset_kind = \"Texture\"\nsource_hash = \"legacy-digest\"\n",
    )
    .unwrap();
    let outside_sidecar = outside.join("hero.png.zmeta");
    let outside_source = "format_version = 7\nuuid = \"aa111111-2222-4333-8444-555555555555\"\nurl = \"res://textures/hero.png\"\nasset_kind = \"Texture\"\nsource_digest = \"outside-digest\"\n";
    fs::write(&outside_sidecar, outside_source).unwrap();
    let linked_sidecar = assets.join("hero.png.zmeta");
    if !create_file_link(&outside_sidecar, &linked_sidecar) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
        return;
    }

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();

    assert!(!report.succeeded());
    assert_eq!(report.issues().len(), 1);
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert_eq!(
        fs::read_to_string(&outside_sidecar).unwrap(),
        outside_source
    );
    remove_file_link(&linked_sidecar);
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn linked_retired_sidecar_cannot_suppress_current_sidecar_minting() {
    let root = fixture_root("migration-retired-sidecar-link");
    let outside = fixture_root("migration-retired-sidecar-link-outside");
    write_manifest(&root, &["assets"]);
    let assets = root.join("assets/textures");
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("hero.png"), b"png fixture").unwrap();
    let outside_sidecar = outside.join("hero.png.meta.toml");
    let outside_source = "format_version = 6\nuuid = \"ab111111-2222-4333-8444-555555555555\"\nurl = \"res://textures/hero.png\"\nasset_kind = \"Texture\"\nsource_hash = \"outside-digest\"\n";
    fs::write(&outside_sidecar, outside_source).unwrap();
    let linked_sidecar = assets.join("hero.png.meta.toml");
    if !create_file_link(&outside_sidecar, &linked_sidecar) {
        fs::remove_dir_all(root).unwrap();
        fs::remove_dir_all(outside).unwrap();
        return;
    }

    let report = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();

    assert!(!report.succeeded());
    assert_eq!(report.issues().len(), 1);
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert_eq!(
        fs::read_to_string(&outside_sidecar).unwrap(),
        outside_source
    );
    remove_file_link(&linked_sidecar);
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
const WINDOWS_ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

#[cfg(windows)]
fn create_file_link(target: &std::path::Path, link: &std::path::Path) -> bool {
    match std::os::windows::fs::symlink_file(target, link) {
        Ok(()) => true,
        Err(error)
            if error.kind() == std::io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(WINDOWS_ERROR_PRIVILEGE_NOT_HELD) =>
        {
            false
        }
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
        Err(error)
            if error.kind() == std::io::ErrorKind::PermissionDenied
                || error.raw_os_error() == Some(WINDOWS_ERROR_PRIVILEGE_NOT_HELD) =>
        {
            false
        }
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
