use std::path::Path;

use super::{
    promotion, receipt_io, staging, ZrPackDeltaInstallError, ZrPackDeltaInstallReport,
    ZrPackInstallReceipt, ZrPackPromotionReport,
};

/// pack 增量安装的公开门面：先校验并重建 staging，再以可恢复事务发布 installed，最后可写入 receipt。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZrPackDeltaInstaller;

impl ZrPackDeltaInstaller {
    /// 用 base 与 delta 生成目标 pack；失败时不会发布 installed 文件。
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

    /// 将已验证的 staging pack 原子提升为 installed，并可保留旧文件的 backup。
    pub fn promote_staged_pack(
        staged_pack: impl AsRef<Path>,
        installed_pack: impl AsRef<Path>,
        backup_pack: Option<impl AsRef<Path>>,
    ) -> Result<ZrPackPromotionReport, ZrPackDeltaInstallError> {
        promotion::promote_staged_pack(staged_pack.as_ref(), installed_pack.as_ref(), backup_pack)
    }

    /// 在重试安装前恢复上次未完成的 promotion journal。
    pub fn recover_pending_promotion(
        staged_pack: impl AsRef<Path>,
        installed_pack: impl AsRef<Path>,
        backup_pack: Option<impl AsRef<Path>>,
    ) -> Result<(), ZrPackDeltaInstallError> {
        super::promotion_journal::PromotionPaths::new(
            staged_pack.as_ref(),
            installed_pack.as_ref(),
            backup_pack.as_ref().map(AsRef::as_ref),
        )?
        .recover()
    }

    /// 组合恢复、delta 应用和 promotion；若目标已提交则校验后返回 AlreadyInstalled。
    pub fn install_delta(
        base_pack: impl AsRef<Path>,
        delta_pack: impl AsRef<Path>,
        staged_pack: impl AsRef<Path>,
        installed_pack: impl AsRef<Path>,
        backup_pack: Option<impl AsRef<Path>>,
    ) -> Result<(ZrPackDeltaInstallReport, ZrPackPromotionReport), ZrPackDeltaInstallError> {
        super::delta_workflow::install_delta(
            base_pack.as_ref(),
            delta_pack.as_ref(),
            staged_pack.as_ref(),
            installed_pack.as_ref(),
            backup_pack.as_ref().map(AsRef::as_ref),
        )
    }

    /// 仅在 staging 与 promotion 报告一致且 delta 已验证时写入 receipt。
    pub fn write_install_receipt(
        receipt_path: impl AsRef<Path>,
        staging_report: &ZrPackDeltaInstallReport,
        promotion_report: &ZrPackPromotionReport,
    ) -> Result<ZrPackInstallReceipt, ZrPackDeltaInstallError> {
        receipt_io::write_install_receipt(receipt_path.as_ref(), staging_report, promotion_report)
    }

    /// 读取并校验当前支持版本的安装 receipt。
    pub fn read_install_receipt(
        receipt_path: impl AsRef<Path>,
    ) -> Result<ZrPackInstallReceipt, ZrPackDeltaInstallError> {
        receipt_io::read_install_receipt(receipt_path.as_ref())
    }
}
