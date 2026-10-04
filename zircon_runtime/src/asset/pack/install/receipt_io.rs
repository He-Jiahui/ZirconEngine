use std::path::Path;

use super::{
    file_io::read_pack_file, ZrPackDeltaInstallError, ZrPackDeltaInstallReport,
    ZrPackInstallReceipt, ZrPackPromotionReport, ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION,
};

// receipt 只在两个阶段报告已互相校验后生成，并拒绝覆盖任一输入、installed 或 backup 路径。
pub(super) fn write_install_receipt(
    receipt_path: &Path,
    staging_report: &ZrPackDeltaInstallReport,
    promotion_report: &ZrPackPromotionReport,
) -> Result<ZrPackInstallReceipt, ZrPackDeltaInstallError> {
    validate_receipt_reports(staging_report, promotion_report)?;
    for pack_path in [
        Some(staging_report.base_pack.as_path()),
        Some(staging_report.delta_pack.as_path()),
        Some(staging_report.staged_pack.as_path()),
        Some(promotion_report.installed_pack.as_path()),
        promotion_report.backup_pack.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if super::promotion_journal::paths_alias(receipt_path, pack_path)? {
            return Err(ZrPackDeltaInstallError::InvalidPromotionPaths(
                "receipt must not overwrite a pack input, installed pack or backup".into(),
            ));
        }
    }
    let receipt = ZrPackInstallReceipt {
        format_version: ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION,
        base_pack: staging_report.base_pack.clone(),
        delta_pack: staging_report.delta_pack.clone(),
        staged_pack: staging_report.staged_pack.clone(),
        installed_pack: promotion_report.installed_pack.clone(),
        backup_pack: promotion_report.backup_pack.clone(),
        target_manifest: staging_report.target_manifest.clone(),
        installed_manifest: promotion_report.installed_manifest.clone(),
        staged_size: staging_report.staged_size,
        installed_size: promotion_report.installed_size,
        delta_apply_verified: staging_report.delta_apply_verified,
        promotion_method: promotion_report.promotion_method,
        promoted: true,
    };
    let bytes = serde_json::to_vec_pretty(&receipt)
        .map_err(|error| ZrPackDeltaInstallError::ReceiptEncode(error.to_string()))?;
    crate::core::resource::io::atomic_write(receipt_path, &bytes).map_err(|error| {
        ZrPackDeltaInstallError::WriteFailed {
            path: receipt_path.to_path_buf(),
            error: error.to_string(),
        }
    })?;
    Ok(receipt)
}

// 读取后立即校验格式版本；未知版本不会被当作当前事务结果继续使用。
pub(super) fn read_install_receipt(
    receipt_path: &Path,
) -> Result<ZrPackInstallReceipt, ZrPackDeltaInstallError> {
    let bytes = read_pack_file(receipt_path)?;
    let receipt = serde_json::from_slice::<ZrPackInstallReceipt>(&bytes)
        .map_err(|error| ZrPackDeltaInstallError::ReceiptDecode(error.to_string()))?;
    if receipt.format_version != ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION {
        return Err(ZrPackDeltaInstallError::ReceiptReportMismatch(format!(
            "install receipt format_version {} is unsupported",
            receipt.format_version
        )));
    }
    Ok(receipt)
}

// staging 的目标 manifest、staged 路径和 delta_apply_verified 是 receipt 可审计性的最小一致性条件。
fn validate_receipt_reports(
    staging_report: &ZrPackDeltaInstallReport,
    promotion_report: &ZrPackPromotionReport,
) -> Result<(), ZrPackDeltaInstallError> {
    if !staging_report.delta_apply_verified {
        return Err(ZrPackDeltaInstallError::ReceiptReportMismatch(
            "staging report did not verify delta apply".to_string(),
        ));
    }
    if staging_report.staged_pack != promotion_report.staged_pack {
        return Err(ZrPackDeltaInstallError::ReceiptReportMismatch(format!(
            "staged pack mismatch: staging report has {}, promotion report has {}",
            staging_report.staged_pack.display(),
            promotion_report.staged_pack.display()
        )));
    }
    if staging_report.target_manifest != promotion_report.installed_manifest {
        return Err(ZrPackDeltaInstallError::ReceiptReportMismatch(
            "target manifest does not match installed manifest".to_string(),
        ));
    }
    Ok(())
}
