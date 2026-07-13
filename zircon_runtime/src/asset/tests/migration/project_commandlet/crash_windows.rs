use super::*;

#[test]
fn minted_sidecar_commit_crash_is_whitelisted_and_next_apply_converges() {
    let root = fixture_root("mint-sidecar-crash");
    write_manifest(&root, &["assets"]);
    let source = root.join("assets/models/hero.glb");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, b"model").unwrap();

    let error = migrate_project_assets_with_commit_window_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        0,
        false,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::Transaction { .. }
    ));

    let recovered =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(recovered.succeeded());
    assert!(source.with_file_name("hero.glb.zmeta").is_file());
    fs::remove_dir_all(root).unwrap();
}

fn write_legacy_material(
    root: &std::path::Path,
    name: &str,
    guid: AssetUuid,
) -> std::path::PathBuf {
    let material = root.join("assets/materials").join(name);
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
        ),
    )
    .unwrap();
    material
}

fn assert_transaction_artifacts_cleared(root: &std::path::Path, owner: &std::path::Path) {
    assert!(!fs::read_dir(owner)
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("zr-migrate")));
    assert_eq!(
        fs::read_dir(root.join(".zircon/asset-migration"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn post_stage_pre_journal_activation_crash_is_detected_and_converges_on_apply() {
    let root = fixture_root("transaction-post-stage-intent");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "fc111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let material = write_legacy_material(&root, "intent.zmaterial", guid);
    let original = fs::read_to_string(&material).unwrap();

    migrate_project_assets_with_stage_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        0,
        3,
    )
    .expect_err("post-stage interruption must retain its intent journal");
    assert_eq!(fs::read_to_string(&material).unwrap(), original);
    let journal_directory = root.join(".zircon/asset-migration");
    assert!(fs::read_dir(&journal_directory).unwrap().count() > 0);
    let before = directory_snapshot(material.parent().unwrap());

    let dry_run = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert!(!dry_run.succeeded());
    assert!(dry_run
        .issues()
        .iter()
        .any(|issue| issue.kind() == AssetMigrationIssueKind::PendingRecovery));
    assert_eq!(directory_snapshot(material.parent().unwrap()), before);

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(report.succeeded());
    assert!(fs::read_to_string(&material)
        .unwrap()
        .contains("kind = \"project\""));
    assert_transaction_artifacts_cleared(&root, material.parent().unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn target_replace_before_journal_sync_converges_without_backup_restore() {
    let root = fixture_root("transaction-target-replace-window");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "fd111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let material = write_legacy_material(&root, "replace.zmaterial", guid);

    migrate_project_assets_with_commit_window_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        0,
        false,
    )
    .expect_err("target replacement interruption must retain active evidence");
    let replaced = fs::read_to_string(&material).unwrap();
    assert!(replaced.contains("kind = \"project\""));

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(report.succeeded());
    assert_eq!(fs::read_to_string(&material).unwrap(), replaced);
    assert_transaction_artifacts_cleared(&root, material.parent().unwrap());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn sidecar_target_replace_and_retired_delete_crash_windows_converge_forward() {
    for after_retired_delete in [false, true] {
        let root = fixture_root(if after_retired_delete {
            "transaction-retired-delete-window"
        } else {
            "transaction-sidecar-target-window"
        });
        write_manifest(&root, &["assets"]);
        let guid: AssetUuid = "fe111111-2222-4333-8444-555555555555".parse().unwrap();
        let source = root.join("assets/shaders/legacy.zshader");
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source, "shader source").unwrap();
        let retired = root.join("assets/shaders/legacy.zshader.meta.toml");
        fs::write(
            &retired,
            format!(
                "format_version = 6\nuuid = \"{guid}\"\nurl = \"res://shaders/legacy.zshader\"\nasset_kind = \"Shader\"\nsource_hash = \"legacy-digest\"\n"
            ),
        )
        .unwrap();
        let current = root.join("assets/shaders/legacy.zshader.zmeta");
        let material = write_legacy_material(&root, "legacy-user.zmaterial", guid);

        migrate_project_assets_with_commit_window_fault(
            AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
            0,
            after_retired_delete,
        )
        .expect_err("sidecar commit substep interruption must retain active evidence");
        assert!(current.is_file());
        assert_eq!(retired.exists(), !after_retired_delete);

        let report =
            migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
                .unwrap();
        assert!(report.succeeded());
        assert!(current.is_file());
        assert!(!retired.exists());
        let current_text = fs::read_to_string(&current).unwrap();
        let current_meta =
            crate::asset::project::AssetMetaDocument::from_toml_str(&current_text).unwrap();
        assert_eq!(current_meta.uuid, guid);
        assert_eq!(current_meta.source_digest, "legacy-digest");
        let material_text = fs::read_to_string(&material).unwrap();
        let material_value: toml::Value = toml::from_str(&material_text).unwrap();
        assert_eq!(
            material_value["shader"]["guid"].as_str(),
            Some(guid.to_string().as_str())
        );
        assert_eq!(
            material_value["shader"]["path_hint"].as_str(),
            Some("assets/shaders/legacy.zshader")
        );
        assert_transaction_artifacts_cleared(&root, current.parent().unwrap());

        let second =
            migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
                .unwrap();
        assert!(second.succeeded());
        assert!(second.changed_files().is_empty());
        assert_eq!(fs::read_to_string(&current).unwrap(), current_text);
        assert_eq!(fs::read_to_string(&material).unwrap(), material_text);
        fs::remove_dir_all(root).unwrap();
    }
}
