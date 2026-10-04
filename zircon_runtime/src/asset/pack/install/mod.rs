//! pack 增量安装按 staging、promotion、恢复 journal 和 receipt 四个阶段组织；模块外只暴露事务门面及结果类型。

mod delta_workflow;
mod error;
mod file_io;
mod installer;
mod promotion;
mod promotion_journal;
mod promotion_report;
mod receipt;
mod receipt_io;
mod staging;
mod staging_report;

pub use error::ZrPackDeltaInstallError;
pub use installer::ZrPackDeltaInstaller;
pub use promotion_report::{ZrPackPromotionMethod, ZrPackPromotionReport};
pub use receipt::{ZrPackInstallReceipt, ZRPACK_INSTALL_RECEIPT_FORMAT_VERSION};
pub use staging_report::ZrPackDeltaInstallReport;
