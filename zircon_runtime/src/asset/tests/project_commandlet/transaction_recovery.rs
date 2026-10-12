//! 事务恢复测试把 journal 视为非可信输入；Apply 可恢复，DryRun 仅报告且必须保持现有磁盘快照。
use super::*;

#[test]
fn interrupted_recovery_accepts_valid_multiline_backup_and_converges() {
    let root = fixture_root("transaction-valid-multiline-backup");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "f4111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let shader_directory = root.join("assets/shaders");
    let material = root.join("assets/materials/quoted.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    let original = format!(
        "version = 2\nname = \"\"\"a\"\"\"\"\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
    );
    let original_value: toml::Value = toml::from_str(&original).unwrap();
    assert_eq!(original_value["name"].as_str(), Some("a\""));
    let original_digest = blake3::hash(original.as_bytes());
    fs::write(&material, original.as_bytes()).unwrap();

    // Derive the publication index from the normal plan: registered sidecars can also change.
    let planned = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap();
    assert!(planned.succeeded());
    let material_index = planned
        .changed_files()
        .iter()
        .position(|change| change.path().file_name() == material.file_name())
        .expect("the legacy material must appear in the normal publication plan");

    let interrupted = migrate_project_assets_with_commit_window_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        material_index,
        false,
    )
    .unwrap_err();
    match interrupted {
        crate::asset::migration::AssetMigrationError::Transaction { phase, source, .. } => {
            assert_eq!(phase, AssetMigrationTransactionPhase::Commit);
            assert_eq!(source.kind(), std::io::ErrorKind::Interrupted);
        }
        other => panic!("expected the target-replacement interruption, found {other}"),
    }
    let replaced = fs::read(&material).unwrap();
    assert_ne!(replaced, original.as_bytes());
    let replaced_digest = blake3::hash(&replaced);
    let backups = fs::read_dir(material.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .contains(".zr-migrate-backup-")
        })
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    let backup_bytes = fs::read(&backups[0]).unwrap();
    assert_eq!(backup_bytes, original.as_bytes());
    assert_eq!(blake3::hash(&backup_bytes), original_digest);
    let journal_directory = root.join(".zircon/asset-migration");
    let assets_before = directory_snapshot(&shader_directory);
    let artifacts_before = directory_snapshot(material.parent().unwrap());
    let journals_before = directory_snapshot(&journal_directory);
    assert!(!journals_before.is_empty());

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
    assert_eq!(directory_snapshot(&shader_directory), assets_before);
    assert_eq!(
        directory_snapshot(material.parent().unwrap()),
        artifacts_before
    );
    assert_eq!(directory_snapshot(&journal_directory), journals_before);

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(report.succeeded());
    assert!(report.applied());
    let migrated = fs::read(&material).unwrap();
    assert_eq!(migrated, replaced);
    assert_eq!(blake3::hash(&migrated), replaced_digest);
    let migrated_value: toml::Value =
        toml::from_str(std::str::from_utf8(&migrated).unwrap()).unwrap();
    assert_eq!(migrated_value["name"].as_str(), Some("a\""));
    assert_eq!(migrated_value["shader"]["kind"].as_str(), Some("project"));
    assert_eq!(
        migrated_value["shader"]["guid"].as_str(),
        Some(guid.to_string().as_str())
    );
    assert_eq!(fs::read_dir(&journal_directory).unwrap().count(), 0);
    for directory in [material.parent().unwrap(), shader_directory.as_path()] {
        assert!(!fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .any(|name| name.to_string_lossy().contains(".zr-migrate-")));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn commit_failure_rolls_back_every_document_and_cleans_transaction_files() {
    let root = fixture_root("transaction-rollback");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "71111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let first = root.join("assets/materials/first.zmaterial");
    let second = root.join("assets/materials/second.zmaterial");
    fs::create_dir_all(first.parent().unwrap()).unwrap();
    let original = format!(
        "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
    );
    fs::write(&first, &original).unwrap();
    fs::write(&second, &original).unwrap();

    let error = migrate_project_assets_with_commit_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        1,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::Transaction {
            phase: AssetMigrationTransactionPhase::Commit,
            ..
        }
    ));
    assert_eq!(fs::read_to_string(&first).unwrap(), original);
    assert_eq!(fs::read_to_string(&second).unwrap(), original);
    let transaction_artifacts = fs::read_dir(first.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".zr-migrate-"))
        .count();
    assert_eq!(transaction_artifacts, 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn dry_run_reports_pending_recovery_and_apply_converges_forward_without_backup_restore() {
    let root = fixture_root("transaction-process-recovery");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "d1111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let materials = root.join("assets/materials");
    fs::create_dir_all(&materials).unwrap();
    let original = format!(
        "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
    );
    let first = materials.join("first.zmaterial");
    let second = materials.join("second.zmaterial");
    fs::write(&first, &original).unwrap();
    fs::write(&second, &original).unwrap();

    migrate_project_assets_with_process_interruption(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        2,
    )
    .expect_err("simulated process interruption must leave recovery artifacts");
    let first_after_interruption = fs::read_to_string(&first).unwrap();
    assert_ne!(first_after_interruption, original);
    assert_eq!(fs::read_to_string(&second).unwrap(), original);
    let journal_directory = root.join(".zircon/asset-migration");
    let artifacts_before_dry_run = directory_snapshot(&materials);
    let journals_before_dry_run = directory_snapshot(&journal_directory);

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
    assert_eq!(
        fs::read_to_string(&first).unwrap(),
        first_after_interruption
    );
    assert_eq!(fs::read_to_string(&second).unwrap(), original);
    assert_eq!(directory_snapshot(&materials), artifacts_before_dry_run);
    assert_eq!(
        directory_snapshot(&journal_directory),
        journals_before_dry_run
    );

    // Block the next migration so it cannot hide restoration by disk recovery.
    let first_before_recovery = fs::read(&first).unwrap();
    let second_before_recovery = fs::read(&second).unwrap();
    let invalid_material = materials.join("blocked-recovery.zmaterial");
    fs::write(&invalid_material, b"version = [\n").unwrap();

    let blocked =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(!blocked.succeeded());
    assert!(blocked
        .issues()
        .iter()
        .any(|issue| issue.kind() == AssetMigrationIssueKind::InvalidDocument));
    assert_eq!(fs::read(&first).unwrap(), first_before_recovery);
    assert_eq!(fs::read(&second).unwrap(), second_before_recovery);
    assert_eq!(fs::read(&invalid_material).unwrap(), b"version = [\n");
    assert_eq!(fs::read_dir(&journal_directory).unwrap().count(), 0);
    assert!(!fs::read_dir(&materials)
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("zr-migrate")));
    fs::remove_file(&invalid_material).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(report.succeeded());
    assert_eq!(
        fs::read_to_string(&first).unwrap(),
        first_after_interruption
    );
    assert_ne!(fs::read_to_string(&second).unwrap(), original);
    assert_eq!(fs::read_dir(&journal_directory).unwrap().count(), 0);
    assert!(!fs::read_dir(&materials)
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("zr-migrate")));
    let first_migrated = fs::read(&first).unwrap();
    let second_migrated = fs::read(&second).unwrap();
    let second_apply =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(second_apply.succeeded());
    assert!(second_apply.changed_files().is_empty());
    assert_eq!(fs::read(&first).unwrap(), first_migrated);
    assert_eq!(fs::read(&second).unwrap(), second_migrated);

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn corrupted_recovery_journal_is_a_typed_failure_and_writes_nothing() {
    let root = fixture_root("transaction-corrupted-journal");
    write_manifest(&root, &["assets"]);
    fs::create_dir_all(root.join("assets")).unwrap();
    let source = root.join("assets/unchanged.txt");
    fs::write(&source, "unchanged").unwrap();
    let journal_directory = root.join(".zircon/asset-migration");
    fs::create_dir_all(&journal_directory).unwrap();
    fs::write(
        journal_directory.join("corrupted.zrjournal"),
        "not = [valid",
    )
    .unwrap();

    let error =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap_err();
    match error {
        crate::asset::migration::AssetMigrationError::InvalidJournal { reason, .. } => {
            assert!(!reason.is_empty());
        }
        other => panic!("expected typed invalid journal error, found {other}"),
    }
    assert_eq!(fs::read_to_string(source).unwrap(), "unchanged");
    assert!(journal_directory.join("corrupted.zrjournal").is_file());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malicious_journal_cannot_alias_an_artifact_to_a_live_target() {
    let root = fixture_root("transaction-malicious-alias");
    write_manifest(&root, &["assets"]);
    fs::create_dir_all(root.join("assets")).unwrap();
    let target = root.join("assets/unchanged.txt");
    fs::write(&target, "unchanged").unwrap();
    let journal_directory = root.join(".zircon/asset-migration");
    fs::create_dir_all(&journal_directory).unwrap();
    let journal = format!(
        "version = 1\nphase = \"active\"\n\n[[documents]]\nstate = \"prepared\"\ntarget = {:?}\nstaging = {:?}\n",
        target.to_string_lossy(),
        target.to_string_lossy(),
    );
    let journal_path = journal_directory.join("malicious.zrjournal");
    fs::write(&journal_path, &journal).unwrap();

    let error = migrate_project_assets(AssetMigrationOptions::new(
        &root,
        AssetMigrationMode::DryRun,
    ))
    .unwrap_err();
    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::InvalidJournal { .. }
    ));
    assert_eq!(fs::read_to_string(&target).unwrap(), "unchanged");
    assert_eq!(fs::read_to_string(journal_path).unwrap(), journal);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn forged_new_target_journal_cannot_delete_an_arbitrary_root_file() {
    let root = fixture_root("transaction-forged-root-delete");
    write_manifest(&root, &["assets"]);
    let assets = root.join("assets");
    fs::create_dir_all(&assets).unwrap();
    let target = assets.join("do-not-delete.txt");
    fs::write(&target, "protected root content").unwrap();
    let transaction_id = "999-1";
    let staging = assets.join(format!(
        ".do-not-delete.txt.zr-migrate-stage-{transaction_id}"
    ));
    fs::write(&staging, "protected root content").unwrap();
    let digest = blake3::hash(b"protected root content").to_hex();
    let journal_directory = root.join(".zircon/asset-migration");
    fs::create_dir_all(&journal_directory).unwrap();
    let journal_path = journal_directory.join(format!(
        ".do-not-delete.txt.zr-migrate-journal-{transaction_id}.zrjournal"
    ));
    let journal = format!(
        "version = 2\ntransaction_id = \"{transaction_id}\"\nphase = \"active\"\n\n[[documents]]\nstate = \"committed\"\ntarget_existed = false\nnew_digest = \"{digest}\"\ntarget = {:?}\nstaging = {:?}\n",
        target.to_string_lossy(),
        staging.to_string_lossy(),
    );
    fs::write(&journal_path, &journal).unwrap();

    let error =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap_err();
    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::InvalidJournal { .. }
    ));
    assert_eq!(
        fs::read_to_string(&target).unwrap(),
        "protected root content"
    );
    assert_eq!(
        fs::read_to_string(&staging).unwrap(),
        "protected root content"
    );
    assert_eq!(fs::read_to_string(&journal_path).unwrap(), journal);
    fs::remove_dir_all(root).unwrap();
}

// 恢复前仍须以扫描出的合法目标约束 journal；备份的摘要自洽不能赋予它覆盖现存文档的权限。
#[test]
fn forged_active_journal_cannot_overwrite_existing_target_from_backup() {
    let root = fixture_root("transaction-forged-existing-backup");
    write_manifest(&root, &["assets"]);
    let assets = root.join("assets");
    fs::create_dir_all(&assets).unwrap();
    let target = assets.join("live.zmaterial");
    let live = "version = 2\nname = \"live\"\n\n[shader]\nkind = \"builtin\"\nlocator = \"builtin://shader/pbr\"\n";
    let forged = "attacker-controlled backup bytes";
    fs::write(&target, live).unwrap();
    let transaction_id = "999-2";
    let staging = assets.join(format!(".live.zmaterial.zr-migrate-stage-{transaction_id}"));
    let backup = assets.join(format!(
        ".live.zmaterial.zr-migrate-backup-{transaction_id}"
    ));
    fs::write(&staging, live).unwrap();
    fs::write(&backup, forged).unwrap();
    let new_digest = blake3::hash(live.as_bytes()).to_hex();
    let original_digest = blake3::hash(forged.as_bytes()).to_hex();
    let journal_directory = root.join(".zircon/asset-migration");
    fs::create_dir_all(&journal_directory).unwrap();
    let journal_path = journal_directory.join(format!(
        ".live.zmaterial.zr-migrate-journal-{transaction_id}.zrjournal"
    ));
    let journal = format!(
        "version = 2\ntransaction_id = \"{transaction_id}\"\nphase = \"active\"\n\n[[documents]]\nstate = \"committed\"\ntarget_existed = true\noriginal_digest = \"{original_digest}\"\nnew_digest = \"{new_digest}\"\ntarget = {:?}\nstaging = {:?}\nbackup = {:?}\n",
        target.to_string_lossy(),
        staging.to_string_lossy(),
        backup.to_string_lossy(),
    );
    fs::write(&journal_path, &journal).unwrap();

    let before = directory_snapshot(&assets);
    let error =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap_err();

    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::InvalidJournal { .. }
    ));
    assert_eq!(fs::read_to_string(&target).unwrap(), live);
    assert_eq!(directory_snapshot(&assets), before);
    assert_eq!(fs::read_to_string(&journal_path).unwrap(), journal);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn forged_active_journal_cannot_restore_retired_backup_or_delete_current_sidecar() {
    let root = fixture_root("transaction-forged-retired-backup");
    write_manifest(&root, &["assets"]);
    let assets = root.join("assets");
    fs::create_dir_all(&assets).unwrap();
    let target = assets.join("live.asset.zmeta");
    let retired = assets.join("live.asset.meta.toml");
    let live = "format_version = 7\n";
    let forged = "attacker-controlled retired bytes";
    fs::write(&target, live).unwrap();
    let transaction_id = "999-3";
    let staging = assets.join(format!(
        ".live.asset.zmeta.zr-migrate-stage-{transaction_id}"
    ));
    let retired_backup = assets.join(format!(
        ".live.asset.meta.toml.zr-migrate-retired-backup-{transaction_id}"
    ));
    fs::write(&staging, live).unwrap();
    fs::write(&retired_backup, forged).unwrap();
    let new_digest = blake3::hash(live.as_bytes()).to_hex();
    let retired_digest = blake3::hash(forged.as_bytes()).to_hex();
    let journal_directory = root.join(".zircon/asset-migration");
    fs::create_dir_all(&journal_directory).unwrap();
    let journal_path = journal_directory.join(format!(
        ".live.asset.zmeta.zr-migrate-journal-{transaction_id}.zrjournal"
    ));
    let journal = format!(
        "version = 2\ntransaction_id = \"{transaction_id}\"\nphase = \"active\"\n\n[[documents]]\nstate = \"committed\"\ntarget_existed = false\nnew_digest = \"{new_digest}\"\nretired_digest = \"{retired_digest}\"\ntarget = {:?}\nstaging = {:?}\nretired_path = {:?}\nretired_backup = {:?}\n",
        target.to_string_lossy(),
        staging.to_string_lossy(),
        retired.to_string_lossy(),
        retired_backup.to_string_lossy(),
    );
    fs::write(&journal_path, &journal).unwrap();

    let before = directory_snapshot(&assets);
    let error =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap_err();

    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::InvalidJournal { .. }
    ));
    assert_eq!(fs::read_to_string(&target).unwrap(), live);
    assert!(!retired.exists());
    assert_eq!(directory_snapshot(&assets), before);
    assert_eq!(fs::read_to_string(&journal_path).unwrap(), journal);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn stage_write_and_backup_copy_failures_leave_no_local_artifacts() {
    for point in [0_u8, 1] {
        let root = fixture_root(&format!("stage-guard-{point}"));
        write_manifest(&root, &["assets"]);
        let guid: AssetUuid = "94111111-2222-4333-8444-555555555555".parse().unwrap();
        write_registered_source(
            &root,
            "assets",
            "shaders/pbr.zshader",
            guid,
            AssetKind::Shader,
        );
        let material = root.join("assets/materials/guard.zmaterial");
        fs::create_dir_all(material.parent().unwrap()).unwrap();
        let original = format!(
            "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
        );
        fs::write(&material, &original).unwrap();

        migrate_project_assets_with_stage_fault(
            AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
            usize::from(point),
            point,
        )
        .expect_err("injected stage failure must remain typed");
        assert_eq!(fs::read_to_string(&material).unwrap(), original);
        assert!(!fs::read_dir(material.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().contains("zr-migrate")));
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn retired_backup_sync_failure_leaves_no_local_artifacts() {
    let root = fixture_root("stage-guard-retired-sync");
    write_manifest(&root, &["assets"]);
    let source = root.join("assets/shaders/legacy.zshader");
    fs::create_dir_all(source.parent().unwrap()).unwrap();
    fs::write(&source, "shader").unwrap();
    let retired = root.join("assets/shaders/legacy.zshader.meta.toml");
    fs::write(
        &retired,
        "format_version = 6\nuuid = \"95111111-2222-4333-8444-555555555555\"\nurl = \"res://shaders/legacy.zshader\"\nasset_kind = \"Shader\"\nsource_hash = \"old\"\n",
    )
    .unwrap();

    migrate_project_assets_with_stage_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        0,
        2,
    )
    .expect_err("retired backup sync failure must remain typed");
    assert!(retired.is_file());
    assert!(!root.join("assets/shaders/legacy.zshader.zmeta").exists());
    assert!(!fs::read_dir(retired.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("zr-migrate")));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn cleanup_phase_is_reentrant_when_a_backup_was_already_deleted() {
    let root = fixture_root("transaction-cleanup-reentry");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "a2111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let material = root.join("assets/materials/hero.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
        ),
    )
    .unwrap();

    migrate_project_assets_with_terminal_interruption(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        true,
    )
    .expect_err("cleanup interruption must retain journal ownership");
    let committed = fs::read_to_string(&material).unwrap();
    assert!(committed.contains("kind = \"project\""));
    let backup = fs::read_dir(material.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("zr-migrate-backup")
        })
        .expect("cleanup-state fixture retains a backup");
    fs::remove_file(backup.path()).unwrap();

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(report.succeeded());
    assert_eq!(fs::read_to_string(&material).unwrap(), committed);
    assert_eq!(
        fs::read_dir(root.join(".zircon/asset-migration"))
            .unwrap()
            .count(),
        0
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rollback_completed_phase_reenters_cleanup_without_restoring_again() {
    verify_rollback_cleanup_reentry("rollback-cleanup-interruption", false);
}

#[test]
fn rollback_journal_delete_failure_reenters_with_missing_artifacts() {
    verify_rollback_cleanup_reentry("rollback-journal-delete-failure", true);
}

fn verify_rollback_cleanup_reentry(name: &str, fail_journal_delete: bool) {
    let root = fixture_root(name);
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "61111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let materials = root.join("assets/materials");
    fs::create_dir_all(&materials).unwrap();
    let original = format!(
        "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
    );
    let first = materials.join("first.zmaterial");
    let second = materials.join("second.zmaterial");
    fs::write(&first, &original).unwrap();
    fs::write(&second, &original).unwrap();

    migrate_project_assets_with_rollback_cleanup_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        1,
        fail_journal_delete,
    )
    .expect_err("injected rollback cleanup failure must retain its journal");
    assert_eq!(fs::read_to_string(&first).unwrap(), original);
    assert_eq!(fs::read_to_string(&second).unwrap(), original);
    let journal_directory = root.join(".zircon/asset-migration");
    let journal = fs::read_dir(&journal_directory)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let journal_bytes = fs::read(&journal).unwrap();
    let journal_source = String::from_utf8_lossy(&journal_bytes);
    let expected_phase = if fail_journal_delete {
        "phase = \"cleanup_rollback\""
    } else {
        "phase = \"rollback_completed\""
    };
    assert!(journal_source.contains(expected_phase));

    migrate_project_assets_with_commit_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        0,
    )
    .expect_err("recovery must finish before the next injected commit failure");
    assert_eq!(fs::read_to_string(&first).unwrap(), original);
    assert_eq!(fs::read_to_string(&second).unwrap(), original);
    assert_eq!(fs::read_dir(&journal_directory).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn restore_failure_is_typed_and_retains_a_recovery_backup() {
    let root = fixture_root("transaction-restore-failure");
    write_manifest(&root, &["assets"]);
    let guid: AssetUuid = "c1111111-2222-4333-8444-555555555555".parse().unwrap();
    write_registered_source(
        &root,
        "assets",
        "shaders/pbr.zshader",
        guid,
        AssetKind::Shader,
    );
    let materials = root.join("assets/materials");
    fs::create_dir_all(&materials).unwrap();
    let original = format!(
        "version = 2\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
    );
    fs::write(materials.join("first.zmaterial"), &original).unwrap();
    fs::write(materials.join("second.zmaterial"), &original).unwrap();

    let error = migrate_project_assets_with_restore_fault(
        AssetMigrationOptions::new(&root, AssetMigrationMode::Apply),
        1,
        0,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        crate::asset::migration::AssetMigrationError::Transaction {
            phase: AssetMigrationTransactionPhase::Rollback,
            ..
        }
    ));
    let backups = fs::read_dir(&materials)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .contains("zr-migrate-backup")
        })
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 2);
    assert!(backups
        .iter()
        .all(|backup| fs::read_to_string(backup.path()).unwrap() == original));

    let report =
        migrate_project_assets(AssetMigrationOptions::new(&root, AssetMigrationMode::Apply))
            .unwrap();
    assert!(report.succeeded());
    for name in ["first.zmaterial", "second.zmaterial"] {
        assert!(fs::read_to_string(materials.join(name))
            .unwrap()
            .contains("kind = \"project\""));
    }
    assert!(!fs::read_dir(&materials)
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().contains("zr-migrate")));
    assert_eq!(
        fs::read_dir(root.join(".zircon/asset-migration"))
            .unwrap()
            .count(),
        0
    );
    fs::remove_dir_all(root).unwrap();
}
