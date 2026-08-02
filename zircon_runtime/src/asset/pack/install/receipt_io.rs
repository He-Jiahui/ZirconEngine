use std::path::Path;

use super::{
    ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION, ZrPackDeltaInstallError, ZrPackDeltaInstallReport,
    ZrPackInstallReceipt, ZrPackPromotionReport,
    file_io::{read_pack_file, write_pack_file},
};

pub(super) fn write_install_receipt(
    receipt_path: &Path,
    staging_report: &ZrPackDeltaInstallReport,
    promotion_report: &ZrPackPromotionReport,
) -> Result<ZrPackInstallReceipt, ZrPackDeltaInstallError> {
    validate_receipt_reports(staging_report, promotion_report)?;
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
    write_pack_file(receipt_path, &bytes)?;
    Ok(receipt)
}

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
