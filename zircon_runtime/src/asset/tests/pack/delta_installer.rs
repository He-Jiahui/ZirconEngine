use super::*;

#[test]
fn delta_installer_rebuilds_target_pack_to_staging() {
    let root = unique_pack_temp_dir("delta-install");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/added.bin", b"added".to_vec()),
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();

    let report =
        ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path).unwrap();

    let staged_reader = ZrPackReader::from_bytes(fs::read(&staged_path).unwrap()).unwrap();
    assert_eq!(report.base_pack, base_path);
    assert_eq!(report.delta_pack, delta_path);
    assert_eq!(report.staged_pack, staged_path);
    assert_eq!(report.target_manifest, target_reader.manifest().clone());
    assert!(report.delta_apply_verified);
    assert_eq!(
        report.staged_size,
        fs::metadata(&report.staged_pack).unwrap().len()
    );
    assert_eq!(
        staged_reader.read_asset("meshes/added.bin").unwrap(),
        b"added"
    );
    assert_eq!(
        staged_reader.read_asset("meshes/keep.bin").unwrap(),
        b"keep"
    );
    assert_eq!(
        staged_reader.read_asset("textures/changed.bin").unwrap(),
        b"new"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_rejects_wrong_base_without_staging() {
    let root = unique_pack_temp_dir("delta-install-wrong-base");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let wrong_base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"wrong".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, wrong_base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();

    let error = ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path)
        .unwrap_err();

    assert_eq!(
        error,
        ZrPackDeltaInstallError::Pack(ZrPackError::DeltaBaseManifestMismatch)
    );
    assert!(!staged_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_promotes_staged_pack_with_backup() {
    let root = unique_pack_temp_dir("delta-promote");
    let installed_path = root.join("installed").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let staged_path = root.join("staging").join("assets.zrpack");
    let installed = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let staged = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let staged_reader = ZrPackReader::from_bytes(staged.bytes.clone()).unwrap();
    fs::create_dir_all(installed_path.parent().unwrap()).unwrap();
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&installed_path, installed.bytes).unwrap();
    fs::write(&staged_path, staged.bytes).unwrap();

    let report = ZrPackDeltaInstaller::promote_staged_pack(
        &staged_path,
        &installed_path,
        Some(&backup_path),
    )
    .unwrap();

    let installed_reader = ZrPackReader::from_bytes(fs::read(&installed_path).unwrap()).unwrap();
    let backup_reader = ZrPackReader::from_bytes(fs::read(&backup_path).unwrap()).unwrap();
    assert_eq!(report.installed_pack, installed_path);
    assert_eq!(report.backup_pack, Some(backup_path.clone()));
    assert_eq!(report.staged_pack, staged_path);
    assert_eq!(report.installed_manifest, staged_reader.manifest().clone());
    assert_eq!(
        report.installed_size,
        fs::metadata(&report.installed_pack).unwrap().len()
    );
    assert_eq!(
        installed_reader.read_asset("textures/changed.bin").unwrap(),
        b"new"
    );
    assert_eq!(
        backup_reader.read_asset("textures/changed.bin").unwrap(),
        b"old"
    );
    assert!(!report.staged_pack.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_copies_staged_pack_when_promotion_rename_fails() {
    let root = unique_pack_temp_dir("delta-promote-copy-fallback");
    let installed_path = root.join("installed").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let staged_path = root.join("staging").join("assets.zrpack");
    let installed = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"old".to_vec(),
    )])
    .unwrap();
    let staged = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"new".to_vec(),
    )])
    .unwrap();
    let staged_reader = ZrPackReader::from_bytes(staged.bytes.clone()).unwrap();
    fs::create_dir_all(installed_path.parent().unwrap()).unwrap();
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&installed_path, installed.bytes).unwrap();
    fs::write(&staged_path, staged.bytes).unwrap();

    let report = ZrPackDeltaInstaller::promote_staged_pack_with_forced_staged_rename_failure(
        &staged_path,
        &installed_path,
        Some(&backup_path),
    )
    .unwrap();

    let installed_reader = ZrPackReader::from_bytes(fs::read(&installed_path).unwrap()).unwrap();
    let backup_reader = ZrPackReader::from_bytes(fs::read(&backup_path).unwrap()).unwrap();
    assert_eq!(
        report.promotion_method,
        ZrPackPromotionMethod::CopiedAfterRenameFailure
    );
    assert_eq!(report.installed_manifest, staged_reader.manifest().clone());
    assert_eq!(
        installed_reader.read_asset("textures/changed.bin").unwrap(),
        b"new"
    );
    assert_eq!(
        backup_reader.read_asset("textures/changed.bin").unwrap(),
        b"old"
    );
    assert!(!staged_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_rejects_invalid_staged_pack_without_replacing_installed() {
    let root = unique_pack_temp_dir("delta-promote-invalid");
    let installed_path = root.join("installed").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let staged_path = root.join("staging").join("assets.zrpack");
    let installed = ZrPackWriter::write([ZrPackInputAsset::new(
        "textures/changed.bin",
        b"old".to_vec(),
    )])
    .unwrap();
    fs::create_dir_all(installed_path.parent().unwrap()).unwrap();
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&installed_path, installed.bytes).unwrap();
    fs::write(&staged_path, b"not-a-pack").unwrap();

    let error = ZrPackDeltaInstaller::promote_staged_pack(
        &staged_path,
        &installed_path,
        Some(&backup_path),
    )
    .unwrap_err();

    let installed_reader = ZrPackReader::from_bytes(fs::read(&installed_path).unwrap()).unwrap();
    assert_eq!(
        error,
        ZrPackDeltaInstallError::Pack(ZrPackError::HeaderTooSmall)
    );
    assert_eq!(
        installed_reader.read_asset("textures/changed.bin").unwrap(),
        b"old"
    );
    assert!(staged_path.exists());
    assert!(!backup_path.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_writes_install_receipt_from_staging_and_promotion() {
    let root = unique_pack_temp_dir("delta-receipt");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let receipt_path = root.join("receipts").join("assets.install.json");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();
    let staging_report =
        ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path).unwrap();
    let promotion_report =
        ZrPackDeltaInstaller::promote_staged_pack(&staged_path, &base_path, Some(&backup_path))
            .unwrap();

    let receipt = ZrPackDeltaInstaller::write_install_receipt(
        &receipt_path,
        &staging_report,
        &promotion_report,
    )
    .unwrap();
    let read_receipt = ZrPackDeltaInstaller::read_install_receipt(&receipt_path).unwrap();

    assert_eq!(receipt, read_receipt);
    assert_eq!(receipt.base_pack, base_path);
    assert_eq!(receipt.delta_pack, delta_path);
    assert_eq!(receipt.staged_pack, staged_path);
    assert_eq!(receipt.installed_pack, promotion_report.installed_pack);
    assert_eq!(receipt.backup_pack, Some(backup_path));
    assert_eq!(receipt.target_manifest, target_reader.manifest().clone());
    assert_eq!(receipt.installed_manifest, target_reader.manifest().clone());
    assert_eq!(receipt.staged_size, staging_report.staged_size);
    assert_eq!(receipt.installed_size, promotion_report.installed_size);
    assert!(receipt.delta_apply_verified);
    assert_eq!(receipt.promotion_method, ZrPackPromotionMethod::Renamed);
    assert!(receipt.promoted);
    assert_eq!(
        receipt.format_version,
        ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION
    );
    assert!(fs::read_to_string(&receipt_path)
        .unwrap()
        .contains("\"format_version\""));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_receipt_records_copy_fallback_promotion_method() {
    let root = unique_pack_temp_dir("delta-receipt-copy-fallback");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let receipt_path = root.join("receipts").join("assets.install.json");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();
    let staging_report =
        ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path).unwrap();
    let promotion_report =
        ZrPackDeltaInstaller::promote_staged_pack_with_forced_staged_rename_failure(
            &staged_path,
            &base_path,
            Some(&backup_path),
        )
        .unwrap();

    let receipt = ZrPackDeltaInstaller::write_install_receipt(
        &receipt_path,
        &staging_report,
        &promotion_report,
    )
    .unwrap();
    let read_receipt = ZrPackDeltaInstaller::read_install_receipt(&receipt_path).unwrap();

    assert_eq!(
        promotion_report.promotion_method,
        ZrPackPromotionMethod::CopiedAfterRenameFailure
    );
    assert_eq!(
        receipt.promotion_method,
        ZrPackPromotionMethod::CopiedAfterRenameFailure
    );
    assert_eq!(
        read_receipt.promotion_method,
        ZrPackPromotionMethod::CopiedAfterRenameFailure
    );
    assert!(fs::read_to_string(&receipt_path)
        .unwrap()
        .contains("\"promotion_method\""));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn delta_installer_rejects_receipt_for_mismatched_reports() {
    let root = unique_pack_temp_dir("delta-receipt-mismatch");
    let base_path = root.join("installed").join("assets.zrpack");
    let delta_path = root.join("downloads").join("assets.delta.zrpd");
    let staged_path = root.join("staging").join("assets.zrpack");
    let backup_path = root.join("backup").join("assets.previous.zrpack");
    let receipt_path = root.join("receipts").join("assets.install.json");
    let base = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"old".to_vec()),
    ])
    .unwrap();
    let target = ZrPackWriter::write([
        ZrPackInputAsset::new("meshes/keep.bin", b"keep".to_vec()),
        ZrPackInputAsset::new("textures/changed.bin", b"new".to_vec()),
    ])
    .unwrap();
    let base_reader = ZrPackReader::from_bytes(base.bytes.clone()).unwrap();
    let target_reader = ZrPackReader::from_bytes(target.bytes).unwrap();
    let delta = ZrPackDeltaWriter::write(&base_reader, &target_reader).unwrap();
    fs::create_dir_all(base_path.parent().unwrap()).unwrap();
    fs::create_dir_all(delta_path.parent().unwrap()).unwrap();
    fs::write(&base_path, base.bytes).unwrap();
    fs::write(&delta_path, delta.bytes).unwrap();
    let staging_report =
        ZrPackDeltaInstaller::rebuild_to_staging(&base_path, &delta_path, &staged_path).unwrap();
    let mut promotion_report =
        ZrPackDeltaInstaller::promote_staged_pack(&staged_path, &base_path, Some(&backup_path))
            .unwrap();
    promotion_report.staged_pack = root.join("staging").join("other.zrpack");

    let error = ZrPackDeltaInstaller::write_install_receipt(
        &receipt_path,
        &staging_report,
        &promotion_report,
    )
    .unwrap_err();

    match error {
        ZrPackDeltaInstallError::ReceiptReportMismatch(message) => {
            assert!(message.contains("staged pack"));
        }
        other => panic!("unexpected install receipt error: {other:?}"),
    }
    assert!(!receipt_path.exists());
    let _ = fs::remove_dir_all(root);
}
