use std::path::Path;

use super::{
    promotion, receipt_io, staging, ZrPackDeltaInstallError, ZrPackDeltaInstallReport,
    ZrPackInstallReceipt, ZrPackPromotionReport,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZrPackDeltaInstaller;

impl ZrPackDeltaInstaller {
    pub fn rebuild_to_staging(
        base_pack: impl AsRef<Path>,
        delta_pack: impl AsRef<Path>,
        staged_pack: impl AsRef<Path>,
    ) -> Result<ZrPackDeltaInstallReport, ZrPackDeltaInstallError> {
        staging::rebuild_to_staging(
            base_pack.as_ref(),
            delta_pack.as_ref(),
            staged_pack.as_ref(),
        )
    }

    pub fn promote_staged_pack(
        staged_pack: impl AsRef<Path>,
        installed_pack: impl AsRef<Path>,
        backup_pack: Option<impl AsRef<Path>>,
    ) -> Result<ZrPackPromotionReport, ZrPackDeltaInstallError> {
        promotion::promote_staged_pack(staged_pack.as_ref(), installed_pack.as_ref(), backup_pack)
    }

    #[cfg(test)]
    pub(crate) fn promote_staged_pack_with_forced_staged_rename_failure(
        staged_pack: impl AsRef<Path>,
        installed_pack: impl AsRef<Path>,
        backup_pack: Option<impl AsRef<Path>>,
    ) -> Result<ZrPackPromotionReport, ZrPackDeltaInstallError> {
        promotion::promote_staged_pack_with_forced_staged_rename_failure(
            staged_pack.as_ref(),
            installed_pack.as_ref(),
            backup_pack,
        )
    }

    pub fn write_install_receipt(
        receipt_path: impl AsRef<Path>,
        staging_report: &ZrPackDeltaInstallReport,
        promotion_report: &ZrPackPromotionReport,
    ) -> Result<ZrPackInstallReceipt, ZrPackDeltaInstallError> {
        receipt_io::write_install_receipt(receipt_path.as_ref(), staging_report, promotion_report)
    }

    pub fn read_install_receipt(
        receipt_path: impl AsRef<Path>,
    ) -> Result<ZrPackInstallReceipt, ZrPackDeltaInstallError> {
        receipt_io::read_install_receipt(receipt_path.as_ref())
    }
}
